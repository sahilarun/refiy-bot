use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] libsql::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Lavalink error: {0}")]
    Lavalink(String),
    #[error("Serenity error: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),
    #[error("Not in a voice channel")]
    NotInVoice,
    #[error("No active player for this guild")]
    NoPlayer,
    #[error("Other error: {0}")]
    Other(String),
}

impl From<lavalink_rs::error::LavalinkError> for BotError {
    fn from(e: lavalink_rs::error::LavalinkError) -> Self {
        BotError::Lavalink(e.to_string())
    }
}
