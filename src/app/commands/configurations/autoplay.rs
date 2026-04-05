use crate::data::Context;
use crate::error::BotError;
use crate::components::v2::{Container, V2Component, TextDisplay, V2MessagePayload};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    category = "Configurations",
    aliases("auto", "ap"),
    required_permissions = "MANAGE_GUILD"
)]
pub async fn autoplay(
    ctx: Context<'_>,
    #[description = "Enable or disable autoplay (on/off)"]
    state: Option<bool>,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(BotError::Other("Must be in a guild".into()))?;
    let db = &ctx.data().database;

    let current = db.getAutoplay(&guild_id.to_string()).await.unwrap_or(false);
    
    let new_state = match state {
        Some(s) => s,
        None => !current,
    };

    db.setAutoplay(&guild_id.to_string(), new_state).await?;

    if let Some(mut player) = ctx.data().guildPlayers.get_mut(&guild_id) {
        player.autoplay = new_state;
    }

    let status_text = if new_state { "enabled" } else { "disabled" };
    
    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "> Autoplay has been **{}** for this server.",
                status_text
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
    
    let _ = ctx.send(poise::CreateReply::default().content("Updated autoplay settings.").ephemeral(true)).await;
    Ok(())
}
