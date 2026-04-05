use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, V2Component, TextDisplay, V2MessagePayload};
use crate::utils::checks::{checkNodes, checkVoiceChannel};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    category = "Configurations",
    rename = "247",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn twentyFourSeven(
    ctx: Context<'_>,
    #[description = "Enable or disable 24/7 mode"]
    enabled: Option<bool>,
) -> Result<(), BotError> {
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().ok_or(BotError::Other("Must be in a guild".into()))?;
    let db = &ctx.data().database;

    if enabled.is_none() {
        let (current_status, _, _) = db.get247Mode(&guild_id.to_string()).await?.unwrap_or((false, None, None));
        let status_text = if current_status { "Enabled" } else { "Disabled" };

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!("> Current Status: **{}**", status_text))),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
        let _ = ctx.send(poise::CreateReply::default().content("").ephemeral(true)).await;
        return Ok(());
    }

    let new_mode = enabled.unwrap();

    if !new_mode {
        db.set247Mode(&guild_id.to_string(), false, None, None).await?;
        if let Some(mut player) = ctx.data().guildPlayers.get_mut(&guild_id) {
            player.mode247 = false;
        }

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new("> **24/7 Mode** has been **disabled**.".to_string())),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
    } else {
        let voice_channel = ctx.guild()
            .and_then(|g| g.voice_states.get(&ctx.author().id).and_then(|vs| vs.channel_id));
        
        if voice_channel.is_none() {
             ctx.send(poise::CreateReply::default().content("❌ You must be in a voice channel to enable 24/7 mode.").ephemeral(true)).await?;
             return Ok(());
        }

        let text_channel = ctx.channel_id();

        db.set247Mode(
            &guild_id.to_string(),
            true,
            voice_channel.map(|c| c.to_string()).as_deref(),
            Some(&text_channel.to_string())
        ).await?;

        if let Some(mut player) = ctx.data().guildPlayers.get_mut(&guild_id) {
            player.mode247 = true;
        }

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new("> **24/7 Mode** has been **enabled**.".to_string())),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
    }

    let _ = ctx.send(poise::CreateReply::default().content("").ephemeral(true)).await;
    Ok(())
}
