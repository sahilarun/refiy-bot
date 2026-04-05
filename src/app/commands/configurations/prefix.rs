use crate::data::Context;
use crate::error::BotError;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    category = "Configurations",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn prefix(
    ctx: Context<'_>,
    #[description = "The new prefix for this server or \"reset\" to use default"]
    new_prefix: String,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(BotError::Other("Must be in a guild".into()))?;
    let db = &ctx.data().database;

    if new_prefix.trim().to_lowercase() == "reset" {
        db.deletePrefix(&guild_id.to_string()).await?;
        
        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new("> The prefix has been reset to default: `.`".to_string())),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
    } else {
        db.setPrefix(&guild_id.to_string(), &new_prefix).await?;

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> The prefix has been set to: `{}`",
                    new_prefix
                ))),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
    }
    let _ = ctx.send(poise::CreateReply::default().content("Updated the prefix.").ephemeral(true)).await;
    Ok(())
}
