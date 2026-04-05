use crate::error::BotError;
use crate::data::Context;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command, guild_only, category = "Informations")]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), BotError> {
    let guildId = ctx.guild_id().unwrap();
    let db = &ctx.data().database;

    let leaderboard = db.getLeaderboard(&guildId.to_string()).await?;

    if leaderboard.is_empty() {
        ctx.send(poise::CreateReply::default().content("❌ No statistics available for this guild yet.")).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for (i, stats) in leaderboard.iter().enumerate() {
        desc.push_str(&format!("{}. <@{}> - **{}** tracks\n", i + 1, stats.userId, stats.playCount));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Music Leaderboard")
        .description(desc)
        .color(ctx.data().config.colorPrimary)
        .footer(serenity::CreateEmbedFooter::new(format!("Top listeners in {}", ctx.guild().unwrap().name)));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
