use crate::engine::PlayedNote;
use crate::layout::Layout;
use crate::scale::{Note, ScaleDescription};
use askama::Template;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use syntoniq_common::pitch::Pitch;
use tokio::sync::broadcast::error::RecvError;
#[cfg(test)]
use tokio::sync::mpsc;
use tokio::sync::{RwLock, broadcast};

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

#[derive(Clone, Debug)]
pub struct RawLightEvent {
    pub position: u8,
    pub color: Color,
    pub label1: String,
    pub label2: String,
}
#[derive(Clone, Debug)]
pub struct RawKeyEvent {
    /// Midi note number
    pub key: u8,
    /// 0..127, 0 = off
    pub velocity: u8,
}
#[derive(Clone, Debug)]
pub struct RawPressureEvent {
    pub key: Option<u8>,
    pub velocity: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum KeyData {
    Shift,
    Layout { idx: usize },
    Clear,
    Sustain,
    Transpose,
    OctaveShift { up: bool },
    Print,
    Note { position: u8 },
}

#[derive(Clone, Debug)]
pub struct KeyEvent {
    pub key: KeyData,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub enum LightData {
    Shift,
    Sustain,
    Transpose,
}
#[derive(Clone, Debug)]
pub struct LightEvent {
    pub light: LightData,
    pub color: Color,
    pub label1: String,
    pub label2: String,
}

#[derive(Clone, Debug)]
pub struct LayoutNamesEvent {
    pub names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SelectLayoutEvent {
    pub idx: usize,
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

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum ShiftKeyState {
    #[default]
    Off, // Next on event turns on
    On,   // Next off event turns on
    Down, // Next off event leaves on
}

#[derive(Debug, Clone)]
pub struct SpecificNote {
    pub layout_idx: usize,
    pub note: Arc<Note>,
    pub position: u8,
}

#[derive(Default, Debug, Clone)]
pub enum TransposeState {
    #[default]
    Off,
    Pending {
        initial_layout: usize,
    },
    FirstSelected {
        initial_layout: usize,
        note1: SpecificNote,
    },
}

#[derive(Default, Debug, Clone)]
pub enum ShiftLayoutState {
    #[default]
    Off,
    FirstSelected(SpecificNote),
}

#[derive(Default, Clone, Debug)]
pub struct EngineState {
    /// Currently selected layout
    pub layout: Option<usize>,
    /// All available layouts
    pub layouts: Vec<Arc<RwLock<Layout>>>,
    /// Mapping from position to note in the current layout
    pub notes: HashMap<u8, Option<Arc<Note>>>,
    /// Mapping from pitch to all positions with that pitch in the current layout
    pub pitch_positions: HashMap<Pitch, HashSet<u8>>,
    /// Number of times a pitch is on; > 1 if simultaneously touching more than one position
    /// with the same pitch in non-sustain mode
    pub pitch_on_count: HashMap<Pitch, u8>,
    /// Last note played for a given pitch
    pub last_note_for_pitch: HashMap<Pitch, Arc<Note>>,
    /// Positions that are actually being touched
    pub positions_down: HashMap<u8, Arc<Note>>,
    pub sustain: bool,
    pub shift_key_state: ShiftKeyState,
    pub transpose_state: TransposeState,
    pub shift_layout_state: ShiftLayoutState,
}
impl EngineState {
    pub fn current_layout(&self) -> Option<Arc<RwLock<Layout>>> {
        self.layout.and_then(|x| self.layouts.get(x).cloned())
    }

    pub fn current_played_notes(&self) -> Vec<String> {
        // It would more efficient to directly print, but this is not performance-critical,
        // and generating a Vec makes testing easier.
        let mut result = Vec::new();
        // Scale name -> notes in the scale
        let mut scale_to_notes: HashMap<ScaleDescription, Vec<&Arc<Note>>> = HashMap::new();
        for note in self.last_note_for_pitch.values() {
            let key = note.scale_description.clone();
            scale_to_notes.entry(key).or_default().push(note);
        }
        let mut keys: Vec<ScaleDescription> = scale_to_notes.keys().cloned().collect();
        keys.sort();
        for scale in keys {
            result.push(format!("Scale: {scale}"));
            let mut notes = scale_to_notes.remove(&scale).unwrap();
            notes.sort_by_key(|note| note.pitch.clone());
            for note in notes {
                let Note {
                    name,
                    description,
                    pitch,
                    scale_description,
                    base_factor,
                    colors: _,
                } = note.as_ref();
                let scale_base_pitch = &scale_description.base_pitch;
                result.push(format!(
                    "  Note: {name} ({description}), pitch={pitch} ({scale_base_pitch} × {base_factor})"
                ));
            }
        }
        result
    }
}

#[derive(Template, Default, Clone)]
#[template(path = "state-view.html")]
pub struct StateView {
    pub selected_layout: String,
    pub scale_name: String,
    pub base_pitch: String,
    pub layout_names: Vec<String>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TestEvent {
    ResetComplete,
    DeviceResetComplete,
    LayoutSelected,
    LayoutsHandled,
    HandledNote,
    HandledKey,
    MoveCanceled,
    Sync,
}

#[derive(Clone, Debug)]
pub enum FromDevice {
    Key(RawKeyEvent),
    Pressure(RawPressureEvent),
}

#[derive(Clone, Debug)]
pub enum ToDevice {
    // TODO: this is probably not right. Which device?
    Light(RawLightEvent),
    ClearLights,
}

#[derive(Clone, Debug)]
pub enum Event {
    Shutdown,
    ToDevice(ToDevice),
    Reset,
    ResetDevice,
    KeyEvent(KeyEvent),
    LightEvent(LightEvent),
    SetLayoutNames(LayoutNamesEvent),
    SelectLayout(SelectLayoutEvent),
    UpdateNote(UpdateNoteEvent),
    PlayNote(PlayNoteEvent),
    #[cfg(test)]
    TestEngine(mpsc::Sender<EngineState>),
    #[cfg(test)]
    TestWeb(mpsc::Sender<StateView>),
    #[cfg(test)]
    TestEvent(TestEvent),
    #[cfg(test)]
    TestSync,
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

#[cfg(test)]
pub fn send_test_event(events_tx: &WeakSender, test_event: TestEvent) {
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::TestEvent(test_event)).unwrap();
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
