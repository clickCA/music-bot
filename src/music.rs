use songbird::input::Input;
use std::path::PathBuf;
use tokio::task::spawn_blocking;

use crate::cleanup::get_guild_temp_dir;

/// Create an audio source from YouTube (URL or search query)
/// Returns the Input source and the path to the downloaded file (if any)
pub async fn create_source(
    guild_id: &u64,
    query: &str,
) -> Result<(Input, Option<PathBuf>), Box<dyn std::error::Error + Send + Sync>> {
    // Ensure guild temp directory exists
    let temp_dir = get_guild_temp_dir(*guild_id);
    if !temp_dir.exists() {
        spawn_blocking(move || std::fs::create_dir_all(&temp_dir))
            .await
            .ok();
    }

    // If it's a URL, use it directly. Otherwise, search YouTube
    let search_query = if query.starts_with("http") {
        query.to_string()
    } else {
        format!("ytsearch1:{}", query)
    };

    // Configure YoutubeDl to use guild-specific temp directory
    use songbird::input::YoutubeDl;
    let source = YoutubeDl::new(reqwest::Client::new(), search_query)
        .cache_to(Some(temp_dir.clone()))
        .await?;

    // Try to detect the downloaded file path (if any)
    let downloaded_file = spawn_blocking(move || {
        // After yt-dlp downloads, check for new files
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            let files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            if !files.is_empty() {
                // Return the most recently modified file
                let mut latest = None;
                let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

                for file in files {
                    if let Ok(meta) = std::fs::metadata(&file) {
                        if let Ok(modified) = meta.modified() {
                            if modified > latest_time {
                                latest_time = modified;
                                latest = Some(file);
                            }
                        }
                    }
                }
                return latest;
            }
        }
        None
    })
    .await
    .ok()
    .flatten();

    Ok((source.into(), downloaded_file))
}
