mod prefix;
mod autoplay;
mod twentyFourSeven;

use crate::data::Data;
use crate::error::BotError;

pub fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![
        prefix::prefix(),
        autoplay::autoplay(),
        twentyFourSeven::twentyFourSeven(),
    ]
}
