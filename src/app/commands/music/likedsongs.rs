use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn likedsongs(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let user_id = ctx.author().id.to_string();
    let database = &ctx.data().database;

    let songs = database.getLikedSongs(&user_id).await?;

    if songs.is_empty() {
        return Err(BotError::Other("> You don't have any liked songs yet.".into()));
    }

    let mut description = String::new();
    for (i, song) in songs.iter().take(10).enumerate() {
        let title = song["title"].as_str().unwrap_or("Unknown");
        let uri = song["uri"].as_str().unwrap_or("#");
        let author = song["author"].as_str().unwrap_or("Unknown");
        description.push_str(&format!("{}. **[{}]({})** - `{}`\n", i + 1, title, uri, author));
    }

    if songs.len() > 10 {
        description.push_str(&format!("\n*And {} more...*", songs.len() - 10));
    }

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "❤️ **Liked Songs ({})**\n\n{}",
                songs.len(),
                description
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
