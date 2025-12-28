# Docker Quick Reference

## Basic Commands

### Start the bot
```bash
docker compose up -d
```

### View logs
```bash
# Follow logs in real-time
docker compose logs -f

# View last 100 lines
docker compose logs --tail 100
```

### Stop the bot
```bash
# Graceful shutdown (recommended)
docker compose down

# Force stop immediately (not recommended)
docker compose kill

# Stop and remove everything (including temp files)
docker compose down -v
```

### Restart the bot
```bash
docker compose restart
```

### Update the bot
```bash
# Rebuild and restart
docker compose up -d --build

# Pull latest changes (if using git)
git pull
docker compose up -d --build
```

## Advanced Commands

### Check container status
```bash
docker compose ps
```

### Execute commands inside container
```bash
# Open a shell
docker compose exec music-bot bash

# Update yt-dlp
docker compose exec music-bot yt-dlp --update-to latest

# Check temp files
docker compose exec music-bot ls -la /tmp/music_bot_downloads

# Check disk usage
docker compose exec music-bot du -sh /tmp/music_bot_downloads/*
```

### Resource monitoring
```bash
# CPU and memory usage
docker stats music-bot

# Container info
docker inspect music-bot
```

## Volume Management

### List volumes
```bash
docker volume ls | grep music
```

### Inspect temp volume
```bash
docker volume inspect music_bot-temp
```

### Clean up temp files (without stopping bot)
```bash
# Option 1: Remove all guild directories
docker compose exec music-bot rm -rf /tmp/music_bot_downloads/guild_*

# Option 2: Stop, remove volume, restart
docker compose down -v
docker compose up -d
```

## Graceful Shutdown

The bot handles shutdown gracefully:

```bash
# Stop with cleanup
docker compose down

# The bot will:
# 1. Stop accepting new commands
# 2. Clear all queues
# 3. Disconnect from voice channels
# 4. Clean up temp files
# 5. Exit cleanly

# Shutdown logs:
# 🛑 Received shutdown signal (Ctrl+C)...
# 🧹 Cleaning up...
# 📝 Cleared 3 queue(s)
# 🧹 Cleaned up all temp directories
# ✅ Shutdown complete. Goodbye!
```

The bot has **30 seconds** to clean up (configurable in compose.yml with `stop_grace_period`).

## Troubleshooting

### Bot won't start
```bash
# Check logs
docker compose logs

# Verify token is set
docker compose exec music-bot printenv | grep DISCORD_TOKEN
```

### Container keeps restarting
```bash
# Check recent logs
docker compose logs --tail 50

# Inspect the container
docker inspect music-bot
```

### Audio not playing
```bash
# Verify ffmpeg and yt-dlp are installed
docker compose exec music-bot which ffmpeg
docker compose exec music-bot which yt-dlp

# Test yt-dlp
docker compose exec music-bot yt-dlp --version
```

### Disk space issues
```bash
# Check temp directory size
docker compose exec music-bot du -sh /tmp/music_bot_downloads

# Clean up completely
docker compose down -v
docker compose up -d
```

### Container stuck during shutdown
```bash
# Force kill if cleanup hangs
docker compose kill

# Or reduce timeout
# Edit compose.yml: stop_grace_period: 10s
```

## Production Tips

1. **Auto-restart**: The container is configured with `restart: unless-stopped`
2. **Health checks**: Built-in healthcheck monitors bot status
3. **Non-root user**: Bot runs as `musicbot` user for security
4. **Volume isolation**: Temp files stored in separate volume
5. **Graceful shutdown**: 30-second grace period for cleanup
6. **Signal handling**: Properly handles SIGTERM for clean shutdown

## Backup & Restore

### Backup configuration
```bash
# Copy .env file
cp .env .env.backup

# Export compose config
docker compose config > compose-config.yml
```

### Restore
```bash
# Restore .env
cp .env.backup .env
docker compose up -d
```

## Monitoring

### Real-time monitoring
```bash
# Logs + resource usage
watch -n 1 'docker stats music-bot && echo "---" && docker compose logs --tail 10'
```

### Check bot connectivity
```bash
# The bot should appear in your Discord server
# Use !queue command to test responsiveness

# Check health status
docker inspect music-bot | grep -A 10 Health
```

## Signal Handling

The bot properly handles these signals:
- **SIGTERM (15)**: Graceful shutdown with cleanup (default from `docker compose down`)
- **SIGINT (2)**: Same as SIGTERM (Ctrl+C)
- **SIGKILL (9)**: Immediate termination (use `docker compose kill`)

Always prefer `docker compose down` over `docker compose kill` to allow proper cleanup.
