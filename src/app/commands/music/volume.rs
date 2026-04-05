use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("vol"))]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Volume level (0-200)"] 
    #[min = 0] #[max = 200]
    level: Option<u16>
) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;

    let (content, val) = if let Some(vol) = level {
        player.set_volume(vol).await?;
        if let Some(mut guildPlayer) = ctx.data().guildPlayers.get_mut(&guild_id) {
            guildPlayer.volume = vol;
        }
        (format!("> Volume has been set to **{}%**.", vol), vol)
    } else {
        let currentVol = if let Some(guildPlayer) = ctx.data().guildPlayers.get(&guild_id) {
            guildPlayer.volume
        } else {
            100
        };
        (format!("> The current volume is **{}%**.", currentVol), currentVol)
    };

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(content)),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
