use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn like(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();
    let user_id = ctx.author().id.to_string();
    let guild_players = &ctx.data().guildPlayers;
    let database = &ctx.data().database;

    let guild_player = guild_players.get(&guild_id).ok_or(BotError::NoPlayer)?;
    let current = guild_player.currentTrack.as_ref().ok_or(BotError::Other("> Nothing is playing right now.".into()))?;

    let track_id = &current.track.info.identifier;
    let is_liked = database.isTrackLiked(&user_id, track_id).await?;

    let message = if is_liked {
        database.removeFromLikedSongs(&user_id, track_id).await?;
        format!("> Removed **[{}]({})** from your liked songs.", current.title, current.uri)
    } else {
        database.addToLikedSongs(
            &user_id,
            track_id,
            &current.title,
            &current.author,
            &current.uri,
            current.track.info.artwork_url.clone(),
            current.duration,
            current.track.info.is_stream
        ).await?;
        format!("> Added **[{}]({})** to your liked songs.", current.title, current.uri)
    };

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(message)),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
