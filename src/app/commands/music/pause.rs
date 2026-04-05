use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn pause(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
    player.set_pause(true).await?;

    if let Some(mut guildPlayer) = ctx.data().guildPlayers.get_mut(&guild_id) {
        guildPlayer.paused = true;
    }

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("> The player has been **paused**.".to_string())),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
