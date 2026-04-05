use poise::serenity_prelude as serenity;
use crate::error::BotError;
use crate::data::Context;
use tokio_tungstenite::tungstenite::Message;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music")]
pub async fn join(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();

    let voice_states = ctx.guild().unwrap().voice_states.clone();
    let voice_state = voice_states.get(&ctx.author().id).ok_or(BotError::Other("> You must be in a voice channel.".into()))?;
    let channel_id = voice_state.channel_id.ok_or(BotError::Other("> You must be in a voice channel.".into()))?;

    if let Some(mut guild_player) = ctx.data().guildPlayers.get_mut(&guild_id) {
        guild_player.voiceChannelId = Some(channel_id.get());
    }

    let shard_manager = ctx.framework().shard_manager();
    let shard_id = serenity::ShardId(ctx.guild().unwrap().shard_id(&ctx.serenity_context().cache));
    let runners = shard_manager.runners.lock().await;

    if let Some(runner) = runners.get(&shard_id) {
        let payload = serenity::json::json!({
            "op": 4,
            "d": {
                "guild_id": guild_id.to_string(),
                "channel_id": channel_id.to_string(),
                "self_mute": false,
                "self_deaf": true,
            }
        });

        let message = Message::Text(serde_json::to_string(&payload).map_err(|e| BotError::Other(e.to_string()))?);
        runner.runner_tx.websocket_message(message);
    }
    drop(runners);

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(format!(
                "> Joined <#{}>.",
                channel_id
            ))),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}
