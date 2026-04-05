use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkVoiceChannel, checkBotVoiceChannel, checkPlayer, checkTracks};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("s", "next"))]
pub async fn skip(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? || !checkTracks(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
    player.stop_now().await?;

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("> The current track has been **skipped**.".to_string())),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
