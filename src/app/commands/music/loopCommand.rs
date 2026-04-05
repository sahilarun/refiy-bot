use crate::error::BotError;
use crate::data::{Context, RepeatMode};
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", rename = "loop")]
pub async fn loopCommand(
    ctx: Context<'_>,
    #[description = "Loop mode"]
    mode: Option<RepeatModeArg>
) -> Result<(), BotError> {
    ctx.defer().await?;
    let guildId = ctx.guild_id().unwrap();
    let guildPlayers = &ctx.data().guildPlayers;

    if let Some(mut guildPlayer) = guildPlayers.get_mut(&guildId) {
        let newMode = match mode {
            Some(RepeatModeArg::Off) => RepeatMode::Off,
            Some(RepeatModeArg::Track) => RepeatMode::Track,
            Some(RepeatModeArg::Queue) => RepeatMode::Queue,
            None => {
                match guildPlayer.repeatMode {
                    RepeatMode::Off => RepeatMode::Track,
                    RepeatMode::Track => RepeatMode::Queue,
                    RepeatMode::Queue => RepeatMode::Off,
                }
            }
        };

        guildPlayer.repeatMode = newMode;

        let components = vec![
            V2Component::Container(Container::new(vec![
                V2Component::TextDisplay(TextDisplay::new(format!(
                    "> Loop mode has been set to **{:?}**.",
                    newMode
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

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatModeArg {
    #[name = "Off"]
    Off,
    #[name = "Track"]
    Track,
    #[name = "Queue"]
    Queue,
}
