use std::sync::atomic::Ordering;
use poise::serenity_prelude as serenity;
use crate::data::Context;
use crate::error::BotError;

#[poise::command(slash_command, prefix_command, category = "Information")]
pub async fn nodes(ctx: Context<'_>) -> Result<(), BotError> {
    let lava = &ctx.data().lavalink;
    let nodes = &lava.nodes;
    
    let mut embed = serenity::CreateEmbed::new()
        .title("Lavalink Nodes")
        .color(ctx.data().config.colorPrimary)
        .timestamp(serenity::Timestamp::now());

    if nodes.is_empty() {
        embed = embed.description("❌ No Lavalink nodes connected.");
    } else {
        for node in nodes.iter() {
            let cpu = node.cpu.load();
            let memory = node.memory.load();
            let status = if node.is_running.load(Ordering::Relaxed) { "🟢 Ready" } else { "🔴 Offline" };
            
            let info = format!(
                "**Status:** {}\n**CPU:** {:.2}%\n**RAM:** {}MB / {}MB\n**Players:** (fetching...)\n**Uptime:** (unknown)",
                status,
                cpu.lavalink_load * 100.0,
                memory.used / 1024 / 1024,
                memory.reservable / 1024 / 1024,
            );
            
            embed = embed.field(format!("🖥️ Node: {}", node.id), info, false);
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
