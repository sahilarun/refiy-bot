use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::track::{TrackData, TrackLoadData};
use lavalink_rs::model::GuildId;
use tracing::{info, error, warn};
use rand::Rng;
use serde::Deserialize;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::BotError;
use crate::data::GuildPlayer;

const MAX_SAME_ARTIST_IN_ROW: usize = 3;
static KEY_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Deserialize, Debug)]
struct LastFmTrack {
    name: String,
    artist: LastFmArtist,
}

#[derive(Deserialize, Debug)]
struct LastFmArtist {
    name: String,
}

#[derive(Deserialize, Debug)]
struct SimilarTracks {
    track: Vec<LastFmTrack>,
}

#[derive(Deserialize, Debug)]
struct LastFmSimilarResponse {
    similartracks: Option<SimilarTracks>,
    error: Option<u64>,
    message: Option<String>,
}

pub async fn handleAutoplay(
    lava: &LavalinkClient,
    guildId: GuildId,
    guildPlayer: &mut GuildPlayer,
    lastTrack: &TrackData,
    lastFmKeys: &[String],
) -> Result<(), BotError> {
    if !guildPlayer.autoplay {
        return Ok(());
    }

    info!("[Autoplay] Finding recommendations for guild {} based on {}", guildId, lastTrack.info.title);

    if !lastFmKeys.is_empty() {
        match get_lastfm_recommendations(lastTrack, lastFmKeys).await {
            Ok(Some(recommendation)) => {
                info!("[Autoplay] Last.fm recommendation: {} - {}", recommendation.artist.name, recommendation.name);
                if let Ok(Some(_)) = search_and_queue(lava, guildId, guildPlayer, &format!("{} {}", recommendation.artist.name, recommendation.name)).await {
                    return Ok(());
                }
            }
            Ok(None) => info!("[Autoplay] No similar tracks found on Last.fm"),
            Err(e) => error!("[Autoplay] Last.fm error: {}", e),
        }
    }

    info!("[Autoplay] Falling back to search-based recommendation");
    let query = format!("{} {}", lastTrack.info.author, lastTrack.info.title);
    search_and_queue(lava, guildId, guildPlayer, &query).await.map(|_| ())
}

async fn get_lastfm_recommendations(
    lastTrack: &TrackData,
    keys: &[String],
) -> Result<Option<LastFmTrack>, BotError> {
    if keys.is_empty() { return Ok(None); }
    
    let idx = KEY_INDEX.fetch_add(1, Ordering::SeqCst);
    let key = &keys[idx % keys.len()];

    let url = format!(
        "https://ws.audioscrobbler.com/2.0/?method=track.getSimilar&artist={}&track={}&limit=10&autocorrect=1&api_key={}&format=json",
        urlencoding::encode(&lastTrack.info.author),
        urlencoding::encode(&lastTrack.info.title),
        key
    );

    let client = reqwest::Client::new();
    let res = client.get(url).send().await.map_err(|e| BotError::Other(e.to_string()))?;
    let data: LastFmSimilarResponse = res.json().await.map_err(|e| BotError::Other(e.to_string()))?;

    if let Some(err) = data.error {
        warn!("[Autoplay] Last.fm API error {}: {:?}", err, data.message);
        return Ok(None);
    }

    if let Some(similars) = data.similartracks {
        if !similars.track.is_empty() {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..similars.track.len());
            return Ok(Some(similars.track[idx].clone()));
        }
    }

    Ok(None)
}

async fn search_and_queue(
    lava: &LavalinkClient,
    guildId: GuildId,
    guildPlayer: &mut GuildPlayer,
    query: &str,
) -> Result<Option<lavalink_rs::model::track::TrackData>, BotError> {
    let identifier = format!("ytsearch:{}", query);
    let res = lava.load_tracks(guildId, &identifier).await.map_err(|e| BotError::Lavalink(e.to_string()))?;
    
    let tracks = match res.data {
        Some(TrackLoadData::Search(tracks)) => tracks,
        Some(TrackLoadData::Track(track)) => vec![track],
        Some(TrackLoadData::Playlist(playlist)) => playlist.tracks,
        _ => return Ok(None),
    };

    if tracks.is_empty() {
        return Ok(None);
    }

    let mut rng = rand::thread_rng();
    
    let selected_idx = if tracks.len() >= 4 {
        rng.gen_range(1..4)
    } else {
        0
    };

    let track = &tracks[selected_idx];
    
    if would_exceed_artist_limit(guildPlayer, &track.info.author) {
        info!("[Autoplay] Skipping {} due to artist limit", track.info.title);
        let backup = &tracks[0];
        if !would_exceed_artist_limit(guildPlayer, &backup.info.author) {
             return add_track_to_player_queue(guildPlayer, backup.clone());
        }
        return Ok(None);
    }

    add_track_to_player_queue(guildPlayer, track.clone())
}

fn would_exceed_artist_limit(guildPlayer: &GuildPlayer, author: &str) -> bool {
    let recent_authors: Vec<_> = guildPlayer.queue.iter()
        .rev()
        .take(MAX_SAME_ARTIST_IN_ROW - 1)
        .map(|t| t.author.to_lowercase())
        .collect();

    recent_authors.len() == MAX_SAME_ARTIST_IN_ROW - 1 && 
    recent_authors.iter().all(|a| *a == author.to_lowercase())
}

fn add_track_to_player_queue(guildPlayer: &mut GuildPlayer, track: TrackData) -> Result<Option<TrackData>, BotError> {
    let queued = crate::data::QueuedTrack {
        track: track.clone(),
        requester: 0.into(),
        title: track.info.title.clone(),
        author: track.info.author.clone(),
        uri: track.info.uri.clone().unwrap_or_default(),
        duration: track.info.length as u64,
        isStream: track.info.is_stream,
        artworkUrl: track.info.artwork_url.clone(),
    };
    guildPlayer.queue.push_back(queued);
    Ok(Some(track))
}

impl Clone for LastFmTrack {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            artist: LastFmArtist { name: self.artist.name.clone() },
        }
    }
}
