use crate::error::BotError;
use crate::data::Context;
use crate::utils::pagination::paginate_v2;
use crate::components::v2::{Container, TextDisplay, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer, checkTracks};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("q"))]
pub async fn queue(ctx: Context<'_>) -> Result<(), BotError> {
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? || !checkTracks(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;

    let guild_player = guild_players.get(&guild_id).ok_or(BotError::NoPlayer)?;
    
    let total_tracks = guild_player.queue.len() + (if guild_player.currentTrack.is_some() { 1 } else { 0 });

    if total_tracks == 0 {
        return Err(BotError::Other("> The queue is currently empty.".into()));
    }

    let mut pages = Vec::new();
    let queue_list: Vec<_> = guild_player.queue.iter().collect();
    let chunk_size = 10;
    
    let num_pages = (queue_list.len() as f32 / chunk_size as f32).ceil() as usize;
    let num_pages = if num_pages == 0 && guild_player.currentTrack.is_some() { 1 } else { num_pages };

    for p in 0..num_pages {
        let mut description = String::new();
        
        if p == 0 {
            if let Some(current) = &guild_player.currentTrack {
                description.push_str("**Now Playing:**\n");
                description.push_str(&format!("1. **[{}]({})** - `{}` - <@{}>\n\n", current.title, current.uri, current.author, current.requester));
            }
            if !queue_list.is_empty() {
                description.push_str("**Upcoming:**\n");
            }
        }

        let start = p * chunk_size;
        let end = std::cmp::min(start + chunk_size, queue_list.len());

        for i in start..end {
            let track = queue_list[i];
            description.push_str(&format!("{}. **[{}]({})** - `{}` - <@{}>\n", i + 2, track.title, track.uri, track.author, track.requester));
        }

        let header = format!("Queue ({} tracks) | Repeat Mode: {:?}", total_tracks, guild_player.repeatMode);
        
        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "**{}**\n\n{}",
                    header,
                    description
                ))),
            ]))
        ];
        
        pages.push(components);
    }

    paginate_v2(ctx, pages).await?;

    Ok(())
}
