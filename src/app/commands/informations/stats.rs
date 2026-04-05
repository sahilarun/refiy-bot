use poise::serenity_prelude as serenity;
use sysinfo::System;
use crate::data::Context;
use crate::error::BotError;
use crate::utils::formatDuration;

#[poise::command(slash_command, prefix_command, category = "Information")]
pub async fn stats(ctx: Context<'_>) -> Result<(), BotError> {
    let data = ctx.data();
    let uptime = chrono::Utc::now().timestamp() as u64 - data.startTime;
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let ram_used = sys.used_memory() / 1024 / 1024;
    let ram_total = sys.total_memory() / 1024 / 1024;
    
    let pid = std::process::id();
    let process_ram = if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
        process.memory() / 1024 / 1024
    } else {
        0
    };

    let guild_count = ctx.cache().guild_count();
    let user_count = ctx.cache().user_count();
    
    let lava = &data.lavalink;
    let nodes = &lava.nodes;
    let node_count = nodes.len();
    let playing_count = data.guildPlayers.iter().filter(|r| r.value().currentTrack.is_some()).count();

    let embed = serenity::CreateEmbed::new()
        .title("Hearth Statistics")
        .color(data.config.colorPrimary)
        .field("📊 General", format!("**Guilds:** {}\n**Users:** {}\n**Playing:** {}", guild_count, user_count, playing_count), true)
        .field("⚙️ System", format!("**Uptime:** {}\n**RAM:** {}MB / {}MB\n**Process:** {}MB", formatDuration(uptime), ram_used, ram_total, process_ram), true)
        .field("🌐 Network", format!("**Lavalink Nodes:** {}\n**Gateway Latency:** {:?}", node_count, ctx.ping().await), true)
        .footer(serenity::CreateEmbedFooter::new(format!("Requested by {}", ctx.author().name)))
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
