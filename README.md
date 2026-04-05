# Reify Bot 🎵

Reify is a high-performance Discord music bot built with **Rust**, **Poise**, and **Serenity**. It utilizes the **Lavalink** music server to provide high-quality audio streaming with a modern, interaction-driven user experience.

## ✨ Features

- **Modern UI (V2 Components)**: Utilizes Discord's latest Components V2 for rich, interactive music controls.
- **Slash & Prefix Commands**: Fully supports both slash commands and traditional prefix commands.
- **Interaction-Aware Response System**: Optimized to handle Discord's 3-second interaction window with early deferral and reliable follow-up patches.
- **Music Controls**: Play, stop, skip, pause, resume, volume control, lyrics, and more.
- **Playlist Management**: Create and manage your own music playlists.
- **High Performance**: Built on top of the Rust ecosystem for speed and stability.

## 🚀 Getting Started

### Prerequisites

- **Rust**: Ensure you have the latest stable Rust toolchain installed.
- **Lavalink Server**: A running instance of Lavalink.
- **Redis**: Required for persistent state management and caching.
- **Database**: PostgreSQL (Prisma-compatible handled via `src/database/`).

### Configuration

Create a `.env` file in the root directory and add your credentials:

```env
DISCORD_TOKEN=your_bot_token
LAVALINK_URI=your_lavalink_uri
LAVALINK_PASSWORD=your_lavalink_password
REDIS_URL=redis://127.0.0.1:6379
DATABASE_URL=postgresql://user:password@localhost:5432/reify
```

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/sahilarun/refiy-bot.git
   cd refiy-bot
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the bot:
   ```bash
   cargo run
   ```

## 📜 Commands

- `/play <query>`: Play a track or playlist from URL or search term.
- `/join`: Connect the bot to your current voice channel.
- `/stop`: Stop music and clear the queue.
- `/skip`: Skip the current track.
- `/volume <level>`: Adjust the volume level.
- `/lyrics`: Fetch lyrics for the currently playing track.
- `/playlist`: Manage your personal music collections.

## 🛠️ Built With

- [Poise](https://github.com/serenity-rs/poise) - Framework for high-level Discord bot commands.
- [Serenity](https://github.com/serenity-rs/serenity) - Low-level Discord API wrapper.
- [Lavalink-rs](https://github.com/n640/lavalink-rs) - Client for the Lavalink node.
- [Serde](https://serde.rs/) - Serialization and deserialization.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
