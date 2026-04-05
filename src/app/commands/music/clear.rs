use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkVoiceChannel, checkBotVoiceChannel, checkPlayer, checkTracks};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn clear(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? || !checkTracks(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;

    if let Some(mut guildPlayer) = guild_players.get_mut(&guild_id) {
        let count = guildPlayer.queue.len();
        guildPlayer.queue.clear();

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> Cleared **{}** tracks from the queue.",
                    count
                ))),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send_interaction(ctx).await?;
    } else {
        return Err(BotError::NoPlayer);
    }

    Ok(())
}
