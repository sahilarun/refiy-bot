use tracing::info;
use lavalink_rs::prelude::TrackLoadData;

use crate::data::{Context, GuildPlayer, QueuedTrack};
use crate::error::BotError;
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkCooldown, ensureConnected, getAuthorVoiceChannel};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    aliases("p"),
    category = "Music"
)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Song name or URL"]
    #[rest]
    query: String,
) -> Result<(), BotError> {
    ctx.defer().await?;

    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkCooldown(ctx).await? {
        return Ok(());
    }

    info!("[Play] Command called with query: {}", query);
    let (guild_id, voice_channel) = getAuthorVoiceChannel(ctx).await?;
    info!("[Play] User in voice channel {} in guild {}", voice_channel, guild_id);

    ensureConnected(ctx, guild_id, voice_channel).await?;

    let lava = &ctx.data().lavalink;
    let queryStr = if query.starts_with("http://") || query.starts_with("https://") {
        query.clone()
    } else {
        format!("ytsearch:{}", query)
    };

    let loadResult = lava
        .load_tracks(guild_id.get(), &queryStr)
        .await
        .map_err(|e| BotError::Lavalink(e.to_string()))?;

    let tracks = match loadResult.data {
        Some(TrackLoadData::Track(track)) => vec![track],
        Some(TrackLoadData::Search(results)) => {
            if results.is_empty() {
                ctx.send(poise::CreateReply::default().content("❌ No results found.").ephemeral(true)).await?;
                return Ok(());
            }
            vec![results.into_iter().next().ok_or(BotError::Other("Empty search result".into()))?]
        }
        Some(TrackLoadData::Playlist(playlist)) => {
            let plTracks = playlist.tracks;
            if plTracks.is_empty() {
                ctx.send(poise::CreateReply::default().content("❌ Empty playlist.").ephemeral(true)).await?;
                return Ok(());
            }

            let mut player = ctx
                .data()
                .guildPlayers
                .entry(guild_id)
                .or_insert_with(|| GuildPlayer::new(ctx.data().config.defaultVolume));

            player.textChannelId = Some(ctx.channel_id().get());

            let mut startPlaying = false;
            let mut firstTrack = None;

            if player.currentTrack.is_none() {
                startPlaying = true;
                let first = plTracks.first().unwrap();
                firstTrack = Some(first.clone());
                player.currentTrack = Some(QueuedTrack {
                    track: first.clone(),
                    requester: ctx.author().id,
                    title: first.info.title.clone(),
                    author: first.info.author.clone(),
                    uri: first.info.uri.clone().unwrap_or_default(),
                    duration: first.info.length as u64,
                    isStream: first.info.is_stream,
                    artworkUrl: first.info.artwork_url.clone(),
                });

                for t in plTracks.iter().skip(1) {
                    player.queue.push_back(QueuedTrack {
                        track: t.clone(),
                        requester: ctx.author().id,
                        title: t.info.title.clone(),
                        author: t.info.author.clone(),
                        uri: t.info.uri.clone().unwrap_or_default(),
                        duration: t.info.length as u64,
                        isStream: t.info.is_stream,
                        artworkUrl: t.info.artwork_url.clone(),
                    });
                }
            } else {
                for t in &plTracks {
                    player.queue.push_back(QueuedTrack {
                        track: t.clone(),
                        requester: ctx.author().id,
                        title: t.info.title.clone(),
                        author: t.info.author.clone(),
                        uri: t.info.uri.clone().unwrap_or_default(),
                        duration: t.info.length as u64,
                        isStream: t.info.is_stream,
                        artworkUrl: t.info.artwork_url.clone(),
                    });
                }
            }

            if startPlaying {
                let player_ctx = lava.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
                let _ = player_ctx.play(&firstTrack.unwrap()).await;
            }

            let components = crate::components::v2::createEmbedV2(
                "Added to Queue",
                &format!("Added **{}** by **{}** to the queue [ `{}` ] - Requested by <@{}>", 
                    plTracks.first().map(|t| t.info.title.as_str()).unwrap_or(""),
                    plTracks.first().map(|t| t.info.author.as_str()).unwrap_or(""),
                    crate::utils::formatDuration(plTracks.iter().map(|t| t.info.length).sum()),
                    ctx.author().id
                ),
                None,
            );
            
            let payload = crate::components::v2::V2MessagePayload::new(components);
            payload.send_interaction(ctx).await?;

            return Ok(());
        }
        _ => {
            ctx.send(poise::CreateReply::default().content("Could not load the track.").ephemeral(true)).await?;
            return Ok(());
        }
    };

    if tracks.is_empty() {
        ctx.send(poise::CreateReply::default().content("No tracks found.").ephemeral(true)).await?;
        return Ok(());
    }

    let track = tracks.first().unwrap();
    let mut isPlaying = false;
    let player_context = lava.get_player_context(guild_id);
    
    {
        let mut player = ctx
            .data()
            .guildPlayers
            .entry(guild_id)
            .or_insert_with(|| GuildPlayer::new(ctx.data().config.defaultVolume));

        player.textChannelId = Some(ctx.channel_id().get());
        
        if player.currentTrack.is_none() {
            player.currentTrack = Some(QueuedTrack {
                track: track.clone(),
                requester: ctx.author().id,
                title: track.info.title.clone(),
                author: track.info.author.clone(),
                uri: track.info.uri.clone().unwrap_or_default(),
                duration: track.info.length as u64,
                isStream: track.info.is_stream,
                artworkUrl: track.info.artwork_url.clone(),
            });
        } else {
            isPlaying = true;
            player.queue.push_back(QueuedTrack {
                track: track.clone(),
                requester: ctx.author().id,
                title: track.info.title.clone(),
                author: track.info.author.clone(),
                uri: track.info.uri.clone().unwrap_or_default(),
                duration: track.info.length as u64,
                isStream: track.info.is_stream,
                artworkUrl: track.info.artwork_url.clone(),
            });
        }
    }

    if !isPlaying {
        let player_ctx = player_context.ok_or(BotError::NoPlayer)?;
        player_ctx.play(&track).await.map_err(|e| BotError::Lavalink(e.to_string()))?;
        
        let components = crate::components::v2::createEmbedV2(
            "Added to Queue",
            &format!("Added **{}** by **{}** to the queue [ `{}` ] - Requested by <@{}>", 
                track.info.title, 
                track.info.author, 
                crate::utils::formatDuration(track.info.length),
                ctx.author().id
            ),
            track.info.artwork_url.as_deref()
        );
        
        crate::components::v2::V2MessagePayload::new(components)
            .send_interaction(ctx)
            .await?;
    } else {
        let components = crate::components::v2::createEmbedV2(
            "Added to Queue",
            &format!("**[{}]({})**", track.info.title, track.info.uri.as_ref().unwrap_or(&"".to_string())),
            track.info.artwork_url.as_deref()
        );
        
        crate::components::v2::V2MessagePayload::new(components)
            .send_interaction(ctx)
            .await?;
    }

    Ok(())
}
