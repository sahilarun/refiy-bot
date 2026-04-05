use poise::serenity_prelude::{self as serenity, CreateEmbed, ChannelId, Http};
use std::sync::Arc;
use crate::config::Config;

pub enum LogType {
    Node,
    Guild,
    Command,
    Error,
}

pub async fn send_log(
    http: &Arc<Http>,
    config: &Config,
    log_type: LogType,
    embed: CreateEmbed,
) {
    let channel_id = match log_type {
        LogType::Node => config.channelNodeLogs,
        LogType::Guild => config.channelGuildLogs,
        LogType::Command => config.channelCommandLogs,
        LogType::Error => config.channelErrorLogs,
    };

    if let Some(id) = channel_id {
        let _ = ChannelId::new(id).send_message(http, serenity::CreateMessage::new().embed(embed)).await;
    }
}
