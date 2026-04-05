pub mod music;
pub mod filters;
pub mod configurations;
pub mod informations;
pub mod playlists;

use crate::data::Data;
use crate::error::BotError;

pub fn allCommands() -> Vec<poise::Command<Data, BotError>> {
    let mut cmds = Vec::new();
    cmds.extend(music::commands());
    cmds.extend(informations::commands());
    cmds.extend(filters::commands());
    cmds.extend(configurations::commands());
    cmds.extend(playlists::commands());
    cmds
}
