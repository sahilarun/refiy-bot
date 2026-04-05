use crate::error::BotError;
use crate::data::Context;
use crate::lavalink::getLyrics;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn lyrics(
    ctx: Context<'_>,
    #[description = "The song to search lyrics for (defaults to current)"]
    query: Option<String>
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap();
    let config = ctx.data().config.clone();

    if query.is_some() {
        return Err(BotError::Other("> Searching lyrics by query is not yet supported. Showing current track lyrics...".into()));
    }

    ctx.defer().await?;

    let lyrics_response = getLyrics(&config, guild_id.get()).await?;
    let text = lyrics_response.lines.map(|lines| {
        lines.iter()
            .map(|line| line.line.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }).unwrap_or_else(|| "No lyrics lines available.".into());

    let content = if text.len() > 4000 { format!("{}...", &text[..3997]) } else { text };

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "🎶 **Lyrics**\n\n{}",
                content
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
