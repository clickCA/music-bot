# Concurrency & Cleanup Strategy

## Problem Solved

The original implementation had several **race conditions** when handling multiple users/guilds:

1. **Global cleanup** - All guilds shared `/tmp/music_bot_downloads`, causing cleanup in one guild to delete files actively playing in another
2. **No file tracking** - No way to know which files were currently being played
3. **File collisions** - Multiple users downloading the same song could overwrite each other

## Solution Implemented

### 1. Per-Guild Isolation

```
/tmp/music_bot_downloads/
├── guild_123456789/
│   └── song_audio.webm
├── guild_987654321/
│   └── another_song.webm
└── guild_111222333/
    └── cool_track.webm
```

Each guild gets its own subdirectory, preventing cross-guild interference.

### 2. Active File Tracking

```rust
type ActiveFiles = Arc<Mutex<HashMap<u64, HashSet<PathBuf>>>>;
```

- **Track** which files are currently playing for each guild
- **Prevent** cleanup from deleting active files
- **Thread-safe** with Mutex protection

### 3. Safe Cleanup Logic

```rust
// src/cleanup.rs:cleanup_guild_temp_files()

// Only delete files NOT in the active set
if path.is_file() && !active_set.contains(&path) {
    if std::fs::remove_file(&path).is_ok() {
        cleaned += 1;
    }
}
```

### 4. File Lifecycle

```
Download → Mark Active → Playing → Track End
                              ↓
                         Remove from Active
                              ↓
                         Cleanup (safe to delete)
```

## Concurrency Guarantees

### ✅ Multiple Users in Same Guild
- Queue ensures sequential playback
- Only one song plays at a time
- Next song starts after previous ends

### ✅ Multiple Guilds Simultaneously
- Each guild has isolated temp directory
- Active files tracked per-guild
- Cleanup never deletes active files
- No cross-guild interference

### ✅ Cleanup Race Conditions Fixed
- **Before**: Any cleanup could delete any file
- **After**: Only inactive files deleted, per-guild

### ✅ File Naming
- yt-dlp handles unique naming
- Each guild's files in separate directory
- No collisions possible

## Module Structure

```
src/
├── main.rs      - Bot commands and event handling
├── cleanup.rs   - Safe cleanup logic per-guild
├── music.rs     - YouTube download and source creation
├── events.rs    - Track end events and auto-play
└── queue.rs     - Queue type definitions
```

## Testing Scenarios

### Scenario 1: Two users play simultaneously in different guilds
```
Guild A: User 1 plays "Song A" → /tmp/music_bot_downloads/guild_A/song_A.webm
Guild B: User 2 plays "Song B" → /tmp/music_bot_downloads/guild_B/song_B.webm

Both play simultaneously without interference ✓
```

### Scenario 2: User queues multiple songs
```
Guild A: User plays "Song 1", "Song 2", "Song 3"
1. Song 1 downloads → marked active
2. Song 1 finishes → removed from active → cleaned up
3. Song 2 downloads → marked active
4. Song 2 finishes → removed from active → cleaned up
...

No disk space buildup ✓
```

### Scenario 3: Cleanup runs while song is playing
```
Guild A: "Song A" is playing (active)
Periodic cleanup runs → sees Song A in active set → skips deletion ✓
Song A finishes → removed from active
Next cleanup → deletes Song A safely ✓
```

## Performance Considerations

- **Mutex locks** are short-lived (only for cleanup checks)
- **Per-guild isolation** means less lock contention
- **Periodic cleanup** every hour is lightweight
- **File detection** happens once per song download

## Monitoring

Logs show cleanup activity:
```
🧹 Guild 123456789: Cleaned up 2 old file(s)
🧹 Cleaned up all temp directories
```

## Docker Volume

The temp directory is mounted as a separate volume in `compose.yml`:
```yaml
volumes:
  bot-temp:
    driver: local
```

This can be cleaned independently:
```bash
docker compose down -v  # Removes all volumes including temp
```
