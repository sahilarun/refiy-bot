mod filter;

use crate::data::Data;
use crate::error::BotError;

pub fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![filter::filter()]
}
