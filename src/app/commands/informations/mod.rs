mod ping;
mod invite;
mod stats;
mod nodes;
mod help;
mod about;
mod leaderboard;
mod new;

use crate::data::Data;
use crate::error::BotError;

pub fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![
        ping::ping(),
        invite::invite(),
        stats::stats(),
        nodes::nodes(),
        help::help(),
        about::about(),
        leaderboard::leaderboard(),
        new::new(),
    ]
}
