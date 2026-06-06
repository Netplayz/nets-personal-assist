# Nets Personal Assist

A high-performance Discord moderation and utility bot written in Rust, built for Roblox community servers.

## Features

| Category | Commands |
|---|---|
| **Time/Date** | `/time [timezone]`, `/timezones [query]` |
| **Moderation** | `/mute <member> <minutes> [reason]`, `/unmute <member>`, `/memberinfo [member]`, `/chatsummary [count]` |
| **Player Stats** | `/playtime <roblox_id>`, `/infractions <roblox_id>` |
| **Identity** | `/roblox <id_or_username>`, `/discord <user>` |
| **Network** | `/ping <host>`, `/portscan <host> [mode] [start] [end]`, `/ipintel <ip>` |
| **System** | `/sysinfo` |

## Performance

- In-memory response caching (moka)
- SQLite WAL mode with optimized PRAGMAs
- Reqwest connection pooling with gzip compression
- Concurrent port scanning with tokio async TCP
- Roblox API rate-limit semaphore (10 concurrent max)
- Full LTO, 1 codegen unit in release builds

## Requirements

- Rust 1.75+
- Discord bot token ([Discord Developer Portal](https://discord.com/developers/applications))

### Optional Tokens
- `IPINFO_TOKEN` — for detailed IP geolocation
- `ABUSEIPDB_TOKEN` — for IP abuse scoring
- `ROBLOX_COOKIE` — for authenticated Roblox API calls

## Quick Start

```bash
cp .env.example .env
# Edit .env with your Discord token
cargo run --release
```

Slash commands are registered globally on first run (may take a few minutes to appear).

## Data

Persistent data (playtime, infractions, mutes) is stored in SQLite at the path specified by `DATABASE_URL` (default: `data.db`).
