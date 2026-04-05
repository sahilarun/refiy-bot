use poise::serenity_prelude as serenity;
use crate::data::QueuedTrack;
use crate::utils::formatDuration;

pub fn createNowPlayingEmbed(track: &QueuedTrack) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new()
        .title(format!("🎶 Now Playing: {}", track.title))
        .url(&track.uri)
        .field("Author", &track.author, true)
        .field("Duration", formatDuration(track.duration), true)
        .field("Requester", format!("<@{}>", track.requester), true)
        .color(0x00ff00);

    if let Some(artwork) = &track.artworkUrl {
        embed = embed.thumbnail(artwork);
    }

    embed
}
