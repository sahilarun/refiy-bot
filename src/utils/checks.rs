use poise::serenity_prelude::{self as serenity, GuildId, ChannelId};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, error};

use crate::data::{Context, GuildPlayer};
use crate::error::BotError;
use crate::components::v2::{Container, V2Component, TextDisplay, V2MessagePayload};

pub async fn getAuthorVoiceChannel(
    ctx: Context<'_>,
) -> Result<(GuildId, serenity::ChannelId), BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or(BotError::Other("Must be used in a guild".into()))?;
    let guild = ctx
        .guild()
        .ok_or(BotError::Other("Cannot access guild".into()))?
        .clone();
    let user_id = ctx.author().id;
    let channel_id = guild
        .voice_states
        .get(&user_id)
        .and_then(|vs| vs.channel_id)
        .ok_or(BotError::NotInVoice)?;
    Ok((guild_id, channel_id))
}

pub fn requirePlayer(ctx: Context<'_>) -> Result<GuildId, BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or(BotError::Other("Must be used in a guild".into()))?;
    if !ctx.data().guildPlayers.contains_key(&guild_id) {
        return Err(BotError::NoPlayer);
    }
    Ok(guild_id)
}

pub async fn ensureConnected(
    ctx: Context<'_>,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), BotError> {
    let lava = &ctx.data().lavalink;

    if lava.get_player_context(guild_id).is_some() {
        return Ok(());
    }

    {
        let mut player = ctx.data()
            .guildPlayers
            .entry(guild_id)
            .or_insert_with(|| GuildPlayer::new(ctx.data().config.defaultVolume));

        player.voiceChannelId = Some(channel_id.get());
    }

    let shard_manager = ctx.framework().shard_manager();
    let shard_id = {
        let guild = ctx.guild().ok_or(BotError::Other("Cannot access guild".into()))?;
        serenity::ShardId(guild.shard_id(&ctx.serenity_context().cache))        
    };

    let runners = shard_manager.runners.lock().await;
    if let Some(runner) = runners.get(&shard_id) {
        let payload = json!({
            "op": 4,
            "d": {
                "guild_id": guild_id.to_string(),
                "channel_id": channel_id.to_string(),
                "self_mute": false,
                "self_deaf": false
            }
        });

        let message = Message::Text(serde_json::to_string(&payload).map_err(|e| BotError::Other(e.to_string()))?);
        runner.runner_tx.websocket_message(message);
    }
    drop(runners);

    info!("[Voice] Waiting for Lavalink player to be ready for guild {}...", guild_id);
    for i in 0..100 {
        if let Some(player) = lava.get_player_context(guild_id) {
            info!("[Voice] Lavalink player is ready for guild {} after {}ms", guild_id, (i + 1) * 100);
            if player.get_player().await.is_ok_and(|p| p.voice.token != "") {
                 return Ok(());
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    error!("[Voice] Timeout waiting for Lavalink player for guild {}", guild_id);
    Err(BotError::Lavalink("Timed out waiting for Lavalink player to be ready".to_string()))
}

pub async fn checkVoiceChannel(ctx: Context<'_>) -> Result<bool, BotError> {    
    let guild = ctx.guild().ok_or(BotError::Other("Guild data missing".into()))?.clone();
    let user_voice = guild.voice_states.get(&ctx.author().id).and_then(|vs| vs.channel_id);

    if user_voice.is_none() {
        let components = vec![V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("❌ You must be in a voice channel.".to_string())),
        ]))];
        V2MessagePayload::new(components).send_interaction(ctx).await?;
        return Ok(false);
    }
    Ok(true)
}

pub async fn checkBotVoiceChannel(ctx: Context<'_>) -> Result<bool, BotError> { 
    let guild = ctx.guild().ok_or(BotError::Other("Guild data missing".into()))?.clone();
    let me = ctx.serenity_context().cache.current_user().id;

    let user_voice = guild.voice_states.get(&ctx.author().id).and_then(|vs| vs.channel_id);
    let bot_voice = guild.voice_states.get(&me).and_then(|vs| vs.channel_id);    

    if let Some(bot_chan) = bot_voice {
        if user_voice != Some(bot_chan) {
            let components = vec![V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!("❌ You must be in the same voice channel as me (<#{}>).", bot_chan))),
            ]))];
            V2MessagePayload::new(components).send_interaction(ctx).await?;
            return Ok(false);
        }
    }
    Ok(true)
}

pub async fn checkCooldown(ctx: Context<'_>) -> Result<bool, BotError> {        
    let user_id = ctx.author().id.to_string();
    let command_name = ctx.command().name.clone();
    let key = format!("{}-{}", user_id, command_name);

    let now = chrono::Utc::now().timestamp_millis();
    let cooldown_duration = 3000;

    if let Some(expiry) = ctx.data().cooldowns.get(&key) {
        if now < *expiry as i64 {
            let remaining = (*expiry as i64 - now) / 1000;
            let components = vec![V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!("❌ Slow down! You can use this command again in {} seconds.", remaining + 1))),
            ]))];
            V2MessagePayload::new(components).send_interaction(ctx).await?;
            return Ok(false);
        }
    }

    ctx.data().cooldowns.insert(key, (now + cooldown_duration) as u64);
    Ok(true)
}

pub async fn checkNodes(ctx: Context<'_>) -> Result<bool, BotError> {
    let lava = &ctx.data().lavalink;
    if lava.nodes.is_empty() {
        let components = vec![V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("❌ No music nodes are currently available.".to_string())),
        ]))];
        V2MessagePayload::new(components).send_interaction(ctx).await?;
        return Ok(false);
    }
    Ok(true)
}

pub async fn checkPlayer(ctx: Context<'_>) -> Result<bool, BotError> {
    let guild_id = ctx.guild_id().ok_or(BotError::Other("Must be used in a guild".into()))?;
    let lava = &ctx.data().lavalink;

    if lava.get_player_context(guild_id.get()).is_none() {
        let components = vec![V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("❌ There is no active player in this server.".to_string())),
        ]))];
        V2MessagePayload::new(components).send_interaction(ctx).await?;
        return Ok(false);
    }
    Ok(true)
}

pub async fn checkTracks(ctx: Context<'_>) -> Result<bool, BotError> {
    let guild_id = ctx.guild_id().ok_or(BotError::Other("Must be used in a guild".into()))?;
    {
        let players = &ctx.data().guildPlayers;
        if let Some(player) = players.get(&guild_id) {
            if player.autoplay {
                return Ok(true);
            }
        }
    }

    let lava = &ctx.data().lavalink;
    let has_tracks = if let Some(player) = lava.get_player_context(guild_id) {
        let queue = player.get_queue();
        let current = player.get_player().await.ok().and_then(|p| p.track);
        let length = queue.get_queue().await.map(|q| q.len()).unwrap_or(0);
        current.is_some() || length > 0
    } else {
        false
    };

    if !has_tracks {
        let components = vec![V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("❌ The queue is currently empty.".to_string())),
        ]))];
        V2MessagePayload::new(components).send(&ctx.serenity_context().http, ctx.channel_id()).await?;
        return Ok(false);
    }
    Ok(true)
}

