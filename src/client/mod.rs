#![allow(non_snake_case)]

use tracing::info;

use crate::config::Config;

pub struct Hearth {
    pub config: Config,
}

impl Hearth {
    pub fn new(config: Config) -> Self {
        info!("[Hearth] Client initialized");
        Self { config }
    }

    pub fn token(&self) -> &str {
        &self.config.token
    }
    pub fn isDeveloper(&self, userId: u64) -> bool {
        self.config.developers.contains(&userId)
    }
}
