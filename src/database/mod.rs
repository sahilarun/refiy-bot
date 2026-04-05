use crate::error::BotError;
use crate::types::lavalink::player_saver::PlayerData;
use libsql::{params, Connection, Database as LibSqlDatabase, Builder};
use std::sync::Arc;
use serde_json::json;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    db: Arc<LibSqlDatabase>,
}

impl Database {
    pub async fn new(url: &str, authToken: &str) -> Result<Self, BotError> {
        let db = Builder::new_remote(url.to_string(), authToken.to_string())
            .build()
            .await
            .map_err(BotError::Database)?;
        let conn = db.connect().map_err(BotError::Database)?;
        let dbArc = Arc::new(db);
        
        conn.execute("CREATE TABLE IF NOT EXISTS guild (id TEXT PRIMARY KEY, locale TEXT, prefix TEXT, defaultVolume INTEGER, enabled247 INTEGER NOT NULL DEFAULT 0, autoplay INTEGER NOT NULL DEFAULT 0, channel247Id TEXT, text247Id TEXT, setupChannelId TEXT, setupTextId TEXT, voiceStatus INTEGER NOT NULL DEFAULT 1, createdAt TEXT DEFAULT (datetime('now')), updatedAt TEXT DEFAULT (datetime('now')))", ()).await?;
        
        let _ = conn.execute("ALTER TABLE guild ADD COLUMN enabled247 INTEGER NOT NULL DEFAULT 0", ()).await;
        let _ = conn.execute("ALTER TABLE guild ADD COLUMN channel247Id TEXT", ()).await;
        let _ = conn.execute("ALTER TABLE guild ADD COLUMN text247Id TEXT", ()).await;
        let _ = conn.execute("ALTER TABLE guild ADD COLUMN autoplay INTEGER NOT NULL DEFAULT 0", ()).await;
        
        conn.execute("CREATE TABLE IF NOT EXISTS liked_songs (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, track_id TEXT NOT NULL, title TEXT NOT NULL, author TEXT NOT NULL, uri TEXT NOT NULL, artwork TEXT, length INTEGER, is_stream INTEGER DEFAULT 0, liked_at TEXT DEFAULT (datetime('now')), UNIQUE(user_id, track_id))", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS playlist (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, name TEXT NOT NULL, created_at TEXT DEFAULT (datetime('now')))", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS playlist_track (id TEXT PRIMARY KEY, url TEXT NOT NULL, playlist_id TEXT NOT NULL REFERENCES playlist(id) ON DELETE CASCADE, info TEXT)", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS track_stats (id TEXT PRIMARY KEY, track_id TEXT NOT NULL, title TEXT NOT NULL, author TEXT NOT NULL, uri TEXT NOT NULL, artwork TEXT, length INTEGER, is_stream INTEGER DEFAULT 0, user_id TEXT NOT NULL, play_count INTEGER NOT NULL DEFAULT 1, guild_id TEXT NOT NULL, last_played TEXT DEFAULT (datetime('now')), created_at TEXT DEFAULT (datetime('now')), UNIQUE(track_id, guild_id))", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS user_stats (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, guild_id TEXT NOT NULL, play_count INTEGER NOT NULL DEFAULT 1, last_played TEXT DEFAULT (datetime('now')), created_at TEXT DEFAULT (datetime('now')), UNIQUE(user_id, guild_id))", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS user_vote (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, voted_at TEXT DEFAULT (datetime('now')), expires_at TEXT NOT NULL, type TEXT NOT NULL DEFAULT 'vote')", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS player_queue (id TEXT PRIMARY KEY, guild_id TEXT NOT NULL, track_data TEXT NOT NULL, position INTEGER NOT NULL, requester_id TEXT, title TEXT, author TEXT, uri TEXT, duration INTEGER, is_stream INTEGER, artwork_url TEXT)", ()).await?;
        conn.execute("CREATE TABLE IF NOT EXISTS player_sessions (guild_id TEXT PRIMARY KEY, data TEXT NOT NULL, updated_at TEXT DEFAULT (datetime('now')))", ()).await?;
        
        Ok(Self { db: dbArc })
    }

    fn conn(&self) -> Result<Connection, BotError> {
        self.db.connect().map_err(BotError::Database)
    }

    pub async fn getPrefix(&self, guildId: &str) -> Result<String, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT prefix FROM guild WHERE id = ?1", params![guildId]).await?;
        if let Some(row) = rows.next().await? {
            if let Ok(prefix) = row.get::<String>(0) {
                return Ok(prefix);
            }
        }
        Ok(".".into())
    }

    pub async fn setPrefix(&self, guildId: &str, prefix: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO guild (id, prefix) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET prefix = ?2",
            params![guildId, prefix],
        ).await?;
        Ok(())
    }

    pub async fn deletePrefix(&self, guildId: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO guild (id) VALUES (?1) ON CONFLICT(id) DO UPDATE SET prefix = NULL",
            params![guildId],
        ).await?;
        Ok(())
    }

    pub async fn getLocale(&self, guildId: &str) -> Result<String, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT locale FROM guild WHERE id = ?1", params![guildId]).await?;
        if let Some(row) = rows.next().await? {
            if let Ok(locale) = row.get::<String>(0) { return Ok(locale); }
        }
        Ok("en-US".into())
    }

    pub async fn setLocale(&self, guildId: &str, locale: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO guild (id, locale) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET locale = ?2",
            params![guildId, locale],
        ).await?;
        Ok(())
    }

    pub async fn getDefaultVolume(&self, guildId: &str) -> Result<u16, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT defaultVolume FROM guild WHERE id = ?1", params![guildId]).await?;
        if let Some(row) = rows.next().await? {
            if let Ok(vol) = row.get::<i64>(0) { return Ok(vol as u16); }
        }
        Ok(100)
    }

    pub async fn get247Mode(&self, guildId: &str) -> Result<Option<(bool, Option<String>, Option<String>)>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query(
            "SELECT enabled247, channel247Id, text247Id FROM guild WHERE id = ?1",
            params![guildId],
        ).await?;
        if let Some(row) = rows.next().await? {
            let enabled = row.get::<i64>(0).unwrap_or(0) != 0;
            let channelId = row.get::<String>(1).ok();
            let textId = row.get::<String>(2).ok();
            return Ok(Some((enabled, channelId, textId)));
        }
        Ok(None)
    }

    pub async fn set247Mode(&self, guildId: &str, enabled: bool, channelId: Option<&str>, textId: Option<&str>) -> Result<(), BotError> {
        let conn = self.conn()?;
        let enabledI = if enabled { 1i64 } else { 0i64 };
        conn.execute(
            "INSERT INTO guild (id, enabled247, channel247Id, text247Id) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET enabled247 = ?2, channel247Id = ?3, text247Id = ?4",
            params![guildId, enabledI, channelId, textId],
        ).await?;
        Ok(())
    }

    pub async fn getAutoplay(&self, guildId: &str) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT autoplay FROM guild WHERE id = ?1", params![guildId]).await?;
        if let Some(row) = rows.next().await? {
            return Ok(row.get::<i64>(0).unwrap_or(0) != 0);
        }
        Ok(false)
    }

    pub async fn setAutoplay(&self, guildId: &str, enabled: bool) -> Result<(), BotError> {
        let conn = self.conn()?;
        let enabledI = if enabled { 1i64 } else { 0i64 };
        conn.execute(
            "INSERT INTO guild (id, autoplay) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET autoplay = ?2",
            params![guildId, enabledI],
        ).await?;
        Ok(())
    }

    pub async fn update_track_stats(
        &self, track_id: &str, title: &str, author: &str, guild_id: &str, 
        user_id: &str, uri: &str, artwork: Option<&str>, length: Option<i64>, 
        is_stream: bool
    ) -> Result<(), BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        let is_stream_i = if is_stream { 1i64 } else { 0i64 };
        conn.execute(
            "INSERT INTO track_stats (id, track_id, title, author, uri, artwork, length, is_stream, user_id, guild_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, track_id, title, author, uri, artwork, length, is_stream_i, user_id, guild_id],
        ).await?;
        Ok(())
    }

    pub async fn update_user_stats(&self, user_id: &str, guild_id: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO user_stats (id, user_id, guild_id) VALUES (?1, ?2, ?3) ON CONFLICT(user_id, guild_id) DO UPDATE SET play_count = play_count + 1, last_played = datetime('now')",
            params![id, user_id, guild_id],
        ).await?;
        Ok(())
    }

    pub async fn has_voted(&self, userId: &str) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut rows = conn.query("SELECT id FROM user_vote WHERE user_id = ?1 AND expires_at > ?2", params![userId, now]).await?;
        Ok(rows.next().await?.is_some())
    }

    pub async fn cleanup_votes(&self) -> Result<(), BotError> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute("DELETE FROM user_vote WHERE expires_at <= ?1", params![now]).await?;
        Ok(())
    }

    pub async fn clear_queue(&self, guildId: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM player_queue WHERE guild_id = ?1", params![guildId]).await?;
        Ok(())
    }

    pub async fn getLikedSongs(&self, userId: &str) -> Result<Vec<serde_json::Value>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT * FROM liked_songs WHERE user_id = ?1 ORDER BY liked_at DESC", params![userId]).await?;
        let mut songs = Vec::new();
        while let Some(row) = rows.next().await? {
            songs.push(json!({
                "id": row.get::<String>(0)?,
                "userId": row.get::<String>(1)?,
                "trackId": row.get::<String>(2)?,
                "title": row.get::<String>(3)?,
                "author": row.get::<String>(4)?,
                "uri": row.get::<String>(5)?,
                "artwork": row.get::<String>(6).ok(),
                "length": row.get::<i64>(7).ok(),
                "isStream": row.get::<i64>(8).unwrap_or(0) != 0,
                "likedAt": row.get::<String>(9)?,
            }));
        }
        Ok(songs)
    }

    pub async fn addToLikedSongs(&self, userId: &str, trackId: &str, title: &str, author: &str, uri: &str, artwork: Option<String>, length: u64, isStream: bool) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        let isStreamI = if isStream { 1i64 } else { 0i64 };
        conn.execute(
            "INSERT INTO liked_songs (id, user_id, track_id, title, author, uri, artwork, length, is_stream) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, userId, trackId, title, author, uri, artwork, length as i64, isStreamI],
        ).await?;
        Ok(true)
    }

    pub async fn isTrackLiked(&self, userId: &str, trackId: &str) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT id FROM liked_songs WHERE user_id = ?1 AND track_id = ?2", params![userId, trackId]).await?;
        Ok(rows.next().await?.is_some())
    }

    pub async fn removeFromLikedSongs(&self, userId: &str, id: &str) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let rows = conn.execute("DELETE FROM liked_songs WHERE user_id = ?1 AND (id = ?2 OR track_id = ?2)", params![userId, id]).await?;
        Ok(rows > 0)
    }

    pub async fn getRecentlyPlayed(&self, userId: &str, guildId: Option<&str>, limit: usize) -> Result<Vec<serde_json::Value>, BotError> {
        let conn = self.conn()?;
        let query = if let Some(gid) = guildId {
            conn.query("SELECT * FROM track_stats WHERE user_id = ?1 AND guild_id = ?2 ORDER BY last_played DESC LIMIT ?3", params![userId, gid, limit as i64]).await?
        } else {
            conn.query("SELECT * FROM track_stats WHERE user_id = ?1 ORDER BY last_played DESC LIMIT ?2", params![userId, limit as i64]).await?
        };
        
        let mut rows = query;
        let mut tracks = Vec::new();
        while let Some(row) = rows.next().await? {
            tracks.push(json!({
                "trackId": row.get::<String>(1)?,
                "title": row.get::<String>(2)?,
                "author": row.get::<String>(3)?,
                "uri": row.get::<String>(4)?,
                "artwork": row.get::<String>(5).ok(),
                "length": row.get::<i64>(6).ok(),
                "isStream": row.get::<i64>(7).unwrap_or(0) != 0,
                "userId": row.get::<String>(8)?,
                "playCount": row.get::<i64>(9)?,
                "guildId": row.get::<String>(10)?,
                "lastPlayed": row.get::<String>(11)?,
            }));
        }
        Ok(tracks)
    }

    pub async fn savePlayerData(&self, guildId: &str, data: &PlayerData) -> Result<(), BotError> {
        let conn = self.conn()?;
        let dataStr = serde_json::to_string(data).map_err(|e| BotError::Other(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO player_sessions (guild_id, data, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(guild_id) DO UPDATE SET data = ?2, updated_at = ?3",
            params![guildId, dataStr, now],
        ).await?;
        Ok(())
    }

    pub async fn removePlayerData(&self, guildId: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM player_sessions WHERE guild_id = ?1", params![guildId]).await?;
        Ok(())
    }

    pub async fn getPlayerData(&self, guildId: &str) -> Result<Option<PlayerData>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT data FROM player_sessions WHERE guild_id = ?1", params![guildId]).await?;
        if let Some(row) = rows.next().await? {
            let dataStr: String = row.get(0)?;
            let data: PlayerData = serde_json::from_str(&dataStr).map_err(|e| BotError::Other(e.to_string()))?;
            return Ok(Some(data));
        }
        Ok(None)
    }

    pub async fn getAllPlayerData(&self) -> Result<Vec<PlayerData>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT data FROM player_sessions", ()).await?;
        let mut allData = Vec::new();
        while let Some(row) = rows.next().await? {
            let dataStr: String = row.get(0)?;
            if let Ok(data) = serde_json::from_str(&dataStr) {
                allData.push(data);
            }
        }
        Ok(allData)
    }

    pub async fn clearAllPlayerData(&self) -> Result<(), BotError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM player_sessions", ()).await?;
        Ok(())
    }

    pub async fn createPlaylist(&self, userId: &str, name: &str) -> Result<String, BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO playlist (id, user_id, name) VALUES (?1, ?2, ?3)",
            params![id.clone(), userId, name],
        ).await?;
        Ok(id)
    }

    pub async fn deletePlaylist(&self, userId: &str, name: &str) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let rows = conn.execute(
            "DELETE FROM playlist WHERE user_id = ?1 AND name = ?2",
            params![userId, name],
        ).await?;
        Ok(rows > 0)
    }

    pub async fn getPlaylists(&self, userId: &str) -> Result<Vec<crate::types::core::database::Playlist>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT * FROM playlist WHERE user_id = ?1", params![userId]).await?;
        let mut playlists = Vec::new();
        while let Some(row) = rows.next().await? {
            playlists.push(crate::types::core::database::Playlist {
                id: row.get::<String>(0)?,
                userId: row.get::<String>(1)?,
                name: row.get::<String>(2)?,
                createdAt: row.get::<String>(3)?,
            });
        }
        Ok(playlists)
    }

    pub async fn getPlaylistId(&self, userId: &str, name: &str) -> Result<Option<String>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT id FROM playlist WHERE user_id = ?1 AND name = ?2", params![userId, name]).await?;
        if let Some(row) = rows.next().await? {
            return Ok(Some(row.get(0)?));
        }
        Ok(None)
    }

    pub async fn addToPlaylist(&self, playlistId: &str, url: &str, info: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO playlist_track (id, url, playlist_id, info) VALUES (?1, ?2, ?3, ?4)",
            params![id, url, playlistId, info],
        ).await?;
        Ok(())
    }

    pub async fn removeFromPlaylist(&self, playlistId: &str, index: usize) -> Result<bool, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT id FROM playlist_track WHERE playlist_id = ?1", params![playlistId]).await?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            ids.push(row.get::<String>(0)?);
        }
        if index < ids.len() {
            conn.execute("DELETE FROM playlist_track WHERE id = ?1", params![ids[index].clone()]).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn getPlaylistTracks(&self, playlistId: &str) -> Result<Vec<crate::types::core::database::PlaylistTrack>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query("SELECT * FROM playlist_track WHERE playlist_id = ?1", params![playlistId]).await?;
        let mut tracks = Vec::new();
        while let Some(row) = rows.next().await? {
            tracks.push(crate::types::core::database::PlaylistTrack {
                id: row.get::<String>(0)?,
                url: row.get::<String>(1)?,
                playlistId: row.get::<String>(2)?,
                info: row.get::<String>(3)?,
            });
        }
        Ok(tracks)
    }

    pub async fn getLeaderboard(&self, guildId: &str) -> Result<Vec<crate::types::core::database::UserStats>, BotError> {
        let conn = self.conn()?;
        let mut rows = conn.query(
            "SELECT user_id, guild_id, play_count FROM user_stats WHERE guild_id = ?1 ORDER BY play_count DESC LIMIT 10",
            params![guildId],
        ).await?;
        let mut leaderboard = Vec::new();
        while let Some(row) = rows.next().await? {
            leaderboard.push(crate::types::core::database::UserStats {
                userId: row.get::<String>(0)?,
                guildId: row.get::<String>(1)?,
                playCount: row.get::<i64>(2)?,
            });
        }
        Ok(leaderboard)
    }

    pub async fn incrementUserStats(&self, userId: &str, guildId: &str) -> Result<(), BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO user_stats (id, user_id, guild_id, play_count) VALUES (?1, ?2, ?3, 1) ON CONFLICT(user_id, guild_id) DO UPDATE SET play_count = play_count + 1, last_played = datetime('now')",
            params![id, userId, guildId],
        ).await?;
        Ok(())
    }

    pub async fn incrementTrackStats(
        &self,
        trackId: &str,
        title: &str,
        author: &str,
        guildId: &str,
        userId: &str,
        uri: &str,
        artwork: Option<&str>,
        length: Option<i64>,
        isStream: bool,
    ) -> Result<(), BotError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO track_stats (id, track_id, title, author, uri, artwork, length, is_stream, user_id, guild_id, play_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1) ON CONFLICT(track_id, guild_id) DO UPDATE SET play_count = play_count + 1, last_played = datetime('now')",
            params![id, trackId, title, author, uri, artwork, length, isStream as i64, userId, guildId],
        ).await?;
        Ok(())
    }
}
