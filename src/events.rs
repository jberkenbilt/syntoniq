use crate::engine::PlayedNote;
use crate::layout::Layout;
use crate::pitch::Pitch;
use crate::scale::Note;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{RwLock, broadcast};

mod rgb_colors;

const COLOR_WHITE: u8 = 0x03;
const COLOR_BLUE: u8 = 0x4f;
const COLOR_GREEN: u8 = 0x15;
const COLOR_PURPLE: u8 = 0x31;
const COLOR_PINK: u8 = 0x38;
const COLOR_RED: u8 = 0x06;
const COLOR_ORANGE: u8 = 0x09;
const COLOR_CYAN: u8 = 0x27;
const COLOR_YELLOW: u8 = 0x0d;
const COLOR_DULL_GRAY: u8 = 0x47;
const COLOR_HIGHLIGHT_GRAY: u8 = 0x01;
const COLOR_MAGENTA: u8 = 0x5f;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Color {
    Off,
    Active,
    ToggleOff,
    ToggleOn,
    FifthOff,
    FifthOn,
    MajorThirdOff,
    MajorThirdOn,
    MinorThirdOff,
    MinorThirdOn,
    TonicOff,
    TonicOn,
    SingleStepOff,
    SingleStepOn,
    OtherOff,
    OtherOn,
    NoteSelected,
}
impl Color {
    pub fn launchpad_color(&self) -> u8 {
        match self {
            Color::Off => 0,
            Color::Active => COLOR_WHITE,
            Color::ToggleOff => COLOR_RED,
            Color::ToggleOn => COLOR_GREEN,
            Color::FifthOff => COLOR_BLUE,
            Color::FifthOn => COLOR_GREEN,
            Color::MajorThirdOff => COLOR_PURPLE,
            Color::MajorThirdOn => COLOR_PINK,
            Color::MinorThirdOff => COLOR_RED,
            Color::MinorThirdOn => COLOR_ORANGE,
            Color::TonicOff => COLOR_CYAN,
            Color::TonicOn => COLOR_YELLOW,
            Color::OtherOff => COLOR_DULL_GRAY,
            Color::OtherOn => COLOR_WHITE,
            Color::SingleStepOff => COLOR_HIGHLIGHT_GRAY,
            Color::SingleStepOn => COLOR_WHITE,
            Color::NoteSelected => COLOR_MAGENTA,
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
    pub layout: Arc<RwLock<Layout>>,
}

#[derive(Clone, Debug)]
pub struct SelectLayoutEvent {
    pub layout: Arc<RwLock<Layout>>,
}

#[derive(Clone, Debug)]
pub struct UpdateNoteEvent {
    pub position: u8,
    pub played_note: Option<PlayedNote>,
}

#[derive(Clone, Debug)]
pub struct PlayNoteEvent {
    pub pitch: Pitch,
    pub velocity: u8,
    pub note: Option<Arc<Note>>,
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
pub type WeakSender = broadcast::WeakSender<Event>;
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

    pub async fn sender(&self) -> WeakSender {
        let tx = self
            .tx
            .read()
            .await
            .clone()
            .expect("sender called after shutdown");
        tx.downgrade()
    }

    pub fn receiver(&self) -> Receiver {
        self.rx.resubscribe()
    }

    pub async fn shutdown(&self) {
        self.tx.write().await.take();
    }
}
