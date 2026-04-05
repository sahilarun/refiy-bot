#![allow(non_snake_case)]
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use dashmap::DashMap;
use poise::serenity_prelude::GuildId;

use crate::config::Config;
use crate::database::Database;
use crate::error::BotError;

#[derive(Clone)]
pub struct Data {
    pub lavalink: lavalink_rs::client::LavalinkClient,
    pub config: Arc<Config>,
    pub database: Database,
    pub redis: redis::aio::ConnectionManager,
    pub guildPlayers: Arc<DashMap<GuildId, GuildPlayer>>,
    pub cooldowns: Arc<DashMap<String, u64>>,
    pub botUserId: u64,
    pub startTime: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildPlayer {
    pub queue: VecDeque<QueuedTrack>,
    pub currentTrack: Option<QueuedTrack>,
    pub volume: u16,
    pub paused: bool,
    pub repeatMode: RepeatMode,
    pub autoplay: bool,
    pub is247: bool,
    pub mode247: bool,
    pub lyricsEnabled: bool,
    pub lyricsId: Option<u64>,
    pub lyricsRequester: Option<u64>,
    pub lastTrackId: Option<String>,
    pub textChannelId: Option<u64>,
    pub voiceChannelId: Option<u64>,
    pub nowPlayingMessageId: Option<u64>,
    pub voiceEndpoint: Option<String>,
    pub voiceToken: Option<String>,
    pub voiceSessionId: Option<String>,
    pub previousTracks: VecDeque<QueuedTrack>,
    pub lastActiveAt: Option<u64>,
    pub aloneSince: Option<u64>,
}

impl std::fmt::Debug for GuildPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuildPlayer")
            .field("queue", &self.queue)
            .field("currentTrack", &self.currentTrack)
            .field("volume", &self.volume)
            .field("paused", &self.paused)
            .field("repeatMode", &self.repeatMode)
            .field("autoplay", &self.autoplay)
            .field("mode247", &self.mode247)
            .field("lyricsEnabled", &self.lyricsEnabled)
            .field("lyricsId", &self.lyricsId)
            .field("lyricsRequester", &self.lyricsRequester)
            .field("lastTrackId", &self.lastTrackId)
            .field("textChannelId", &self.textChannelId)
            .field("voiceChannelId", &self.voiceChannelId)
            .field("nowPlayingMessageId", &self.nowPlayingMessageId)
            .field("voiceEndpoint", &self.voiceEndpoint)
            .field("voiceToken", &self.voiceToken)
            .field("voiceSessionId", &self.voiceSessionId)
            .field("previousTracks", &self.previousTracks)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueuedTrack {
    pub track: lavalink_rs::model::track::TrackData,
    pub requester: serenity::model::id::UserId,
    pub title: String,
    pub author: String,
    pub uri: String,
    pub duration: u64,
    pub isStream: bool,
    pub artworkUrl: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    Track,
    Queue,
}

impl std::fmt::Display for RepeatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepeatMode::Off => write!(f, "Off"),
            RepeatMode::Track => write!(f, "Track"),
            RepeatMode::Queue => write!(f, "Queue"),
        }
    }
}

pub type Context<'a> = poise::Context<'a, Data, BotError>;

impl GuildPlayer {
    pub fn new(volume: u16) -> Self {
        Self {
            queue: VecDeque::new(),
            currentTrack: None,
            volume,
            paused: false,
            repeatMode: RepeatMode::Off,
            autoplay: false,
            is247: false,
            mode247: false,
            lyricsEnabled: false,
            lyricsId: None,
            lyricsRequester: None,
            lastTrackId: None,
            textChannelId: None,
            voiceChannelId: None,
            nowPlayingMessageId: None,
            voiceEndpoint: None,
            voiceToken: None,
            voiceSessionId: None,
            previousTracks: VecDeque::new(),
            lastActiveAt: None,
            aloneSince: None,
        }
    }
}
