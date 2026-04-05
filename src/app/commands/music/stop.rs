use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("sp", "leave", "dc", "disconnect"))]
pub async fn stop(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;

    if let Some(mut guildPlayer) = ctx.data().guildPlayers.get_mut(&guild_id) {
        guildPlayer.queue.clear();
        guildPlayer.currentTrack = None;
    }

    lavalink.delete_player(guild_id.get()).await.map_err(|e| BotError::Lavalink(e.to_string()))?;

    let payload = serde_json::json!({
        "op": 4,
        "d": {
            "guild_id": guild_id.to_string(),
            "channel_id": null,
            "self_mute": false,
            "self_deaf": false
        }
    });
    let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&payload).map_err(|e| BotError::Other(e.to_string()))?);
    let _ = ctx.serenity_context().shard.websocket_message(message);
    
    ctx.data().guildPlayers.remove(&guild_id);

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new("> The player has been **stopped** and the queue cleared.".to_string())),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
