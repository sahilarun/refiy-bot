#![allow(non_snake_case)]
use super::HearthCategory;

#[derive(Debug, Clone)]
pub struct CommandOptions {
    pub cooldown: u64,
    pub onlyDeveloper: bool,
    pub onlyGuildOwner: bool,
    pub category: HearthCategory,
    pub premium: bool,
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            cooldown: 3,
            onlyDeveloper: false,
            onlyGuildOwner: false,
            category: HearthCategory::Unknown,
            premium: false,
        }
    }
}
