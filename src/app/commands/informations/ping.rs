#![allow(non_snake_case)]
use std::time::Instant;

use poise::serenity_prelude::{self as serenity, CreateEmbed};

use crate::data::Context;
use crate::error::BotError;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    category = "Informations"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), BotError> {
    let start = Instant::now();
    let msg = ctx.say("🏓 Pinging...").await?;

    let rtt = start.elapsed().as_millis();

    let gatewayLatency = {
        let shardManager = ctx.framework().shard_manager();
        let runners = shardManager.runners.lock().await;
        runners
            .values()
            .next()
            .and_then(|runner| runner.latency)
            .map(|d| d.as_millis())
    };

    let gatewayStr = match gatewayLatency {
        Some(ms) => format!("`{}ms`", ms),
        None => "`N/A`".to_string(),
    };

    let embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .color(ctx.data().config.colorPrimary)
        .field("Gateway Latency", &gatewayStr, true)
        .field("API Latency", format!("`{}ms`", rtt), true)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Requested by {}",
            ctx.author().name
        )));

    msg.edit(ctx, poise::CreateReply::default().content("").embed(embed))
        .await?;

    Ok(())
}
