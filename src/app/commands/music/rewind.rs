use crate::error::BotError;
use crate::data::Context;
use crate::utils::formatDuration;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn rewind(
    ctx: Context<'_>,
    #[description = "Seconds to rewind (default 10)"]
    seconds: Option<u64>
) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
    let p = player.get_player().await?;
    
    let rewind_ms = seconds.unwrap_or(10) * 1000;
    let new_position = if p.state.position > rewind_ms { p.state.position - rewind_ms } else { 0 };

    player.set_position(std::time::Duration::from_millis(new_position)).await?;

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "> Rewound to **{}**.",
                formatDuration(new_position)
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
