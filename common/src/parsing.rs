// Rust 1.89.0 is giving false positive on needless lifetimes.
#![allow(clippy::needless_lifetimes)]

pub mod model;
pub mod pass1;
pub mod pass2;
