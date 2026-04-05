mod play;
mod lyrics;
mod skip;
mod stop;
mod pause;
mod resume;
mod volume;
mod seek;
mod queue;
mod nowplaying;
mod shuffle;
mod loopCommand;
mod previous;
mod clear;
mod moveTrack;
mod like;
mod likedsongs;
mod recent;
mod join;
mod replay;
mod forward;
mod rewind;

use crate::data::Data;
use crate::error::BotError;

pub fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![
        play::play(),
        lyrics::lyrics(),
        skip::skip(),
        stop::stop(),
        pause::pause(),
        resume::resume(),
        volume::volume(),
        seek::seek(),
        queue::queue(),
        nowplaying::nowplaying(),
        shuffle::shuffle(),
        loopCommand::loopCommand(),
        previous::previous(),
        clear::clear(),
        moveTrack::moveTrack(),
        like::like(),
        likedsongs::likedsongs(),
        recent::recent(),
        join::join(),
        replay::replay(),
        forward::forward(),
        rewind::rewind(),
    ]
}
