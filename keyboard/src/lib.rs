pub mod controller;
#[cfg(feature = "csound")]
pub mod csound;
pub mod engine;
pub mod events;
pub mod launchpad;
pub mod midi_player;
#[cfg(test)]
pub mod test_util;
pub mod view;

#[derive(Copy, Clone)]
pub enum DeviceType {
    Empty,
    Launchpad,
}
