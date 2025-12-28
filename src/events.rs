use serenity::async_trait;
use songbird::{
    Call, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};
use std::path::PathBuf;
use sync::Arc;

use crate::cleanup::{cleanup_guild_temp_files, ActiveFiles};
use crate::music::create_source;
use crate::queue::Queue;

/// Track end notification handler
pub struct TrackEndNotifier {
    pub guild_id: songbird::model::id::GuildId,
    pub call: Arc<tokio::sync::Mutex<Call>>,
    pub queue: Queue,
    pub active_files: ActiveFiles,
    pub downloaded_file: Option<PathBuf>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<songbird::Event> {
        // Remove this file from active set
        if let Some(ref file_path) = self.downloaded_file {
            let mut active = self.active_files.lock().await;
            if let Some(files) = active.get_mut(&self.guild_id.0) {
                files.remove(file_path);
            }
        }

        // Clean up old files for this guild (only inactive ones)
        cleanup_guild_temp_files(self.guild_id.0, &self.active_files).await;

        // Remove the finished song from queue
        let mut queue = self.queue.lock().await;
        if !queue.is_empty() {
            queue.remove(0);
        }

        // Play next song if available
        if !queue.is_empty() {
            let next_url = queue[0].clone();
            drop(queue); // Release lock before async operation

            if let Ok((source, file_path)) = create_source(&self.guild_id.0, &next_url).await {
                // Mark new file as active
                if let Some(ref path) = file_path {
                    let mut active = self.active_files.lock().await;
                    active.entry(self.guild_id.0).or_default().insert(path.clone());
                }

                let mut call = self.call.lock().await;
                let handle = call.play_input(source);

                // Add track end handler for the next song
                let _ = handle.add_event(
                    songbird::Event::Track(TrackEvent::End),
                    TrackEndNotifier {
                        guild_id: self.guild_id,
                        call: self.call.clone(),
                        queue: self.queue.clone(),
                        active_files: self.active_files.clone(),
                        downloaded_file: file_path,
                    },
                );
            }
        }

        None
    }
}
