use poise::serenity_prelude as serenity;
use tracing::info;
use crate::data::Data;
use crate::error::BotError;

pub async fn handle(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    data: &Data,
) -> Result<(), BotError> {
    if msg.author.bot {
        return Ok(());
    }

    let bot_id = ctx.cache.current_user().id;
    let mention = format!("<@{}>", bot_id);
    let mention_nick = format!("<@!{}>", bot_id);

    if msg.content == mention || msg.content == mention_nick {
        let guild_id = msg.guild_id.map(|id| id.to_string()).unwrap_or_default();
        let prefix = if !guild_id.is_empty() {
            data.database.getPrefix(&guild_id).await.unwrap_or(data.config.prefix.clone())
        } else {
            data.config.prefix.clone()
        };

        let embed = serenity::CreateEmbed::new()
            .title("Hearth — Prefix Information")
            .description(format!(
                "👋 Hey! My prefix for this server is `{}`\n\nYou can also use my **slash commands** by typing `/`!",
                prefix
            ))
            .color(data.config.colorSuccess)
            .thumbnail(ctx.cache.current_user().face())
            .footer(serenity::CreateEmbedFooter::new(format!("Requested by {}", msg.author.name)));

        msg.channel_id.send_message(&ctx.http, serenity::CreateMessage::default().embed(embed)).await.map_err(|e| {
            info!("[Events] Failed to send mention reply: {}", e);
            BotError::Serenity(e)
        })?;
    }

    Ok(())
}
