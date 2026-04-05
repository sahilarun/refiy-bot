#![allow(non_snake_case)]
mod client;
mod app;
mod components;
mod config;
mod data;
mod database;
mod error;
mod events;
mod lavalink;
mod types;
mod utils;
mod api;

use std::sync::Arc;
use dashmap::DashMap;
use poise::serenity_prelude as serenity;
use tracing::info;

use crate::config::Config;
use crate::data::Data;
use crate::database::Database;
use crate::error::BotError;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(32 * 1024 * 1024)
        .build()?;
    runtime.block_on(async {
        tokio::spawn(async_main()).await?
    })
}

async fn async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    info!("[Hearth] Starting bot...");
    let config = Config::from_env()?;
    let token = config.token.clone();
    let prefix = config.prefix.clone();
    let database = Database::new(&config.databaseUrl, &config.databasePassword).await?;
    info!("[Hearth] Database connected");

    let config_setup = config.clone();
    let database_setup = database.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: app::commands::allCommands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix),
                dynamic_prefix: Some(|ctx| {
                    Box::pin(async move {
                        let guild_id = match ctx.guild_id {
                            Some(id) => id.to_string(),
                            None => return Ok(None),
                        };
                        let db: &crate::database::Database = &ctx.data.database;
                        let prefix_result: Result<String, crate::error::BotError> = db.getPrefix(&guild_id).await;
                        let prefix: Option<String> = prefix_result.ok();
                        Ok(prefix)
                    })
                }),
                case_insensitive_commands: true,
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::event_handler(ctx, event, framework, data))
            },
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            tracing::error!("[Command] Error: {}", error);
                            let components = crate::components::v2::createErrorV2();
                            let _ = crate::components::v2::V2MessagePayload::new(components)
                                .send(ctx.serenity_context().http.as_ref(), ctx.channel_id())
                                .await;
                        }
                        poise::FrameworkError::ArgumentParse { error, ctx, .. } => {
                            let _ = ctx
                                .say(format!("❌ Invalid argument: {}", error))
                                .await;
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!("[Framework] Error handling error: {}", e);
                            }
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                if let Err(e) = poise::builtins::register_globally(ctx, &framework.options().commands).await {
                    tracing::error!("[Hearth] Global command registration failed: {}. Continuing anyway...", e);
                } else {
                    info!("[Hearth] Commands registered globally");
                }
                let botUserId = ready.user.id.get();
                let configArc = Arc::new(config_setup);
                let guildPlayers = Arc::new(DashMap::new());
                
                let redisClient = redis::Client::open(configArc.redisUrl.clone()).map_err(|e: redis::RedisError| BotError::Config(e.to_string()))?;
                let mut redisManager = match redis::aio::ConnectionManager::new(redisClient).await {
                    Ok(mgr) => {
                        info!("[Hearth] Redis connected");
                        mgr
                    }
                    Err(e) => {
                        tracing::error!("[Hearth] Redis connection failed: {}. Creating a dummy connection manager...", e);
                        return Err(BotError::Other(format!("Redis connection failed: {}", e)).into());
                    }
                };

                if let Ok(saved_players) = crate::utils::redis::loadAllPlayers(&mut redisManager).await {
                    for (guild_id, player) in &saved_players {
                        guildPlayers.insert(poise::serenity_prelude::GuildId::new(*guild_id), player.clone());
                    }
                    info!("[Hearth] Restored {} player sessions from Redis", saved_players.len());
                }

                let lavalink = crate::lavalink::createLavalinkClient(
                    configArc.clone(),
                    database_setup.clone(),
                    redisManager.clone(),
                    guildPlayers.clone(),
                    ctx.http.clone(),
                    botUserId
                ).await?;
                info!("[Hearth] Lavalink client created");
                let data = Data {
                    lavalink,
                    config: configArc,
                    database: database_setup,
                    redis: redisManager,
                    guildPlayers,
                    cooldowns: Arc::new(DashMap::new()),
                    botUserId,
                    startTime: chrono::Utc::now().timestamp() as u64,
                };
                
                let dataClone = data.clone();
                tokio::spawn(async move {
                    api::start(dataClone).await;
                });
                
                Ok(data)
            })
        })
        .build();
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await?;
    info!("[Hearth] Connecting to Discord...");
    client.start_autosharded().await?;
    Ok(())
}
