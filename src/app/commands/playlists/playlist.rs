use poise::serenity_prelude as serenity;
use lavalink_rs::prelude::TrackLoadData;
use lavalink_rs::model::track::TrackInfo;

use crate::data::{Context, GuildPlayer, QueuedTrack};
use crate::error::BotError;
use crate::utils::{ensureConnected, getAuthorVoiceChannel};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    category = "Playlists",
    subcommands("create", "delete", "list", "add", "remove", "load", "info")
)]
pub async fn playlist(_ctx: Context<'_>) -> Result<(), BotError> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
) -> Result<(), BotError> {
    let db = &ctx.data().database;
    let user_id = ctx.author().id.to_string();

    db.createPlaylist(&user_id, &name).await?;

    let embed = serenity::CreateEmbed::new()
        .title("Playlist Created")
        .description(format!("Created a new playlist: **{}**", name))
        .color(ctx.data().config.colorSuccess);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
) -> Result<(), BotError> {
    let db = &ctx.data().database;
    let user_id = ctx.author().id.to_string();

    if db.deletePlaylist(&user_id, &name).await? {
        let embed = serenity::CreateEmbed::new()
            .title("Playlist Deleted")
            .description(format!("Deleted playlist: **{}**", name))
            .color(ctx.data().config.colorSuccess);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(poise::CreateReply::default().content("❌ Playlist not found.").ephemeral(true)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), BotError> {
    let db = &ctx.data().database;
    let user_id = ctx.author().id.to_string();

    let playlists = db.getPlaylists(&user_id).await?;

    if playlists.is_empty() {
        ctx.send(poise::CreateReply::default().content("❌ You have no playlists.")).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for (i, pl) in playlists.iter().enumerate() {
        desc.push_str(&format!("{}. **{}** (Created: {})\n", i + 1, pl.name, pl.createdAt));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Your Playlists")
        .description(desc)
        .color(ctx.data().config.colorPrimary);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
    #[description = "URL or search query"]
    #[rest]
    query: String,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let db = &ctx.data().database;
    let lava = &ctx.data().lavalink;
    let user_id = ctx.author().id.to_string();

    let playlist_id = db.getPlaylistId(&user_id, &name).await?.ok_or(BotError::Other("Playlist not found".into()))?;

    let query_str = if query.starts_with("http") { query } else { format!("ytsearch:{}", query) };
    let load_result = lava.load_tracks(ctx.guild_id().unwrap().get(), &query_str).await.map_err(|e| BotError::Lavalink(e.to_string()))?;

    let track = match load_result.data {
        Some(TrackLoadData::Track(t)) => t,
        Some(TrackLoadData::Search(list)) => list.into_iter().next().ok_or(BotError::Other("No tracks found".into()))?,
        _ => return Err(BotError::Other("Unsupported track type".into())),
    };

    let info_json = serde_json::to_string(&track.info).map_err(|e| BotError::Other(e.to_string()))?;
    db.addToPlaylist(&playlist_id, &track.info.uri.as_ref().unwrap_or(&"".into()), &info_json).await?;

    let embed = serenity::CreateEmbed::new()
        .title("Track Added")
        .description(format!("Added **{}** to playlist **{}**", track.info.title, name))
        .color(ctx.data().config.colorSuccess);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
    #[description = "Index of the track to remove (starting from 1)"]
    index: usize,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let db = &ctx.data().database;
    let user_id = ctx.author().id.to_string();

    let playlist_id = db.getPlaylistId(&user_id, &name).await?.ok_or(BotError::Other("Playlist not found".into()))?;

    if db.removeFromPlaylist(&playlist_id, index - 1).await? {
        let embed = serenity::CreateEmbed::new()
            .title("Track Removed")
            .description(format!("Removed track #{} from playlist **{}**", index, name))
            .color(ctx.data().config.colorSuccess);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(poise::CreateReply::default().content("❌ Invalid track index.").ephemeral(true)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn load(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let db = &ctx.data().database;
    let lava = &ctx.data().lavalink;
    let user_id = ctx.author().id.to_string();

    let (guild_id, voice_channel) = getAuthorVoiceChannel(ctx).await?;
    ensureConnected(ctx, guild_id, voice_channel).await?;

    let playlist_id = db.getPlaylistId(&user_id, &name).await?.ok_or(BotError::Other("Playlist not found".into()))?;
    let tracks = db.getPlaylistTracks(&playlist_id).await?;

    if tracks.is_empty() {
        ctx.send(poise::CreateReply::default().content("❌ This playlist is empty.")).await?;
        return Ok(());
    }

    let mut first_track = None;
    let mut added_count = 0;

    {
        let mut player = ctx.data().guildPlayers.entry(guild_id).or_insert_with(|| GuildPlayer::new(ctx.data().config.defaultVolume));
        player.textChannelId = Some(ctx.channel_id().get());

        for t_data in tracks {
            if let Ok(_info) = serde_json::from_str::<TrackInfo>(&t_data.info) {
                let load_result = lava.load_tracks(guild_id.get(), &t_data.url).await;
                if let Ok(load_data) = load_result {
                    if let Some(TrackLoadData::Track(track)) = load_data.data {
                        let queued = QueuedTrack {
                            track: track.clone(),
                            requester: ctx.author().id,
                            title: track.info.title.clone(),
                            author: track.info.author.clone(),
                            uri: track.info.uri.clone().unwrap_or_default(),
                            duration: track.info.length,
                            isStream: track.info.is_stream,
                            artworkUrl: track.info.artwork_url.clone(),
                        };

                        if player.currentTrack.is_none() {
                            player.currentTrack = Some(queued);
                            first_track = Some(track);
                        } else {
                            player.queue.push_back(queued);
                        }
                        added_count += 1;
                    }
                }
            }
        }
    }

    if let Some(track) = first_track {
        let pc = lava.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
        pc.play(&track).await.map_err(|e| BotError::Lavalink(e.to_string()))?;
    }

    let embed = serenity::CreateEmbed::new()
        .title("Playlist Loaded")
        .description(format!("Loaded **{}** songs from playlist **{}**", added_count, name))
        .color(ctx.data().config.colorPrimary);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "Name of the playlist"]
    name: String,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let db = &ctx.data().database;
    let user_id = ctx.author().id.to_string();

    let playlist_id = db.getPlaylistId(&user_id, &name).await?
        .ok_or(BotError::Other("Playlist not found.".into()))?;

    let tracks = db.getPlaylistTracks(&playlist_id).await?;

    if tracks.is_empty() {
        ctx.send(poise::CreateReply::default().content("❌ This playlist is empty.")).await?;
        return Ok(());
    }

    let mut pages = Vec::new();
    let chunk_size = 10;
    let num_pages = (tracks.len() as f32 / chunk_size as f32).ceil() as usize;

    for i in 0..num_pages {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, tracks.len());
        let mut desc = String::new();

        for (j, t) in tracks[start..end].iter().enumerate() {
            if let Ok(_info) = serde_json::from_str::<TrackInfo>(&t.info) {
                desc.push_str(&format!("{}. **[{}]({})** - `{}`\n", start + j + 1, _info.title, t.url, _info.author));
            } else {
                desc.push_str(&format!("{}. **Unknown Track** - `{}`\n", start + j + 1, t.url));
            }
        }

        let embed = serenity::CreateEmbed::new()
            .title(format!("Playlist: {}", name))
            .description(desc)
            .color(ctx.data().config.colorPrimary)
            .footer(serenity::CreateEmbedFooter::new(format!("Page {}/{} | {} tracks", i + 1, num_pages, tracks.len())));
        
        pages.push(embed);
    }

    if pages.len() == 1 {
        ctx.send(poise::CreateReply::default().embed(pages[0].clone())).await?;
    } else {
        crate::utils::paginate(ctx, pages).await?;
    }

    Ok(())
}
