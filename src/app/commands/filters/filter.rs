use lavalink_rs::model::player::{Filters, Equalizer as Band, Timescale, Rotation, Karaoke};
use crate::error::BotError;
use crate::data::Context;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkVoiceChannel, checkBotVoiceChannel, checkPlayer};

#[poise::command(
    slash_command, 
    prefix_command, 
    guild_only, 
    category = "Filters"
)]
pub async fn filter(
    ctx: Context<'_>,
    #[description = "The filter to apply"]
    filter: FilterMode
) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkVoiceChannel(ctx).await? || !checkBotVoiceChannel(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = &ctx.data().lavalink;
    let player = lavalink.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;

    let mut filters = Filters::default();

    match filter {
        FilterMode::Off => {
            player.set_filters(filters).await?;
        }
        FilterMode::Bassboost => {
            filters.equalizer = Some(vec![
                Band { band: 0, gain: 0.6 },
                Band { band: 1, gain: 0.7 },
                Band { band: 2, gain: 0.8 },
                Band { band: 3, gain: 0.55 },
            ]);
            player.set_filters(filters).await?;
        }
        FilterMode::Nightcore => {
            filters.timescale = Some(Timescale {
                speed: Some(1.3),
                pitch: Some(1.3),
                rate: Some(1.0),
            });
            player.set_filters(filters).await?;
        }
        FilterMode::Vaporwave => {
            filters.timescale = Some(Timescale {
                speed: Some(0.85),
                pitch: Some(0.8),
                rate: Some(1.0),
            });
            player.set_filters(filters).await?;
        }
        FilterMode::EightD => {
            filters.rotation = Some(Rotation {
                rotation_hz: Some(0.2),
            });
            player.set_filters(filters).await?;
        }
        FilterMode::Karaoke => {
            filters.karaoke = Some(Karaoke {
                level: Some(1.0),
                mono_level: Some(1.0),
                filter_band: Some(220.0),
                filter_width: Some(100.0),
            });
            player.set_filters(filters).await?;
        }
        FilterMode::Pop => {
            filters.equalizer = Some(vec![
                Band { band: 0, gain: -0.08 },
                Band { band: 1, gain: 0.12 },
                Band { band: 2, gain: 0.4 },
                Band { band: 3, gain: 0.12 },
                Band { band: 4, gain: -0.04 },
            ]);
            player.set_filters(filters).await?;
        }
        FilterMode::Soft => {
            filters.equalizer = Some(vec![
                Band { band: 0, gain: 0.0 },
                Band { band: 1, gain: 0.0 },
                Band { band: 2, gain: -0.1 },
                Band { band: 3, gain: -0.25 },
                Band { band: 4, gain: -0.4 },
            ]);
            player.set_filters(filters).await?;
        }
        FilterMode::TrebleBass => {
            filters.equalizer = Some(vec![
                Band { band: 0, gain: 0.6 },
                Band { band: 1, gain: 0.6 },
                Band { band: 13, gain: 0.6 },
                Band { band: 14, gain: 0.6 },
            ]);
            player.set_filters(filters).await?;
        }
        FilterMode::Tremolo => {
            return Err(BotError::Other("> Tremolo filter is not supported yet.".into()));
        }
        FilterMode::Vibrato => {
            return Err(BotError::Other("> Vibrato filter is not supported yet.".into()));
        }
    }

    let status_text = match filter {
        FilterMode::Off => "> All audio filters have been reset to default.".to_string(),
        _ => format!("> The audio filter **{:?}** has been applied.", filter),
    };

    let components = vec![
        V2Component::Container(Container::new(vec![
            V2Component::TextDisplay(TextDisplay::new(status_text)),
        ]))
    ];

    let payload = V2MessagePayload::new(components);
    payload.send_interaction(ctx).await?;

    Ok(())
}

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    #[name = "Reset"]
    Off,
    #[name = "Bassboost"]
    Bassboost,
    #[name = "Nightcore"]
    Nightcore,
    #[name = "Vaporwave"]
    Vaporwave,
    #[name = "8D"]
    EightD,
    #[name = "Karaoke"]
    Karaoke,
    #[name = "Pop"]
    Pop,
    #[name = "Soft"]
    Soft,
    #[name = "TrebleBass"]
    TrebleBass,
    #[name = "Tremolo"]
    Tremolo,
    #[name = "Vibrato"]
    Vibrato,
}
