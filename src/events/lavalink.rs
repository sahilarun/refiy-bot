use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::events::{TrackStart, TrackEnd, TrackException, TrackStuck};
use lavalink_rs::macros::hook;
use tracing::{info, error, warn};
use std::sync::Arc;
use dashmap::DashMap;
use poise::serenity_prelude as serenity;
use serenity::all::Http;

use crate::config::Config;
use crate::data::{GuildPlayer, RepeatMode};
use crate::database::Database;
use crate::types::lavalink::player_saver::{PlayerData, QueueTrack, QueueTrackInfo};

#[derive(Clone)]
pub struct LavalinkHandler {
    pub config: Arc<Config>,
    pub guildPlayers: Arc<DashMap<serenity::GuildId, GuildPlayer>>,
    pub database: Database,
    pub redis: redis::aio::ConnectionManager,
    pub http: Arc<Http>,
}

#[hook]
pub async fn track_start(client: LavalinkClient, _sessionId: String, event: &TrackStart) {
    info!("[Lavalink] ▶ Track started in guild {}", event.guild_id);
    let handler = client.data::<LavalinkHandler>().expect("LavalinkHandler not found in client data");

    let log_embed = serenity::CreateEmbed::new()
        .title("▶ Track Started")
        .field("Guild ID", event.guild_id.to_string(), true)
        .field("Track", &event.track.info.title, true)
        .color(handler.config.colorSuccess)
        .timestamp(serenity::Timestamp::now());
    crate::utils::logs::send_log(&handler.http, &handler.config, crate::utils::logs::LogType::Node, log_embed).await;
    
    let serenityGuildId = serenity::GuildId::new(event.guild_id.0);
    if let Some(mut guildPlayer) = handler.guildPlayers.get_mut(&serenityGuildId) {
        let mut same_track = false;
        if let Some(current) = &guildPlayer.currentTrack {
            if current.track.info.identifier == event.track.info.identifier {
                same_track = true;
            }
        }

        if !same_track {
            warn!("[Lavalink] Desync detected in guild {}. Event track: {}, currentTrack: {:?}", 
                event.guild_id, 
                event.track.info.identifier,
                guildPlayer.currentTrack.as_ref().map(|t| &t.track.info.identifier)
            );
            
            if let Some(next) = guildPlayer.queue.front() {
                if next.track.info.identifier == event.track.info.identifier {
                    info!("[Lavalink] Found matching track at front of queue, advancing state.");
                    let next_track = guildPlayer.queue.pop_front();
                    guildPlayer.currentTrack = next_track;
                }
            }
        }

        if let Some(current) = guildPlayer.currentTrack.clone() {
            guildPlayer.lastTrackId = Some(current.track.info.identifier.clone());

            let _ = handler.database.incrementTrackStats(
                &current.track.info.identifier,
                &current.track.info.title,
                &current.track.info.author,
                &event.guild_id.to_string(),
                &current.requester.to_string(),
                &current.track.info.uri.as_ref().cloned().unwrap_or_default(),
                current.track.info.artwork_url.as_deref(),
                Some(current.track.info.length as i64),
                current.track.info.is_stream,
            ).await;

            let _ = handler.database.incrementUserStats(
                &current.requester.to_string(),
                &event.guild_id.to_string(),
            ).await;

            if let Some(_voiceChannelId) = guildPlayer.voiceChannelId {
                // let status = format!("♪ {} - {}", current.title, current.author);
                // let _ = serenity::ChannelId::new(voiceChannelId).edit_voice_status(&handler.http, status).await;
            }

            if let Some(msgId) = guildPlayer.nowPlayingMessageId {
                if let Some(channelId) = guildPlayer.textChannelId {
                    let _ = serenity::ChannelId::new(channelId).delete_message(&handler.http, msgId).await;
                }
            }

            guildPlayer.lastActiveAt = None;
            if let Some(channelId) = guildPlayer.textChannelId {
                let channel_id = serenity::ChannelId::new(channelId);
                let components = crate::components::v2::music::createNowPlayingV2(
                    &current.title,
                    &current.author,
                    &current.uri,
                    current.duration,
                    current.artworkUrl.as_deref(),
                    false,
                    current.requester.get(),
                );
                
                if let Ok(msg) = crate::components::v2::V2MessagePayload::new(components)
                    .send(&handler.http, channel_id)
                    .await
                {
                    guildPlayer.nowPlayingMessageId = Some(msg.id.get());
                }
            }
        }
    }

    let _ = savePlayerState(event.guild_id, &handler).await;
}

#[hook]
pub async fn track_end(client: LavalinkClient, _sessionId: String, event: &TrackEnd) {
    info!("[Lavalink] ⏹ Track ended in guild {} with reason {:?}", event.guild_id, event.reason);
    let guildId = event.guild_id;
    let handler = client.data::<LavalinkHandler>().expect("LavalinkHandler not found in client data");
    
    let serenityGuildId = serenity::GuildId::new(event.guild_id.0);
    if let Some(mut guildPlayer) = handler.guildPlayers.get_mut(&serenityGuildId) {
        if let Some(msgId) = guildPlayer.nowPlayingMessageId {
            if let Some(channelId) = guildPlayer.textChannelId {
                let _ = serenity::ChannelId::new(channelId).delete_message(&handler.http, msgId).await;
            }
            guildPlayer.nowPlayingMessageId = None;
        }

        if let Some(_voiceChannelId) = guildPlayer.voiceChannelId {
            // let _ = serenity::ChannelId::new(voiceChannelId).edit_voice_status(&handler.http, "").await;
        }

        if let Some(current) = guildPlayer.currentTrack.clone() {
            guildPlayer.previousTracks.push_front(current);
            if guildPlayer.previousTracks.len() > 10 {
                guildPlayer.previousTracks.pop_back();
            }
        }
    }

    if event.reason == lavalink_rs::model::events::TrackEndReason::Finished || 
       event.reason == lavalink_rs::model::events::TrackEndReason::LoadFailed ||
       event.reason == lavalink_rs::model::events::TrackEndReason::Stopped {
        let _ = playNextTrack(guildId, client.clone(), &handler.guildPlayers, &handler.database, &handler.config).await;
    } else {
        if let Some(mut player) = handler.guildPlayers.get_mut(&serenityGuildId) {
            player.currentTrack = None;
            player.nowPlayingMessageId = None;
            if player.queue.is_empty() {
                player.lastActiveAt = Some(chrono::Utc::now().timestamp() as u64);
            }
        }
    }

    if let Some(mut player) = handler.guildPlayers.get_mut(&serenityGuildId) {
        if player.currentTrack.is_none() && player.queue.is_empty() {
             if player.lastActiveAt.is_none() {
                 player.lastActiveAt = Some(chrono::Utc::now().timestamp() as u64);
             }
             
             if player.autoplay {
                 if let Some(last_track) = player.previousTracks.front() {
                     let last_track_data = last_track.track.clone();
                     let mut player_clone = player.clone();
                     let lava = client.clone();
                     
                     tokio::spawn(async move {
                         let _ = crate::lavalink::autoplay::handleAutoplay(
                             &lava,
                             guildId,
                             &mut player_clone,
                             &last_track_data,
                             &[] // TODO: Add Last.fm keys if available in config
                         ).await;
                     });
                 }
             }
        }
    }
    
    let log_embed = serenity::CreateEmbed::new()
        .title("⏹ Track Ended")
        .field("Guild ID", event.guild_id.to_string(), true)
        .field("Reason", format!("{:?}", event.reason), true)
        .color(handler.config.colorError)
        .timestamp(serenity::Timestamp::now());
    crate::utils::logs::send_log(&handler.http, &handler.config, crate::utils::logs::LogType::Node, log_embed).await;

    let _ = savePlayerState(event.guild_id, &handler).await;
}

#[hook]
pub async fn track_exception(client: LavalinkClient, _session_id: String, event: &TrackException) {
    error!("[Lavalink] ❌ Track exception in guild {}: {:?}", event.guild_id, event.exception);
    let handler = client.data::<LavalinkHandler>().expect("LavalinkHandler not found in client data");

    let log_embed = serenity::CreateEmbed::new()
        .title("❌ Track Exception")
        .field("Guild ID", event.guild_id.to_string(), true)
        .field("Error", format!("{:?}", event.exception), false)
        .color(handler.config.colorError)
        .timestamp(serenity::Timestamp::now());
    crate::utils::logs::send_log(&handler.http, &handler.config, crate::utils::logs::LogType::Error, log_embed).await;
}

#[hook]
pub async fn track_stuck(client: LavalinkClient, _session_id: String, event: &TrackStuck) {
    warn!("[Lavalink] ⚠ Track stuck in guild {} at {}ms", event.guild_id, event.threshold_ms);
    let handler = client.data::<LavalinkHandler>().expect("LavalinkHandler not found in client data");

    let log_embed = serenity::CreateEmbed::new()
        .title("⚠ Track Stuck")
        .field("Guild ID", event.guild_id.to_string(), true)
        .field("Threshold", format!("{}ms", event.threshold_ms), true)
        .color(handler.config.colorError)
        .timestamp(serenity::Timestamp::now());
    crate::utils::logs::send_log(&handler.http, &handler.config, crate::utils::logs::LogType::Error, log_embed).await;
}

pub async fn savePlayerState(guildId: lavalink_rs::model::GuildId, handler: &LavalinkHandler) -> Result<(), crate::error::BotError> {
    let serenityGuildId = serenity::GuildId::new(guildId.0);
    let guildPlayer = match handler.guildPlayers.get(&serenityGuildId) {
        Some(p) => p,
        None => return Ok(()),
    };
    
    let playerData = PlayerData {
        guildId: guildId.0.to_string(),
        voiceChannelId: guildPlayer.voiceChannelId.map(|id| id.to_string()),
        textChannelId: guildPlayer.textChannelId.map(|id| id.to_string()),
        messageId: guildPlayer.nowPlayingMessageId.map(|id| id.to_string()),
        nodeId: None,
        nodeSessionId: guildPlayer.voiceSessionId.clone(),
        volume: Some(guildPlayer.volume),
        repeatMode: Some(match guildPlayer.repeatMode {
            RepeatMode::Off => "off".to_string(),
            RepeatMode::Track => "track".to_string(),
            RepeatMode::Queue => "queue".to_string(),
        }),
        enabledAutoplay: Some(guildPlayer.autoplay),
        lyricsEnabled: None,
        lyricsId: None,
        lyricsRequester: None,
        localeString: None,
        lyrics: None,
        track: guildPlayer.currentTrack.as_ref().map(|t| QueueTrack {
            encoded: Some(t.track.encoded.clone()),
            info: Some(QueueTrackInfo {
                title: Some(t.title.clone()),
                uri: Some(t.uri.clone()),
                author: Some(t.author.clone()),
                duration: Some(t.duration),
                identifier: Some(t.track.info.identifier.clone()),
                isStream: Some(t.isStream),
                isSeekable: Some(t.track.info.is_seekable),
                sourceName: Some(t.track.info.source_name.clone()),
                artworkUrl: t.artworkUrl.clone(),
            }),
            requester: Some(t.requester.to_string()),
        }),
        queue: Some(guildPlayer.queue.iter().map(|t| QueueTrack {
            encoded: Some(t.track.encoded.clone()),
            info: Some(QueueTrackInfo {
                title: Some(t.title.clone()),
                uri: Some(t.uri.clone()),
                author: Some(t.author.clone()),
                duration: Some(t.duration),
                identifier: Some(t.track.info.identifier.clone()),
                isStream: Some(t.isStream),
                isSeekable: Some(t.track.info.is_seekable),
                sourceName: Some(t.track.info.source_name.clone()),
                artworkUrl: t.artworkUrl.clone(),
            }),
            requester: Some(t.requester.to_string()),
        }).collect()),
    };
    
    handler.database.savePlayerData(&guildId.0.to_string(), &playerData).await?;
    
    let mut redis_con = handler.redis.clone();
    let _ = crate::utils::redis::savePlayer(&mut redis_con, guildId.0, &guildPlayer).await;
    
    Ok(())
}

pub async fn playNextTrack(
    guildId: lavalink_rs::model::GuildId, 
    client: LavalinkClient,
    guildPlayers: &Arc<DashMap<serenity::GuildId, GuildPlayer>>,
    _database: &Database,
    _config: &Arc<Config>,
) -> Result<(), crate::error::BotError> {
    let serenityGuildId = serenity::GuildId::new(guildId.0);
    
    let (nextTrack, autoplayEnabled, lastTrack) = {
        let mut gp = guildPlayers.get_mut(&serenityGuildId).ok_or(crate::error::BotError::NoPlayer)?;
        
        let next = match gp.repeatMode {
            RepeatMode::Track => {
                gp.currentTrack.clone()
            },
            RepeatMode::Queue => {
                if let Some(current) = gp.currentTrack.take() {
                    gp.queue.push_back(current);
                }
                gp.queue.pop_front()
            },
            RepeatMode::Off => {
                gp.queue.pop_front()
            }
        };

        if next.is_some() {
            gp.currentTrack = next.clone();
            if let Some(ref track) = next {
                gp.lastTrackId = Some(track.track.info.identifier.clone());
            }
            (next, gp.autoplay, None)
        } else {
            let last = gp.currentTrack.take();
            gp.currentTrack = None;
            (None, gp.autoplay, last)
        }
    };

    if let Some(track) = nextTrack {
        if let Some(player) = client.get_player_context(guildId) {
            player.play(&track.track).await?;
        }
    } else if autoplayEnabled {
        if let Some(last_track) = lastTrack {
            let lava = client.clone();
            let mut gp_clone = {
                let gp = guildPlayers.get(&serenityGuildId).ok_or(crate::error::BotError::NoPlayer)?;
                gp.clone()
            };
            
            tokio::spawn(async move {
                let _ = crate::lavalink::autoplay::handleAutoplay(
                    &lava,
                    guildId,
                    &mut gp_clone,
                    &last_track.track,
                    &[] 
                ).await;
            });
        }
    }

    Ok(())
}
