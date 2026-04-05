pub mod ready;
pub mod voice_state_update;
pub mod guild_create;
pub mod guild_delete;
pub mod lavalink;
pub mod message_create;
pub mod interaction_create;

// use std::sync::Arc;
use tracing::{info, error};
use poise::serenity_prelude as serenity;

use crate::data::Data;
use crate::error::BotError;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, BotError>,
    data: &Data,
) -> Result<(), BotError> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            ready::handle(ctx, data_about_bot, data).await?;
        }

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            let bot_id = ctx.cache.current_user().id;
            if new.user_id == bot_id && new.channel_id.is_none() {
                if let Some(mut player) = data.guildPlayers.get_mut(&new.guild_id.unwrap()) {
                    player.voiceSessionId = None;
                    player.voiceChannelId = None;
                    player.voiceToken = None;
                    player.voiceEndpoint = None;
                }
            }
            voice_state_update::handle(ctx, old, new, data).await?;
        }

        serenity::FullEvent::GuildCreate { guild, is_new } => {
            guild_create::handle(ctx, guild, *is_new, data).await?;
        }

        serenity::FullEvent::GuildDelete { incomplete, full } => {
            guild_delete::handle(ctx, incomplete, full.as_ref(), data).await?;
        }
        
        serenity::FullEvent::VoiceServerUpdate { event } => {
            if let Some(guildId) = event.guild_id {
                let token = event.token.clone();
                let endpoint = event.endpoint.clone();
                
                if let Some(mut player) = data.guildPlayers.get_mut(&guildId) {
                    player.voiceToken = Some(token.clone());
                    player.voiceEndpoint = endpoint.clone();
                    info!("[Voice] Received VoiceServerUpdate for guild {} (Endpoint: {:?}, Token: YES)", guildId, endpoint);
                    
                    if let (Some(ep), Some(session), Some(chan)) = (player.voiceEndpoint.as_ref(), player.voiceSessionId.as_ref(), player.voiceChannelId) {
                        let lavalink = data.lavalink.clone();
                        let epClone: String = ep.to_string();
                        let sessionId: String = session.to_string();
                        let tokenClone: String = token.to_string();
                        
                        tokio::spawn(async move {
                            if let Err(e) = crate::lavalink::updatePlayerVoice(&lavalink, guildId, &epClone, &tokenClone, &sessionId, Some(chan)).await {
                                error!("[Voice] Failed to update Lavalink player: {}", e);
                            }
                        });
                    }
                }
            }
        }

        serenity::FullEvent::Message { new_message } => {
            message_create::handle(ctx, new_message, data).await?;
        }

        serenity::FullEvent::InteractionCreate { interaction } => {
            interaction_create::handle(ctx, interaction, data).await?;
        }

        _ => {}
    }

    Ok(())
}
