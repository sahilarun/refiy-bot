use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkNodeConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub secure: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub token: String,
    pub prefix: String,
    pub databaseUrl: String,
    pub databasePassword: String,
    pub redisUrl: String,
    pub lavalinkNodes: Vec<LavalinkNodeConfig>,
    pub defaultVolume: u16,
    pub colorPrimary: u32,
    pub colorError: u32,
    pub colorSuccess: u32,
    pub developers: Vec<u64>,
    pub channelNodeLogs: Option<u64>,
    pub channelGuildLogs: Option<u64>,
    pub channelCommandLogs: Option<u64>,
    pub channelErrorLogs: Option<u64>,
}

impl Config {
    pub fn from_env() -> Result<Self, crate::error::BotError> {
        let token = std::env::var("DISCORD_TOKEN").map_err(|_| crate::error::BotError::Config("DISCORD_TOKEN not set".into()))?;
        let prefix = std::env::var("PREFIX").unwrap_or_else(|_| ".".into());
        let databaseUrl = std::env::var("DATABASE_URL").map_err(|_| crate::error::BotError::Config("DATABASE_URL not set".into()))?;
        let databasePassword = std::env::var("DATABASE_PASSWORD").unwrap_or_default();
        let redisUrl = std::env::var("REDIS_URL").map_err(|_| crate::error::BotError::Config("REDIS_URL not set".into()))?;
        let lavalinkNodes = if let Ok(nodesStr) = std::env::var("LAVALINK_NODES") {
            nodesStr.split(',')
                .filter_map(|s| {
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() >= 3 {
                        Some(LavalinkNodeConfig {
                            host: parts[0].to_string(),
                            port: parts[1].parse().unwrap_or(2333),
                            password: parts[2].to_string(),
                            secure: parts.get(3).map(|&s| s == "true").unwrap_or(false),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            // Fallback
            vec![LavalinkNodeConfig {
                host: std::env::var("LAVALINK_HOST").unwrap_or_else(|_| "localhost".into()),
                port: std::env::var("LAVALINK_PORT").unwrap_or_else(|_| "2333".into()).parse().unwrap_or(2333),
                password: std::env::var("LAVALINK_PASSWORD").unwrap_or_else(|_| "youshallnotpass".into()),
                secure: false,
            }]
        };

        if lavalinkNodes.is_empty() {
            return Err(crate::error::BotError::Config("No Lavalink nodes configured".into()));
        }

        let defaultVolume = std::env::var("DEFAULT_VOLUME").unwrap_or_else(|_| "100".into()).parse().unwrap_or(100);
        let colorPrimary = u32::from_str_radix(&std::env::var("COLOR_PRIMARY").unwrap_or_else(|_| "5865F2".into()), 16).unwrap_or(0x5865F2);
        let colorError = u32::from_str_radix(&std::env::var("COLOR_ERROR").unwrap_or_else(|_| "ED4245".into()), 16).unwrap_or(0xED4245);
        let colorSuccess = u32::from_str_radix(&std::env::var("COLOR_SUCCESS").unwrap_or_else(|_| "57F287".into()), 16).unwrap_or(0x57F287);
        let developers = std::env::var("DEVELOPERS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
            
        let channelNodeLogs = std::env::var("CHANNEL_NODE_LOGS").ok().and_then(|s| s.parse().ok());
        let channelGuildLogs = std::env::var("CHANNEL_GUILD_LOGS").ok().and_then(|s| s.parse().ok());
        let channelCommandLogs = std::env::var("CHANNEL_COMMAND_LOGS").ok().and_then(|s| s.parse().ok());
        let channelErrorLogs = std::env::var("CHANNEL_ERROR_LOGS").ok().and_then(|s| s.parse().ok());

        Ok(Self {
            token,
            prefix,
            databaseUrl,
            databasePassword,
            redisUrl,
            lavalinkNodes,
            defaultVolume,
            colorPrimary,
            colorError,
            colorSuccess,
            developers,
            channelNodeLogs,
            channelGuildLogs,
            channelCommandLogs,
            channelErrorLogs,
        })
    }
}
