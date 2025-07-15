use std::error::Error;
use std::fmt::Display;

pub mod controller;
pub mod events;
pub mod midi_player;
pub mod web;

fn to_sync_send<E: Display>(e: E) -> Box<dyn Error + Sync + Send> {
    e.to_string().into()
}
