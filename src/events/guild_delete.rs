use poise::serenity_prelude as serenity;
use tracing::info;

use crate::data::Data;
use crate::error::BotError;

pub async fn handle(
    _ctx: &serenity::Context,
    incomplete: &serenity::UnavailableGuild,
    _full: Option<&serenity::Guild>,
    data: &Data,
) -> Result<(), BotError> {
    let mut embed = serenity::CreateEmbed::new()
        .title("📤 Left Guild")
        .color(data.config.colorError)
        .timestamp(serenity::Timestamp::now());

    if let Some(guild) = _full {
        info!("[Guild] Left guild: {} (ID: {})", guild.name, guild.id);
        embed = embed
            .field("Name", &guild.name, true)
            .field("ID", guild.id.to_string(), true);
    } else {
        info!("[Guild] Left guild: {}", incomplete.id);
        embed = embed.field("ID", incomplete.id.to_string(), true);
    }

    crate::utils::logs::send_log(&_ctx.http, &data.config, crate::utils::logs::LogType::Guild, embed).await;
    data.guildPlayers.remove(&incomplete.id);

    Ok(())
}
