#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    pub guildId: String,
    pub voiceChannelId: Option<String>,
    pub textChannelId: Option<String>,
    pub messageId: Option<String>,
    pub nodeId: Option<String>,
    pub nodeSessionId: Option<String>,
    pub volume: Option<u16>,
    pub repeatMode: Option<String>,
    pub enabledAutoplay: Option<bool>,
    pub lyricsEnabled: Option<bool>,
    pub lyricsId: Option<String>,
    pub lyricsRequester: Option<String>,
    pub localeString: Option<String>,
    pub lyrics: Option<LyricsData>,
    pub track: Option<QueueTrack>,
    pub queue: Option<Vec<QueueTrack>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsData {
    pub provider: Option<String>,
    pub text: Option<String>,
    pub lines: Option<Vec<LyricsLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsLine {
    pub line: String,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueTrack {
    pub encoded: Option<String>,
    pub info: Option<QueueTrackInfo>,
    pub requester: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueTrackInfo {
    pub title: Option<String>,
    pub uri: Option<String>,
    pub author: Option<String>,
    pub duration: Option<u64>,
    pub identifier: Option<String>,
    pub isStream: Option<bool>,
    pub isSeekable: Option<bool>,
    pub sourceName: Option<String>,
    pub artworkUrl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingMessage {
    pub messageId: Option<String>,
    pub channelId: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSchema {
    pub players: HashMap<String, PlayerData>,
    pub sessions: HashMap<String, String>,
}
