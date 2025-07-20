use anyhow::anyhow;
use std::fmt::Display;
pub mod config;
pub mod controller;
pub mod csound;
pub mod engine;
pub mod events;
pub mod layout;
pub mod midi_player;
pub mod pitch;
pub mod scale;
pub mod view;

fn to_anyhow<E: Display>(e: E) -> anyhow::Error {
    anyhow!("{e}")
}
