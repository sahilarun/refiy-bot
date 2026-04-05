use poise::serenity_prelude as serenity;
use crate::data::{Data, RepeatMode};
use crate::error::BotError;

pub async fn handle(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    data: &Data,
) -> Result<(), BotError> {
    if let serenity::Interaction::Component(mci) = interaction {
        let custom_id = &mci.data.custom_id;
        
        if !custom_id.starts_with("music_") {
            return Ok(());
        }

        let guild_id = match mci.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let user_id = mci.user.id;
        
        let (bot_voice_channel, user_voice_channel) = {
            let guild = guild_id.to_guild_cached(&ctx).ok_or(BotError::NoPlayer)?;
            let bvc = guild.voice_states.get(&ctx.cache.current_user().id)
                .and_then(|vs| vs.channel_id);
            let uvc = guild.voice_states.get(&user_id)
                .and_then(|vs| vs.channel_id);
            (bvc, uvc)
        };

        if user_voice_channel.is_none() {
            mci.create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("You must be in a voice channel to use music controls.")
                        .ephemeral(true)
                ),
            ).await?;
            return Ok(());
        }

        if bot_voice_channel.is_some() && user_voice_channel != bot_voice_channel {
            mci.create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("You must be in the same voice channel as the bot to use controls.")
                        .ephemeral(true)
                ),
            ).await?;
            return Ok(());
        }

        let pc = data.lavalink.get_player_context(lavalink_rs::model::GuildId(guild_id.get()))
            .ok_or(BotError::NoPlayer)?;
        
        let mut player = data.guildPlayers.get_mut(&guild_id)
            .ok_or(BotError::NoPlayer)?;

        mci.create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Acknowledge,
        ).await?;

        match custom_id.as_str() {
            "music_pause" => {
                let paused = player.paused;
                pc.set_pause(!paused).await
                    .map_err(|e| BotError::Lavalink(e.to_string()))?;
                player.paused = !paused;
                if let (Some(msg_id), Some(chan_id)) = (player.nowPlayingMessageId, player.textChannelId) {
                    if let Some(track) = &player.currentTrack {
                        let components = crate::components::v2::music::createNowPlayingV2(
                            &track.title,
                            &track.author,
                            &track.uri,
                            track.duration,
                            track.artworkUrl.as_deref(),
                            player.paused,
                            track.requester.get(),
                        );
                        let _ = crate::components::v2::V2MessagePayload::new(components)
                            .edit(&ctx.http, serenity::ChannelId::new(chan_id), serenity::MessageId::new(msg_id))
                            .await;
                    }
                }
            }
            "music_skip" => {
                pc.stop_now().await
                    .map_err(|e| BotError::Lavalink(e.to_string()))?;
            }
            "music_repeat" => {
                player.repeatMode = match player.repeatMode {
                    RepeatMode::Off => RepeatMode::Track,
                    RepeatMode::Track => RepeatMode::Queue,
                    RepeatMode::Queue => RepeatMode::Off,
                };
            }
            "music_shuffle" => {
                use rand::seq::SliceRandom;
                let mut items: Vec<_> = player.queue.drain(..).collect();
                items.shuffle(&mut rand::thread_rng());
                player.queue.extend(items);
            }
            _ => return Ok(()),
        }

        Ok(())
    } else {
        Ok(())
    }
}
