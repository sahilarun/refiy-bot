use crate::error::BotError;
use crate::data::Context;
use crate::utils::formatDuration;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("sk"))]
pub async fn seek(
    ctx: Context<'_>,
    #[description = "Time to seek to (e.g. 1:30 or 90)"]
    time: String
) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
    
    let ms = parseTime(&time).ok_or(BotError::Other("> Invalid time format. Use seconds (90) or mm:ss (1:30).".into()))?;

    if let Some(guild_player) = ctx.data().guildPlayers.get(&guild_id) {
        if let Some(current) = &guild_player.currentTrack {
            if ms > current.track.info.length {
                return Err(BotError::Other(format!("> Seek time exceeds track duration ({}s).", current.track.info.length / 1000).into()));
            }
        }
    }

    player.set_position(std::time::Duration::from_millis(ms)).await?;

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "> Seeked to **{}**.",
                formatDuration(ms)
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}

fn parseTime(s: &str) -> Option<u64> {
    if let Ok(seconds) = s.parse::<u64>() {
        return Some(seconds * 1000);
    }

    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 2 {
        let minutes = parts[0].parse::<u64>().ok()?;
        let seconds = parts[1].parse::<u64>().ok()?;
        return Some((minutes * 60 + seconds) * 1000);
    }

    None
}
