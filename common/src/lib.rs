use anstream::ColorChoice;
use anyhow::anyhow;
use std::fmt::Display;
use std::sync::LazyLock;

pub mod parsing;
pub mod pitch;

pub fn to_anyhow<E: Display>(e: E) -> anyhow::Error {
    anyhow!("{e}")
}

// Set CLICOLOR_FORCE to force color output; set NO_COLOR to force non-color output.
pub static USE_COLOR: LazyLock<bool> =
    LazyLock::new(|| !matches!(anstream::stdout().current_choice(), ColorChoice::Never));
