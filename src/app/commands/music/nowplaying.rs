use crate::error::BotError;
use crate::data::{Context, RepeatMode};
use crate::utils::createNowPlayingEmbed;
use poise::serenity_prelude as serenity;
use futures::StreamExt;
use crate::components::v2::{Container, TextDisplay, V2MessagePayload, V2Component};
use crate::utils::checks::{checkNodes, checkPlayer};

#[poise::command(slash_command, prefix_command, guild_only, category = "Music", aliases("np"))]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    if !checkNodes(ctx).await? || !checkPlayer(ctx).await? {
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let guild_players = &ctx.data().guildPlayers;

    let guild_player = guild_players.get(&guild_id).ok_or(BotError::NoPlayer)?;
    let current = guild_player.currentTrack.as_ref().ok_or(BotError::Other("> Nothing is playing right now.".into()))?;

    let ctx_id = ctx.id();
    let embed = createNowPlayingEmbed(current);
    
    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(format!("{}pause", ctx_id)).emoji('⏯').style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new(format!("{}skip", ctx_id)).emoji('⏭').style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("{}repeat", ctx_id)).emoji('🔁').style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("{}shuffle", ctx_id)).emoji('🔀').style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("{}stop", ctx_id)).emoji('⏹').style(serenity::ButtonStyle::Danger),
        ])]);

    ctx.send(reply).await?;

    let lava = &ctx.data().lavalink;
    let mut collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(60))
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id.to_string()))
        .stream();

    while let Some(mci) = collector.next().await {
        let pc = lava.get_player_context(guild_id).ok_or(BotError::NoPlayer)?;
        let mut player = guild_players.get_mut(&guild_id).ok_or(BotError::NoPlayer)?;

        let custom_id = &mci.data.custom_id;
        let mut action_taken = "";

        if custom_id.ends_with("pause") {
            let paused = player.paused;
            pc.set_pause(!paused).await.map_err(|e: lavalink_rs::error::LavalinkError| BotError::Lavalink(e.to_string()))?;
            player.paused = !paused;
            action_taken = if player.paused { "> The player has been **paused**." } else { "> The player has been **resumed**." };
        } else if custom_id.ends_with("skip") {
            pc.stop_now().await.map_err(|e: lavalink_rs::error::LavalinkError| BotError::Lavalink(e.to_string()))?;
            action_taken = "> The current track has been **skipped**.";
        } else if custom_id.ends_with("repeat") {
            player.repeatMode = match player.repeatMode {
                RepeatMode::Off => RepeatMode::Track,
                RepeatMode::Track => RepeatMode::Queue,
                RepeatMode::Queue => RepeatMode::Off,
            };
            action_taken = match player.repeatMode {
                RepeatMode::Off => "> Repeat mode has been disabled.",
                RepeatMode::Track => "> Repeat mode has been set to **Track**.",
                RepeatMode::Queue => "> Repeat mode has been set to **Queue**.",
            };
        } else if custom_id.ends_with("shuffle") {
            use rand::seq::SliceRandom;
            let mut items: Vec<_> = player.queue.drain(..).collect();
            items.shuffle(&mut rand::thread_rng());
            player.queue.extend(items);
            action_taken = "> The queue has been **shuffled**.";
        } else if custom_id.ends_with("stop") {
            player.queue.clear();
            player.currentTrack = None;
            pc.stop_now().await.map_err(|e: lavalink_rs::error::LavalinkError| BotError::Lavalink(e.to_string()))?;
            action_taken = "> The player has been **stopped**.";
        }

        if !action_taken.is_empty() {
             let components = vec![
                V2Component::Container(Container::new(vec![
                    V2Component::TextDisplay(TextDisplay::new(action_taken.to_string())),
                ]))
            ];
            let payload = V2MessagePayload::new(components);
            payload.send_interaction(ctx).await?;
        }

        mci.create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::Acknowledge,
        )
        .await?;
    }

    Ok(())
}
