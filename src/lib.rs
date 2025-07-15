use anyhow::anyhow;
use std::fmt::Display;
pub mod config;
pub mod controller;
pub mod events;
pub mod midi_player;
pub mod pitch;
pub mod scale;
pub mod web;

fn to_anyhow<E: Display>(e: E) -> anyhow::Error {
    anyhow!("{e}")
}
