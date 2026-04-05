use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn recent(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let user_id = ctx.author().id.to_string();
    let database = &ctx.data().database;

    let tracks = database.getRecentlyPlayed(&user_id, None, 10).await?;

    if tracks.is_empty() {
        return Err(BotError::Other("> You haven't played any tracks yet.".into()));
    }

    let mut description = String::new();
    for (i, track) in tracks.iter().enumerate() {
        let title = track["title"].as_str().unwrap_or("Unknown");
        let uri = track["uri"].as_str().unwrap_or("#");
        let author = track["author"].as_str().unwrap_or("Unknown");
        description.push_str(&format!("{}. **[{}]({})** - `{}`\n", i + 1, title, uri, author));
    }

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "🕒 **Recently Played**\n\n{}",
                description
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
