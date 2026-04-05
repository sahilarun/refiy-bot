use poise::serenity_prelude as serenity;
use tracing::info;
use std::collections::VecDeque;

use crate::data::{Data, GuildPlayer, QueuedTrack, RepeatMode};
use crate::error::BotError;
use crate::types::lavalink::player_saver::PlayerData;

pub async fn handle(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    data: &Data,
) -> Result<(), BotError> {
    info!("[Bot] ✅ Logged in as {} (ID: {})", ready.user.name, ready.user.id);
    info!("[Bot] Connected to {} guild(s)", ready.guilds.len());

    let embed = serenity::CreateEmbed::new()
        .title("🚀 Bot Ready")
        .description(format!("Logged in as **{}**\nConnected to **{}** guilds", ready.user.name, ready.guilds.len()))
        .color(data.config.colorSuccess)
        .timestamp(serenity::Timestamp::now());
    crate::utils::logs::send_log(&ctx.http, &data.config, crate::utils::logs::LogType::Node, embed).await;

    ctx.set_presence(
        Some(serenity::ActivityData::playing("Traveling... 🌠")),
        serenity::OnlineStatus::Idle,
    );

    let allPlayerData: Vec<PlayerData> = match data.database.getAllPlayerData().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("[Recovery] Failed to load session data: {}", e);
            Vec::new()
        }
    };
    info!("[Recovery] Found {} saved sessions", allPlayerData.len());

    for node in data.lavalink.nodes.iter() {
        info!("[Lavalink] Node {} (Websocket: {}) connected", node.id, node.websocket_address);
    }

    if let Err(e) = data.database.clearAllPlayerData().await {
        tracing::error!("[Recovery] Failed to clear stale session data: {}", e);
    }

    for p in allPlayerData {
        let guildId = match p.guildId.parse::<u64>() {
            Ok(id) => serenity::GuildId::new(id),
            Err(_) => continue,
        };

        let is_247 = matches!(data.database.get247Mode(&guildId.to_string()).await, Ok(Some((true, _, _))));
        if !is_247 {
            continue;
        }

        let voiceChannelId = match p.voiceChannelId.as_ref().and_then(|id: &String| id.parse::<u64>().ok()) {
            Some(id) => serenity::ChannelId::new(id),
            None => continue,
        };

        let mut queue = VecDeque::new();
        if let Some(q) = p.queue {
            for t in q {
                if let (Some(encoded), Some(info)) = (t.encoded, t.info) {
                    queue.push_back(QueuedTrack {
                        track: lavalink_rs::model::track::TrackData {
                            encoded,
                            info: lavalink_rs::model::track::TrackInfo {
                                identifier: info.identifier.clone().unwrap_or_default(),
                                is_seekable: info.isSeekable.unwrap_or(true),
                                author: info.author.clone().unwrap_or_default(),
                                length: info.duration.unwrap_or(0),
                                is_stream: info.isStream.unwrap_or(false),
                                position: 0,
                                title: info.title.clone().unwrap_or_default(),
                                uri: info.uri.clone(),
                                source_name: info.sourceName.clone().unwrap_or_default(),
                                artwork_url: info.artworkUrl.clone(),
                                isrc: None,
                            },
                            plugin_info: None,
                            user_data: None,
                        },
                        requester: t.requester.as_ref().and_then(|id: &String| id.parse::<u64>().ok()).unwrap_or(0).into(),
                        title: info.title.clone().unwrap_or_default(),
                        author: info.author.clone().unwrap_or_default(),
                        uri: info.uri.clone().unwrap_or_default(),
                        duration: info.duration.unwrap_or(0),
                        isStream: info.isStream.unwrap_or(false),
                        artworkUrl: info.artworkUrl.clone(),
                    });
                }
            }
        }

        let currentTrack = p.track.and_then(|t| {
            if let (Some(encoded), Some(info)) = (t.encoded, t.info) {
                Some(QueuedTrack {
                    track: lavalink_rs::model::track::TrackData {
                        encoded,
                        info: lavalink_rs::model::track::TrackInfo {
                            identifier: info.identifier.clone().unwrap_or_default(),
                            is_seekable: info.isSeekable.unwrap_or(true),
                            author: info.author.clone().unwrap_or_default(),
                            length: info.duration.unwrap_or(0),
                            is_stream: info.isStream.unwrap_or(false),
                            position: 0,
                            title: info.title.clone().unwrap_or_default(),
                            uri: info.uri.clone(),
                            source_name: info.sourceName.clone().unwrap_or_default(),
                            artwork_url: info.artworkUrl.clone(),
                            isrc: None,
                        },
                        plugin_info: None,
                        user_data: None,
                    },
                    requester: t.requester.as_ref().and_then(|id: &String| id.parse::<u64>().ok()).unwrap_or(0).into(),
                    title: info.title.clone().unwrap_or_default(),
                    author: info.author.clone().unwrap_or_default(),
                    uri: info.uri.clone().unwrap_or_default(),
                    duration: info.duration.unwrap_or(0),
                    isStream: info.isStream.unwrap_or(false),
                    artworkUrl: info.artworkUrl.clone(),
                })
            } else {
                None
            }
        });

        let repeatMode = match p.repeatMode.as_deref() {
            Some("track") => RepeatMode::Track,
            Some("queue") => RepeatMode::Queue,
            _ => RepeatMode::Off,
        };

        let gp = GuildPlayer {
            queue,
            currentTrack: currentTrack.clone(),
            volume: p.volume.unwrap_or(100),
            paused: false,
            repeatMode,
            autoplay: p.enabledAutoplay.unwrap_or(false),
            textChannelId: p.textChannelId.as_ref().and_then(|id: &String| id.parse::<u64>().ok()),
            voiceChannelId: Some(voiceChannelId.get()),
            nowPlayingMessageId: p.messageId.as_ref().and_then(|id: &String| id.parse::<u64>().ok()),
            voiceEndpoint: None,
            voiceToken: None,
            voiceSessionId: p.nodeSessionId.clone(),
            lastTrackId: None,
            lyricsEnabled: false,
            lyricsId: None,
            lyricsRequester: None,
            mode247: is_247,
            is247: is_247,
            previousTracks: VecDeque::new(),
            lastActiveAt: None,
            aloneSince: None,
        };
        let is_247 = gp.mode247;

        data.guildPlayers.insert(guildId, gp);

        if is_247 {
            info!("[Recovery] 24/7 Autojoin for guild {} to {}", guildId, voiceChannelId);
            
            let payload = serde_json::json!({
                "op": 4,
                "d": {
                    "guild_id": guildId.to_string(),
                    "channel_id": voiceChannelId.to_string(),
                    "self_mute": false,
                    "self_deaf": false
                }
            });
            
            let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&payload).unwrap());
            ctx.shard.websocket_message(message);

            if let Some(ct) = currentTrack {
                let lava = data.lavalink.clone();
                tokio::spawn(async move {
                    let max_retries = 5;
                    for i in 0..max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        if let Some(player) = lava.get_player_context(lavalink_rs::model::GuildId(guildId.get())) {
                            info!("[Recovery] Player context found for {} on retry {}, resuming track...", guildId, i + 1);
                            let _ = player.play(&ct.track).await;
                            break;
                        } else {
                            info!("[Recovery] Waiting for player context for {} (Attempt {}/{})...", guildId, i + 1, max_retries);
                        }
                    }
                });
            }
        }
    }

    let guildPlayers = data.guildPlayers.clone();
    let lava = data.lavalink.clone();
    let shard = ctx.shard.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let guilds: Vec<serenity::GuildId> = guildPlayers.iter().map(|r| *r.key()).collect();
            
            for g_id in guilds {
                let mut should_leave = false;
                if let Some(gp) = guildPlayers.get(&g_id) {
                    if gp.mode247 { continue; }
                    if let Some(idle_at) = gp.lastActiveAt {
                        let elapsed = chrono::Utc::now().timestamp() as u64 - idle_at;
                        if elapsed > 60 {
                            info!("[Monitor] Inactivity leave for guild {}", g_id);
                            should_leave = true;
                        }
                    }
                    if let Some(alone_at) = gp.aloneSince {
                        let elapsed = chrono::Utc::now().timestamp() as u64 - alone_at;
                        if elapsed > 60 {
                            info!("[Monitor] Alone leave for guild {}", g_id);
                            should_leave = true;
                        }
                    }
                }

                if should_leave {
                    let payload = serde_json::json!({
                        "op": 4,
                        "d": {
                            "guild_id": g_id.to_string(),
                            "channel_id": null,
                            "self_mute": false,
                            "self_deaf": false
                        }
                    });
                    let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&payload).unwrap());
                    shard.websocket_message(message);
                    let _ = lava.delete_player(lavalink_rs::model::GuildId(g_id.get())).await;
                    guildPlayers.remove(&g_id);
                }
            }
        }
    });

    Ok(())
}
