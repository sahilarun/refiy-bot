use crate::error::BotError;
use crate::data::Context;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command, category = "Informations")]
pub async fn new(ctx: Context<'_>) -> Result<(), BotError> {
    let embed = serenity::CreateEmbed::new()
        .title("What's New in Hearth?")
        .description("We've recently rewritten the entire bot in **Rust** for better performance!")
        .field("Performance", "Lightning-fast command execution and lower latency audio streaming.", false)
        .field("Audio Filters", "Added 12+ new filters including Karaoke, 8D, and Bassboost.", false)
        .field("Unified UI", "Interactive buttons for music control and a new pagination system.", false)
        .field("Lavalink v4", "Full support for the latest Lavalink features and stability fixes.", false)
        .color(ctx.data().config.colorPrimary)
        .footer(serenity::CreateEmbedFooter::new("Version 0.1.0-rust"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
