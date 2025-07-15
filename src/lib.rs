use std::error::Error;
use std::fmt::Display;

pub mod controller;
pub mod midi_player;

fn to_sync_send<E: Display>(e: E) -> Box<dyn Error + Sync + Send> {
    e.to_string().into()
}
