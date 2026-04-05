use poise::serenity_prelude as serenity;
use tracing::{info, error};

use crate::data::Data;
use crate::error::BotError;

pub async fn handle(
    ctx: &serenity::Context,
    _old: &Option<serenity::VoiceState>,
    new: &serenity::VoiceState,
    data: &Data,
) -> Result<(), BotError> {
    let guildId = match new.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let botId = ctx.cache.current_user().id;

    if new.user_id == botId {
        let sessionId = &new.session_id;
        if let Some(mut player) = data.guildPlayers.get_mut(&guildId) {
            player.voiceSessionId = Some(sessionId.clone());
            player.voiceChannelId = new.channel_id.map(|cid| cid.get());
            info!("[Voice] Received VoiceStateUpdate for guild {} (Session: {}, Channel: {:?})", guildId, sessionId, player.voiceChannelId);
            
            if let (Some(ep), Some(token), Some(session)) = (
                player.voiceEndpoint.as_ref(),
                player.voiceToken.as_ref(),
                player.voiceSessionId.as_ref()
              ) {
                let lavalink = data.lavalink.clone();
                let epClone: String = ep.to_string();
                let tokenClone: String = token.to_string();
                let sessionClone: String = session.to_string();
                let chan = player.voiceChannelId;
                
                tokio::spawn(async move {
                    if let Err(e) = crate::lavalink::updatePlayerVoice(&lavalink, guildId, &epClone, &tokenClone, &sessionClone, chan).await {
                        error!("[Voice] Failed to update Lavalink player: {}", e);
                    }
                });
            }
        }

        if new.channel_id.is_none() {
            let mut should_remove = true;
            if let Some(player) = data.guildPlayers.get(&guildId) {
                if player.mode247 || player.voiceChannelId.is_some() {
                    should_remove = false;
                }
            }

            if should_remove {
                info!("[Voice] Bot disconnected from guild {} - cleaning up.", guildId);
                data.guildPlayers.remove(&guildId);
                let _ = data.lavalink.delete_player(guildId).await;
            } else {
                info!("[Voice] Bot disconnected from guild {} but re-joining or 24/7. Skipping removal.", guildId);
            }
        }
        return Ok(());
    }

    let botChannel = {
        let guild = match ctx.cache.guild(guildId) {
            Some(g) => g.clone(),
            None => return Ok(()),
        };
        guild
            .voice_states
            .get(&botId)
            .and_then(|vs| vs.channel_id)
    };

    let botChannel = match botChannel {
        Some(ch) => ch,
        None => return Ok(()),
    };

    let usersInChannel = {
        let guild = match ctx.cache.guild(guildId) {
            Some(g) => g.clone(),
            None => return Ok(()),
        };
        guild
            .voice_states
            .iter()
            .filter(|(uid, vs)| {
                vs.channel_id == Some(botChannel) && **uid != botId
            })
            .count()
    };

    if usersInChannel == 0 {
        if let Some(mut player) = data.guildPlayers.get_mut(&guildId) {
            if !player.mode247 && player.aloneSince.is_none() {
                info!("[Voice] Bot is alone in guild {} — starting 1min timeout", guildId);
                player.aloneSince = Some(chrono::Utc::now().timestamp() as u64);
            }
        }
    } else {
        if let Some(mut player) = data.guildPlayers.get_mut(&guildId) {
            if player.aloneSince.is_some() {
                info!("[Voice] Users returned to channel in guild {} — resetting timeout", guildId);
                player.aloneSince = None;
            }
        }
    }

    Ok(())
}
