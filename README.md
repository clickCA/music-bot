# Discord Music Bot

A Discord music bot built with Rust that can play songs by name or YouTube URL.

## Features

- Play songs by searching with the song name
- Play songs directly from YouTube URLs
- Queue management (add multiple songs)
- Pause/Resume/Stop/Skip controls
- View the current queue
- Auto-join voice channels
- Auto-play next song in queue
- **Automatic cleanup of downloaded music files** - Prevents disk space issues
- **Concurrency-safe** - Multiple guilds can play simultaneously without conflicts
- **Graceful shutdown** - Handles Ctrl+C/SIGTERM with proper cleanup

## Quick Start (Docker)

**This is a Docker-first project.** The recommended way to run the bot is with Docker.

1. **Create a Discord Bot**:
   - Go to [Discord Developer Portal](https://discord.com/developers/applications)
   - Create a new application
   - Go to the "Bot" section and create a bot
   - Enable "MESSAGE CONTENT INTENT" and "SERVER MEMBERS INTENT"
   - Copy the bot token

2. **Configure the Bot**:
   ```bash
   cp .env.example .env
   nano .env  # Add your Discord bot token
   ```

3. **Run with Docker Compose**:
   ```bash
   docker compose up -d

   # View logs
   docker compose logs -f

   # Stop the bot
   docker compose down
   ```

## Prerequisites (Local Development)

If you want to run without Docker:

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **yt-dlp** - Install on your system:
   - Linux: `sudo apt install yt-dlp` or `pip install yt-dlp`
   - macOS: `brew install yt-dlp`
   - Windows: Download from [github.com/yt-dlp](https://github.com/yt-dlp/yt-dlp)
3. **FFmpeg** - Install on your system:
   - Linux: `sudo apt install ffmpeg`
   - macOS: `brew install ffmpeg`
   - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

4. **Run locally**:
   ```bash
   cargo run
   ```

## Setup (Local Development Details)

If running locally, follow these steps:

2. **Configure the Bot**:
   ```bash
   cp .env.example .env
   ```
   - Edit `.env` and paste your Discord bot token

3. **Invite the Bot to Your Server**:
   - Go to OAuth2 > URL Generator
   - Select scopes: `bot` and `applications.commands`
   - Select bot permissions: `Connect`, `Speak`, `Use Voice Activity`, `Send Messages`, `Read Messages`
   - Use the generated URL to invite the bot

## Docker Commands

- `docker compose up -d` - Start bot in background
- `docker compose logs -f` - Follow bot logs
- `docker compose down` - Stop and remove bot
- `docker compose restart` - Restart the bot
- `docker compose exec music-bot yt-dlp --update-to latest` - Update yt-dlp
- `docker compose down -v` - Stop and remove all volumes (including temp files)

## Commands

- `!join` - Join your voice channel
- `!play <song name or URL>` - Play a song by name or YouTube URL
- `!pause` - Pause the current song
- `!resume` - Resume playback
- `!skip` - Skip to the next song
- `!stop` - Stop and clear the queue
- `!queue` - Show the current queue
- `!leave` - Leave the voice channel
- `!shutdown` - Gracefully shut down the bot (admin only)

## Examples

```
!play never gonna give you up
!play https://www.youtube.com/watch?v=dQw4w9WgXcQ
!queue
!skip
```

## Troubleshooting

1. **"You need to be in a voice channel!"** - Join a voice channel first
2. **Song doesn't play** - Make sure yt-dlp and FFmpeg are installed
3. **Bot doesn't respond** - Check that MESSAGE CONTENT INTENT is enabled in Discord Developer Portal
4. **Audio quality issues** - Ensure your internet connection is stable and yt-dlp is up to date

## Docker Benefits

- ✅ **Isolated Environment** - No need to install Rust, yt-dlp, or FFmpeg locally
- ✅ **Consistent** - Same environment everywhere
- ✅ **Easy Updates** - Rebuild and restart with one command
- ✅ **Auto-restart** - Container restarts automatically if it crashes
- ✅ **Portable** - Works on any system with Docker

## Storage & Cleanup

The bot automatically manages downloaded music files to prevent disk space issues:

- **Per-Guild Isolation**: Each guild gets its own temp directory
- **Active File Tracking**: Only inactive files are deleted
- **Automatic Cleanup**: Downloaded files are deleted after each song finishes
- **Periodic Cleanup**: Runs every hour to catch any orphaned files
- **Startup Cleanup**: Cleans any remaining files from previous sessions
- **Concurrency-Safe**: Multiple guilds can play without conflicts

### Manual Cleanup Commands

**Docker:**
```bash
# Check temp files inside container
docker compose exec music-bot ls /tmp/music_bot_downloads

# Remove all temp volumes
docker compose down -v  # WARNING: This removes all volumes
```

**Local Development:**
```bash
# Check temp directories
ls /tmp/music_bot_downloads/

# Clean manually
rm -rf /tmp/music_bot_downloads/*
```

For more details on how concurrency and cleanup works, see [CONCURRENCY.md](CONCURRENCY.md).

## Graceful Shutdown

The bot handles shutdown signals gracefully:

### Local Development
```bash
# Press Ctrl+C - bot will:
# 1. Stop accepting new commands
# 2. Clear all queues
# 3. Clean up temp files
# 4. Disconnect from voice channels
# 5. Exit cleanly
```

### Docker
```bash
# Stop the bot gracefully
docker compose down

# Bot gets 30 seconds to clean up (configured in compose.yml)
# You'll see shutdown logs:
# 🛑 Received shutdown signal (Ctrl+C)...
# 🧹 Cleaning up...
# 📝 Cleared 3 queue(s)
# 🧹 Cleaned up all temp directories
# ✅ Shutdown complete. Goodbye!
```

### What Happens on Shutdown
1. **Stops accepting new commands** - Sets shutdown flag
2. **Clears all queues** - Removes pending songs
3. **Stops playback** - Disconnects from voice channels
4. **Cleans up temp files** - Deletes all downloaded music
5. **Exits cleanly** - Proper process termination

The bot has **30 seconds** to clean up before being force-killed (configurable with `stop_grace_period` in compose.yml).
