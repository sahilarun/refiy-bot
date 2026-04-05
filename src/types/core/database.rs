#![allow(non_snake_case)]

#[derive(Debug, Clone)]
pub struct UserPremiumStats {
    pub active: bool,
    pub expiresAt: Option<String>,
    pub premiumType: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GlobalPremiumStats {
    pub activeRegularUsers: i64,
    pub activeVoteUsers: i64,
    pub totalActiveUsers: i64,
}

#[derive(Debug, Clone)]
pub struct PremiumStatus {
    pub premiumType: String,
    pub timeRemaining: i64,
}

#[derive(Debug, Clone)]
pub struct SetupInfo {
    pub id: String,
    pub guildId: String,
    pub channelId: String,
    pub messageId: String,
    pub createdAt: String,
}

#[derive(Debug, Clone)]
pub struct GuildPlayerSettings {
    pub defaultVolume: u16,
}
#[derive(Debug, Clone)]
pub struct PlaylistStats {
    pub count: i64,
    pub tracks: i64,
}

#[derive(Debug, Clone)]
pub struct GeneralStats {
    pub trackStats: i64,
    pub userStats: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub id: String,
    pub userId: String,
    pub name: String,
    pub createdAt: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaylistTrack {
    pub id: String,
    pub url: String,
    pub playlistId: String,
    pub info: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserStats {
    pub userId: String,
    pub guildId: String,
    pub playCount: i64,
}
