use crate::error::BotError;
use crate::data::Context;
use poise::serenity_prelude as serenity;
use sysinfo::System;

#[poise::command(slash_command, prefix_command, category = "Informations")]
pub async fn about(ctx: Context<'_>) -> Result<(), BotError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let pid = sysinfo::get_current_pid().map_err(|e| BotError::Other(e.to_string()))?;
    let process = sys.process(pid).ok_or(BotError::Other("Could not find current process".into()))?;
    
    let memory_usage = process.memory() / 1024 / 1024;
    let uptime = process.run_time();

    let embed = serenity::CreateEmbed::new()
        .title("About Hearth")
        .description("Hearth is a high-quality Discord music bot rewritten in Rust for maximum performance and reliability.")
        .field("Version", "0.1.0", true)
        .field("Framework", "Poise / Serenity", true)
        .field("Runtime", "Tokio", true)
        .field("Memory Usage", format!("{} MB", memory_usage), true)
        .field("Process Uptime", format!("{}s", uptime), true)
        .field("Developer", "<@573727266643640346>", true)
        .field("Links", "[Support Server](https://discord.gg/hearth)\n[Invite Bot](https://discord.com/api/oauth2/authorize?client_id=123456789&permissions=8&scope=bot%20applications.commands)", false)
        .color(ctx.data().config.colorPrimary)
        .footer(serenity::CreateEmbedFooter::new("Made with ❤️ in Rust"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
