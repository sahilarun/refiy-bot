use axum::{Router, routing::{get, post}, extract::{Path, State, Query}, Json, response::IntoResponse};
use std::sync::Arc;
use serde::Deserialize;
use serde_json::json;
use crate::data::{Data, RepeatMode};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub guildId: Option<String>,
}

#[derive(Deserialize)]
pub struct LikeRequest {
    pub userId: String,
    pub uri: String,
    pub action: String,
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct RecentQuery {
    pub limit: Option<usize>,
    pub guildId: Option<String>,
}

pub fn routes() -> Router<Arc<Data>> {
    Router::new()
        .route("/status/{guild_id}", get(get_player_status))
        .route("/search", get(search_tracks))
        .route("/liked/{user_id}", get(get_liked_songs))
        .route("/like", post(handle_like))
        .route("/recent/{user_id}", get(get_recent_tracks))
}

async fn get_player_status(
    Path(guild_id): Path<String>,
    State(data): State<Arc<Data>>,
) -> impl IntoResponse {
    let guild_id_u64 = match guild_id.parse::<u64>() {
        Ok(id) => id,
        Err(_) => return Json::<serde_json::Value>(json!({"error": "Invalid guild ID"})).into_response(),
    };

    let guild_id_serenity = poise::serenity_prelude::GuildId::new(guild_id_u64);
    let player_context = data.lavalink.get_player_context(guild_id_serenity);

    if let Some(player) = player_context {
        let internal_player = data.guildPlayers.get(&guild_id_serenity);
        
        let mut queue = Vec::new();
        let mut current = None;
        let mut repeat_mode = 0;
        let mut autoplay = false;
        let mut volume = 100;

        if let Some(ip) = internal_player {
            volume = ip.volume;
            autoplay = ip.autoplay;
            repeat_mode = match ip.repeatMode {
                RepeatMode::Off => 0,
                RepeatMode::Track => 1,
                RepeatMode::Queue => 2,
            };

            if let Some(ct) = &ip.currentTrack {
                current = Some(json!({
                    "title": ct.title,
                    "author": ct.author,
                    "duration": ct.duration,
                    "uri": ct.uri,
                    "artwork": ct.artworkUrl,
                    "isStream": ct.isStream,
                }));
            }

            for track in ip.queue.iter().take(20) {
                queue.push(json!({
                    "title": track.title,
                    "author": track.author,
                    "duration": track.duration,
                    "uri": track.uri,
                    "artwork": track.artworkUrl,
                    "requester": track.requester.to_string(),
                }));
            }
        }

        Json(json!({
            "connected": true,
            "playing": !player.get_player().await.map(|p| p.paused).unwrap_or(true),
            "paused": player.get_player().await.map(|p| p.paused).unwrap_or(false),
            "volume": volume,
            "position": player.get_player().await.map(|p| p.state.position).unwrap_or(0),
            "current": current,
            "queue": queue,
            "repeatMode": repeat_mode,
            "autoplay": autoplay,
        })).into_response()
    } else {
        Json(json!({
            "connected": false,
            "playing": false,
            "paused": false,
            "current": null,
            "queue": [],
            "volume": 0,
            "position": 0,
            "repeatMode": 0,
            "autoplay": false,
        })).into_response()
    }
}

async fn search_tracks(
    Query(query): Query<SearchQuery>,
    State(data): State<Arc<Data>>,
) -> impl IntoResponse {
    let guild_id = query.guildId.as_ref().and_then(|id| id.parse::<u64>().ok()).unwrap_or(0);
    let result = match data.lavalink.load_tracks(guild_id, &query.q).await {
        Ok(res) => res,
        Err(e) => return Json::<serde_json::Value>(json!({"error": e.to_string()})).into_response(),
    };

    let mut tracks = Vec::new();
    if let Some(load_data) = result.data {
        match load_data {
            lavalink_rs::model::track::TrackLoadData::Track(t) => {
                tracks.push(json!({
                    "title": t.info.title,
                    "author": t.info.author,
                    "duration": t.info.length,
                    "uri": t.info.uri,
                    "artwork": t.info.artwork_url,
                    "isStream": t.info.is_stream,
                }));
            }
            lavalink_rs::model::track::TrackLoadData::Search(list) => {
                for t in list.into_iter().take(10) {
                    tracks.push(json!({
                        "title": t.info.title,
                        "author": t.info.author,
                        "duration": t.info.length,
                        "uri": t.info.uri,
                        "artwork": t.info.artwork_url,
                        "isStream": t.info.is_stream,
                    }));
                }
            }
            lavalink_rs::model::track::TrackLoadData::Playlist(pl) => {
                for t in pl.tracks.into_iter().take(10) {
                    tracks.push(json!({
                        "title": t.info.title,
                        "author": t.info.author,
                        "duration": t.info.length,
                        "uri": t.info.uri,
                        "artwork": t.info.artwork_url,
                        "isStream": t.info.is_stream,
                    }));
                }
            }
            _ => {}
        }
    }

    Json::<serde_json::Value>(json!({ "tracks": tracks })).into_response()
}

async fn get_liked_songs(
    Path(user_id): Path<String>,
    State(data): State<Arc<Data>>,
) -> impl IntoResponse {
    match data.database.getLikedSongs(&user_id).await {
        Ok(songs) => Json::<serde_json::Value>(json!({ "likedSongs": songs })).into_response(),
        Err(e) => Json::<serde_json::Value>(json!({"error": e.to_string()})).into_response(),
    }
}

async fn handle_like(
    State(data): State<Arc<Data>>,
    Json(body): Json<LikeRequest>,
) -> impl IntoResponse {
    if body.action == "like" {
        let result = match data.lavalink.load_tracks(0, &body.uri).await {
            Ok(res) => res,
            Err(e) => return Json::<serde_json::Value>(json!({"error": e.to_string()})).into_response(),
        };

        let found = if let Some(load_data) = result.data {
            match load_data {
                lavalink_rs::model::track::TrackLoadData::Track(t) => Some(t),
                lavalink_rs::model::track::TrackLoadData::Search(list) => list.into_iter().next(),
                lavalink_rs::model::track::TrackLoadData::Playlist(pl) => pl.tracks.into_iter().next(),
                _ => None,
            }
        } else {
            None
        };

        let found_track = match found {
            Some(t) => t,
            None => return Json::<serde_json::Value>(json!({"error": "Track not found"})).into_response(),
        };

        match data.database.addToLikedSongs(
            &body.userId,
            &found_track.info.identifier,
            &found_track.info.title,
            &found_track.info.author,
            &found_track_info_uri(&found_track),
            found_track.info.artwork_url.clone(),
            found_track.info.length,
            found_track.info.is_stream,
        ).await {
            Ok(success) => Json::<serde_json::Value>(json!({ "success": success })).into_response(),
            Err(e) => Json::<serde_json::Value>(json!({"error": e.to_string()})).into_response(),
        }
    } else if body.action == "unlike" {
        let track_id = body.id.unwrap_or_else(|| body.uri.clone());
        match data.database.removeFromLikedSongs(&body.userId, &track_id).await {
            Ok(success) => Json::<serde_json::Value>(json!({ "success": success })).into_response(),
            Err(e) => Json::<serde_json::Value>(json!({"error": e.to_string()})).into_response(),
        }
    } else {
        Json::<serde_json::Value>(json!({"error": "Invalid action"})).into_response()
    }
}

fn found_track_info_uri(track: &lavalink_rs::model::track::TrackData) -> String {
    track.info.uri.as_ref().cloned().unwrap_or_default()
}

async fn get_recent_tracks(
    Path(user_id): Path<String>,
    Query(query): Query<RecentQuery>,
    State(data): State<Arc<Data>>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10);
    match data.database.getRecentlyPlayed(&user_id, query.guildId.as_deref(), limit).await {
        Ok(tracks) => Json::<serde_json::Value>(json!({ "tracks": tracks })).into_response(),
        Err(_e) => Json::<serde_json::Value>(json!({"tracks": []})).into_response(),
    }
}
