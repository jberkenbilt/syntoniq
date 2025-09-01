use anyhow::anyhow;
use std::fmt::Display;

pub mod pitch;

pub fn to_anyhow<E: Display>(e: E) -> anyhow::Error {
    anyhow!("{e}")
}
