#![allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct HearthConfiguration {
    pub defaultPrefix: String,
    pub defaultSearchPlatform: String,
    pub defaultVolume: u16,
    pub defaultLocale: String,
    pub lyricsLines: usize,
    pub serverPort: u16,
    pub info: BotInfo,
    pub topgg: TopGG,
    pub premium: PremiumConfig,
    pub developersIds: Vec<String>,
    pub color: Colors,
    pub webhooks: HearthWebhooks,
}

#[derive(Debug, Clone)]
pub struct BotInfo {
    pub banner: String,
    pub inviteLink: String,
    pub supportServer: String,
    pub voteLink: String,
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub primary: u32,
    pub secondary: u32,
    pub yes: u32,
    pub no: u32,
    pub warn: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TopGG {
    pub enabled: bool,
    pub webhookAuth: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct PremiumConfig {
    pub enabled: bool,
}
#[derive(Debug, Clone)]
pub struct HearthWebhooks {
    pub nodeLog: String,
    pub voteLog: String,
    pub guildLog: String,
    pub commandLog: String,
    pub errorLog: String,
    pub report: String,
}

#[derive(Debug, Clone)]
pub struct HearthEnvironment {
    pub token: Option<String>,
    pub databaseUrl: Option<String>,
    pub databasePassword: Option<String>,
}
