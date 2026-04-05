pub mod autoplay;
use lavalink_rs::client::LavalinkClient;
use std::sync::Arc;
use dashmap::DashMap;
use poise::serenity_prelude::GuildId;
use tracing::{info, error};

use crate::error::BotError;
use crate::config::Config;
use crate::data::GuildPlayer;
use crate::database::Database;

pub async fn createLavalinkClient(
    config: Arc<Config>,
    database: Database,
    redis: redis::aio::ConnectionManager,
    guildPlayers: Arc<DashMap<GuildId, GuildPlayer>>,
    http: Arc<poise::serenity_prelude::Http>,
    _botUserId: u64,
) -> Result<LavalinkClient, BotError> {
    let events = lavalink_rs::model::events::Events {
        track_start: Some(crate::events::lavalink::track_start),
        track_end: Some(crate::events::lavalink::track_end),
        track_exception: Some(crate::events::lavalink::track_exception),
        track_stuck: Some(crate::events::lavalink::track_stuck),
        ..Default::default()
    };

    let mut nodes = Vec::new();
    for node_cfg in &config.lavalinkNodes {
        let node = lavalink_rs::node::NodeBuilder {
            hostname: format!("{}:{}", node_cfg.host, node_cfg.port),
            password: node_cfg.password.to_string(),
            user_id: _botUserId.into(),
            ..Default::default()
        };
        nodes.push(node);
    }
    
    let handler = crate::events::lavalink::LavalinkHandler {
        config,
        guildPlayers,
        database,
        redis,
        http,
    };

    let client = LavalinkClient::new_with_data(
        events,
        nodes,
        lavalink_rs::model::client::NodeDistributionStrategy::RoundRobin(std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0))),
        Arc::new(handler),
    ).await;

    Ok(client)
}

pub async fn updatePlayerVoice(
    lavalink: &LavalinkClient,
    guildId: GuildId,
    endpoint: &str,
    token: &str,
    sessionId: &str,
    channelId: Option<u64>,
) -> Result<(), BotError> {
    let mut cleanEndpoint = endpoint.to_string();
    if let Some(pos) = cleanEndpoint.find("://") {
        cleanEndpoint = cleanEndpoint[(pos + 3)..].to_string();
    }
    let mut cleanEndpoint = cleanEndpoint.trim().trim_end_matches('.').to_string();
    if !cleanEndpoint.contains(':') {
        cleanEndpoint.push_str(":443");
    }

    info!("[Voice] Updating Lavalink player for guild {} (Endpoint: {}, Session: {}, Channel: {:?})", guildId, cleanEndpoint, sessionId, channelId);
    
    let voiceData = lavalink_rs::model::player::ConnectionInfo {
        token: token.to_string(),
        endpoint: cleanEndpoint,
        session_id: sessionId.to_string(),
        channel_id: channelId.map(lavalink_rs::model::ChannelId::from),
    };

    let update = lavalink_rs::model::http::UpdatePlayer {
        voice: Some(voiceData.clone()),
        ..Default::default()
    };

    if let Some(player) = lavalink.get_player_context(guildId) {
        info!("[Voice] Player context exists for guild {}, updating...", guildId);
        player.update_player(&update, false).await.map_err(|e| {
            error!("[Voice] Failed to update player via context: {}", e);
            BotError::Lavalink(e.to_string())
        })?;
    } else {
        info!("[Voice] No player context for guild {}, creating new one...", guildId);
        lavalink.create_player_context(guildId, voiceData).await.map_err(|e| {
            error!("[Voice] Failed to create player context: {}", e);
            BotError::Lavalink(e.to_string())
        })?;
    }

    info!("[Voice] Updated Lavalink player for guild {}", guildId);
    Ok(())
}

pub async fn getLyrics(
    config: &Config,
    guildId: u64,
) -> Result<crate::types::lavalink::lyrics::LyricsSearchResponse, BotError> {
    let node = config.lavalinkNodes.first().ok_or(BotError::Other("No Lavalink nodes available".into()))?;
    let client = reqwest::Client::new();
    let url = format!(
        "http{}://{}:{}/v4/sessions/any/players/{}/lyrics",
        if node.secure { "s" } else { "" },
        node.host, 
        node.port, 
        guildId
    );

    let response = client
        .get(&url)
        .header("Authorization", &node.password)
        .send()
        .await
        .map_err(|e| BotError::Other(format!("Failed to fetch lyrics request: {}", e)))?;

    if response.status() == 204 {
        return Err(BotError::Other("No lyrics found for this track.".into()));
    }

    let lyrics = response
        .json::<crate::types::lavalink::lyrics::LyricsSearchResponse>()
        .await
        .map_err(|e| BotError::Other(format!("Failed to parse lyrics: {}", e)))?;

    Ok(lyrics)
}
