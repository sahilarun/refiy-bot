use poise::serenity_prelude as serenity;
use tracing::info;

use crate::data::Data;
use crate::error::BotError;

pub async fn handle(
    ctx: &serenity::Context,
    guild: &serenity::Guild,
    is_new: Option<bool>,
    data: &Data,
) -> Result<(), BotError> {
    if is_new.unwrap_or(false) {
        info!(
            "[Guild] Joined new guild: {} (ID: {}, Members: {})",
            guild.name, guild.id, guild.member_count
        );

        let embed = serenity::CreateEmbed::new()
            .title("📥 Joined Guild")
            .field("Name", &guild.name, true)
            .field("ID", guild.id.to_string(), true)
            .field("Members", guild.member_count.to_string(), true)
            .color(data.config.colorSuccess)
            .timestamp(serenity::Timestamp::now());
        crate::utils::logs::send_log(&ctx.http, &data.config, crate::utils::logs::LogType::Guild, embed).await;
    }
    Ok(())
}
