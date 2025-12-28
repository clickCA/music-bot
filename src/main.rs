mod cleanup;
mod events;
mod music;
mod queue;
mod shutdown;

use cleanup::{ActiveFiles, cleanup_all_temp_files, cleanup_guild_temp_files};
use events::TrackEndNotifier;
use music::create_source;
use queue::{Queue, QueueMap};
use shutdown::ShutdownHandler;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::GuildId},
    prelude::*,
};
use songbird::{Call, EventContext, TrackEvent};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

struct Handler {
    queues: QueueMap,
    active_files: ActiveFiles,
}

impl Handler {
    fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
            active_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_or_create_queue(&self, guild_id: u64) -> Queue {
        let mut queues = self.queues.lock().await;
        queues
            .entry(guild_id)
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .clone()
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.trim();

        if !content.starts_with('!') {
            return;
        }

        let args: Vec<&str> = content.splitn(2, ' ').collect();
        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return,
        };

        match args[0] {
            "!join" => {
                let channel_id = msg
                    .member
                    .as_ref()
                    .and_then(|m| m.voice.as_ref())
                    .and_then(|v| v.channel_id);

                let connect_to = match channel_id {
                    Some(channel) => channel,
                    None => {
                        let _ = msg.reply(&ctx.http, "You need to be in a voice channel!").await;
                        return;
                    }
                };

                let manager = songbird::get(&ctx).await.unwrap();
                let _ = manager.join(guild_id, connect_to).await;
                let _ = msg.reply(&ctx.http, format!("Joined <#{}>", connect_to)).await;
            }

            "!play" => {
                if args.len() < 2 {
                    let _ = msg
                        .reply(&ctx.http, "Usage: !play <song name or YouTube URL>")
                        .await;
                    return;
                }

                let query = args[1].to_string();
                let queue = self.get_or_create_queue(guild_id.0).await;

                // Ensure bot is in voice channel
                let channel_id = msg
                    .member
                    .as_ref()
                    .and_then(|m| m.voice.as_ref())
                    .and_then(|v| v.channel_id);

                let connect_to = match channel_id {
                    Some(channel) => channel,
                    None => {
                        let _ = msg.reply(&ctx.http, "You need to be in a voice channel!").await;
                        return;
                    }
                };

                let manager = songbird::get(&ctx).await.unwrap();
                let has_handler = manager.get(guild_id).is_some();

                if !has_handler {
                    let _ = manager.join(guild_id, connect_to).await;
                }

                // Add to queue
                let mut queue_lock = queue.lock().await;
                queue_lock.push(query.clone());
                let queue_len = queue_lock.len();
                drop(queue_lock);

                if queue_len == 1 {
                    // Play immediately if queue was empty
                    if let Some(handler_lock) = manager.get(guild_id) {
                        let mut handler = handler_lock.lock().await;

                        match create_source(&guild_id.0, &query).await {
                            Ok((source, file_path)) => {
                                // Mark file as active
                                if let Some(ref path) = file_path {
                                    let mut active = self.active_files.lock().await;
                                    active.entry(guild_id.0).or_default().insert(path.clone());
                                }

                                let handle = handler.play_input(source);

                                // Add event handler for when track ends
                                let _ = handle.add_event(
                                    songbird::Event::Track(TrackEvent::End),
                                    TrackEndNotifier {
                                        guild_id,
                                        call: handler_lock.clone(),
                                        queue: queue.clone(),
                                        active_files: self.active_files.clone(),
                                        downloaded_file: file_path,
                                    },
                                );

                                let _ = msg.reply(&ctx.http, "🎵 Now playing!").await;
                            }
                            Err(e) => {
                                let _ = msg
                                    .reply(&ctx.http, format!("Error playing song: {}", e))
                                    .await;
                                queue.lock().await.remove(0);
                            }
                        }
                    }
                } else {
                    let _ = msg
                        .reply(&ctx.http, format!("Added to queue (position {})", queue_len))
                        .await;
                }
            }

            "!pause" => {
                let manager = songbird::get(&ctx).await.unwrap();
                if let Some(handler_lock) = manager.get(guild_id) {
                    let handler = handler_lock.lock().await;
                    let _ = handler.queue().pause();
                    let _ = msg.reply(&ctx.http, "Paused ⏸️").await;
                }
            }

            "!resume" => {
                let manager = songbird::get(&ctx).await.unwrap();
                if let Some(handler_lock) = manager.get(guild_id) {
                    let handler = handler_lock.lock().await;
                    let _ = handler.queue().resume();
                    let _ = msg.reply(&ctx.http, "Resumed ▶️").await;
                }
            }

            "!skip" => {
                let manager = songbird::get(&ctx).await.unwrap();
                if let Some(handler_lock) = manager.get(guild_id) {
                    let handler = handler_lock.lock().await;
                    let _ = handler.queue().skip();
                    let _ = msg.reply(&ctx.http, "Skipped ⏭️").await;
                }
            }

            "!stop" => {
                let queue = self.get_or_create_queue(guild_id.0).await;
                queue.lock().await.clear();

                let manager = songbird::get(&ctx).await.unwrap();
                if let Some(handler_lock) = manager.get(guild_id) {
                    let handler = handler_lock.lock().await;
                    handler.queue().stop();
                }
                let _ = msg.reply(&ctx.http, "Stopped and cleared queue ⏹️").await;
            }

            "!queue" => {
                let queue = self.get_or_create_queue(guild_id.0).await;
                let queue_lock = queue.lock().await;

                if queue_lock.is_empty() {
                    let _ = msg.reply(&ctx.http, "Queue is empty!").await;
                } else {
                    let queue_list: String = queue_lock
                        .iter()
                        .enumerate()
                        .map(|(i, song)| {
                            if i == 0 {
                                format!("▶️ {}", song)
                            } else {
                                format!("{}. {}", i, song)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    let _ = msg
                        .reply(&ctx.http, format!("**Queue:**\n{}", queue_list))
                        .await;
                }
            }

            "!leave" => {
                let manager = songbird::get(&ctx).await.unwrap();
                if manager.get(guild_id).is_some() {
                    let _ = manager.remove(guild_id).await;
                    let _ = msg.reply(&ctx.http, "Left the voice channel 👋").await;
                }
            }

            "!shutdown" => {
                // Check if user has permission to shutdown
                if let Some(member) = &msg.member {
                    if let Ok(permissions) = member.permissions(&ctx.http).await {
                        if permissions.administrator() {
                            let _ = msg.reply(&ctx.http, "🛑 Initiating graceful shutdown...").await;
                            // Trigger graceful shutdown
                            let shutdown = ShutdownHandler::new(self.queues.clone());
                            tokio::spawn(async move {
                                shutdown.run().await;
                            });
                        } else {
                            let _ = msg.reply(&ctx.http, "❌ You need administrator permissions to shut down the bot.").await;
                        }
                    }
                }
            }

            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let handler = Handler::new();
    let queues = handler.queues.clone();
    let active_files = handler.active_files.clone();

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    // Start periodic cleanup task (runs every hour for all guilds)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            // Clean up old files for all guilds
            let guilds: Vec<u64> = {
                let active = active_files.lock().await;
                active.keys().copied().collect()
            };
            for guild_id in guilds {
                cleanup_guild_temp_files(guild_id, &active_files).await;
            }
        }
    });

    // Initial cleanup on startup
    cleanup_all_temp_files().await;

    // Create shutdown handler
    let shutdown = ShutdownHandler::new(queues);
    let shutdown_trigger = shutdown.is_shutting_down.clone();

    // Spawn shutdown handler
    tokio::spawn(async move {
        shutdown.run().await;
    });

    // Start the Discord client
    let shard_manager = client.shard_manager.clone();

    tokio::select! {
        // Run the bot normally
        result = client.start() => {
            if let Err(why) = result {
                println!("Client error: {:?}", why);
            }
        }
        // Wait for shutdown signal
        _ = async {
            // Check every second if shutdown was triggered
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if *shutdown_trigger.lock().await {
                    println!("🛑 Shutdown signal received, stopping bot...");
                    shard_manager.shutdown_all().await;
                    break;
                }
            }
        } => {}
    }
}
