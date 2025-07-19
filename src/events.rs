use crate::engine::PlayedNote;
use crate::layout::Layout;
use crate::scale::Note;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

mod rgb_colors;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Color {
    Off,
    Blue,
    Green,
    Purple,
    Pink,
    Red,
    Orange,
    Cyan,
    Yellow,
    DullGray,
    HighlightGray,
    White,
}
impl Color {
    pub fn launchpad_color(&self) -> u8 {
        match self {
            Color::Off => 0,
            Color::Blue => 0x4f, //2d,
            Color::Green => 0x15,
            Color::Purple => 0x35,
            Color::Pink => 0x38,
            Color::Red => 0x06,
            Color::Orange => 0x09,
            Color::Cyan => 0x25,
            Color::Yellow => 0x0d,
            Color::DullGray => 0x47,
            Color::HighlightGray => 0x7d,
            Color::White => 0x03,
        }
    }

    pub fn rgb_color(&self) -> &'static str {
        rgb_colors::RGB_COLORS[self.launchpad_color() as usize]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LightMode {
    Off,
    On,
    Flashing,
    Pulsing,
}

#[derive(Clone, Debug)]
pub struct LightEvent {
    pub mode: LightMode,
    pub position: u8,
    pub color: Color,
    pub label1: String,
    pub label2: String,
}
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// Midi note number
    pub key: u8,
    /// 0..127, 0 = off
    pub velocity: u8,
}
#[derive(Clone, Debug)]
pub struct PressureEvent {
    pub key: Option<u8>,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub struct AssignLayoutEvent {
    pub position: u8,
    pub layout: Arc<Layout>,
}

#[derive(Clone, Debug)]
pub struct SelectLayoutEvent {
    pub layout: Arc<Layout>,
}

#[derive(Clone, Debug)]
pub struct UpdateNoteEvent {
    pub position: u8,
    pub played_note: Option<PlayedNote>,
}

#[derive(Clone, Debug)]
pub struct PlayNoteEvent {
    pub note: Arc<Note>,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub enum Event {
    Shutdown,
    Light(LightEvent),
    Key(KeyEvent),
    Pressure(PressureEvent),
    Reset,
    AssignLayout(AssignLayoutEvent),
    SelectLayout(SelectLayoutEvent),
    UpdateNote(UpdateNoteEvent),
    PlayNote(PlayNoteEvent),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Light(LightEvent {
                mode,
                position,
                color,
                ..
            }) => write!(
                f,
                "light: mode={mode:?}, position={position}, color={color:?}"
            ),
            Event::Key(KeyEvent { key, velocity }) => {
                write!(f, "key: key={key:02}, velocity={velocity}")
            }
            Event::Pressure(PressureEvent { key, velocity }) => write!(
                f,
                "pressure: key={}, velocity={velocity}",
                key.map(|x| format!("{x:02}"))
                    .unwrap_or("global".to_string())
            ),
            _ => write!(f, "{self:?}"),
        }
    }
}

pub type UpgradedSender = broadcast::Sender<Event>;
pub type Sender = broadcast::WeakSender<Event>;
pub type Receiver = broadcast::Receiver<Event>;

pub struct Events {
    tx: RwLock<Option<UpgradedSender>>,
    rx: Receiver,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

/// Receive an event, ignoring lag
pub async fn receive_check_lag(rx: &mut Receiver, warn_prefix: Option<&str>) -> Option<Event> {
    loop {
        let event = rx.recv().await;
        match event {
            Ok(Event::Shutdown) => return None,
            Ok(event) => return Some(event),
            Err(err) => match err {
                RecvError::Closed => return None,
                RecvError::Lagged(n) => {
                    if let Some(p) = warn_prefix {
                        log::warn!("{p}: missed {n} events");
                    }
                    continue;
                }
            },
        }
    }
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1000);
        Self {
            tx: RwLock::new(Some(tx)),
            rx,
        }
    }

    pub fn sender(&self) -> Sender {
        let tx = self
            .tx
            .read()
            .unwrap()
            .clone()
            .expect("sender called after shutdown");
        tx.downgrade()
    }

    pub fn receiver(&self) -> Receiver {
        self.rx.resubscribe()
    }

    pub fn shutdown(&self) {
        self.tx.write().unwrap().take();
    }
}
