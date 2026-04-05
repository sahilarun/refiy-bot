use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", rename = "move")]
pub async fn moveTrack(
    ctx: Context<'_>,
    #[description = "The position of the track to move"]
    from: usize,
    #[description = "The new position for the track"]
    to: usize
) -> Result<(), BotError> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;

    if from == 0 || to == 0 {
        return Err(BotError::Other("> Positions must be greater than 0.".into()));
    }

    if let Some(mut guild_player) = guild_players.get_mut(&guild_id) {
        if from > guild_player.queue.len() || to > guild_player.queue.len() {
            return Err(BotError::Other(format!("> Queue only has {} tracks.", guild_player.queue.len()).into()));
        }

        let track = guild_player.queue.remove(from - 1).unwrap();
        guild_player.queue.insert(to - 1, track.clone());

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> Moved **[{}]({})** from position **{}** to **{}**.",
                    track.title, track.uri, from, to
                ))),
            ]))
        ];

        let payload = V2MessagePayload::new(components);
        payload.send_interaction(ctx).await?;
    } else {
        return Err(BotError::NoPlayer);
    }

    Ok(())
}
