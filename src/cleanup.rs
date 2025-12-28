use std::{collections::HashMap, collections::HashSet, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

// Track active files to prevent race conditions
pub type ActiveFiles = Arc<Mutex<HashMap<u64, HashSet<PathBuf>>>>;

/// Get per-guild temp directory
pub fn get_guild_temp_dir(guild_id: u64) -> PathBuf {
    PathBuf::from(format!("/tmp/music_bot_downloads/guild_{}", guild_id))
}

/// Cleanup old files for a specific guild (only files not currently playing)
pub async fn cleanup_guild_temp_files(guild_id: u64, active_files: &ActiveFiles) {
    let temp_dir = get_guild_temp_dir(guild_id);
    if !temp_dir.exists() {
        return;
    }

    // Get currently active files for this guild
    let active_set = {
        let active = active_files.lock().await;
        active.get(&guild_id).cloned().unwrap_or(HashSet::new())
    };

    let cleanup_result = spawn_blocking(move || {
        let mut cleaned = 0;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Only delete files that are NOT currently being played
                if path.is_file() && !active_set.contains(&path) {
                    if std::fs::remove_file(&path).is_ok() {
                        cleaned += 1;
                    }
                }
            }
        }
        cleaned
    })
    .await;

    match cleanup_result {
        Ok(count) if count > 0 => println!("🧹 Guild {}: Cleaned up {} old file(s)", guild_id, count),
        _ => {}
    }
}

/// Clean up the entire temp directory (for startup/shutdown)
pub async fn cleanup_all_temp_files() {
    let temp_base = PathBuf::from("/tmp/music_bot_downloads");
    if !temp_base.exists() {
        return;
    }

    let _ = spawn_blocking(move || {
        if let Ok(entries) = std::fs::read_dir(&temp_base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(path);
                }
            }
        }
    })
    .await;

    println!("🧹 Cleaned up all temp directories");
}
