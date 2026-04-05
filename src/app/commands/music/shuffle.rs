use crate::error::BotError;
use crate::data::Context;
use rand::seq::SliceRandom;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer, checkTracks};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("shu", "sh", "shuf"))]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? || !checkTracks(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;

    if let Some(mut guildPlayer) = guild_players.get_mut(&guild_id) {
        if guildPlayer.queue.is_empty() {
            return Err(BotError::Other("> The queue is empty.".into()));
        }

        let count = guildPlayer.queue.len();
        {
            let mut queueVec: Vec<_> = guildPlayer.queue.drain(..).collect();
            let mut rng = rand::thread_rng();
            queueVec.shuffle(&mut rng);
            guildPlayer.queue = std::collections::VecDeque::from(queueVec);
        }

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> Shuffled **{}** tracks.",
                    count
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
