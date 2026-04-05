use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("prev"))]
pub async fn previous(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;
    let lavalink = &ctx.data().lavalink;

    let prevTrack = if let Some(mut guildPlayer) = guild_players.get_mut(&guild_id) {
        if let Some(prev) = guildPlayer.previousTracks.pop_front() {
            if let Some(current) = guildPlayer.currentTrack.clone() {
                guildPlayer.queue.push_front(current);
            }
            guildPlayer.currentTrack = Some(prev.clone());
            Some(prev)
        } else {
            return Err(BotError::Other("> No previous tracks found.".into()));
        }
    } else {
        return Err(BotError::NoPlayer);
    };

    if let Some(track) = prevTrack {
        let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
        player.play(&track.track).await?;

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> Playing previous track: **[{}]({})**",
                    track.title, track.uri
                ))),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send_interaction(ctx).await?;
    }

    Ok(())
}
