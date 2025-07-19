use crate::config::Config;
use crate::events::{
    AssignLayoutEvent, Color, Event, KeyEvent, LightEvent, LightMode, PlayNoteEvent,
    SelectLayoutEvent, UpdateNoteEvent,
};
use crate::layout::Layout;
use crate::pitch::Pitch;
use crate::scale::{Note, Scale, ScaleType};
use crate::{controller, csound, events, midi_player};
use anyhow::anyhow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

mod keys {
    pub const CLEAR: u8 = 60;
    pub const SUSTAIN: u8 = 95; // Chord
    pub const LAYOUT_MIN: u8 = 101;
    pub const LAYOUT_MAX: u8 = 108;
    pub const LAYOUT_SCROLL: u8 = 19;
}

#[derive(Debug, Clone)]
pub struct PlayedNote {
    note: Arc<Note>,
    velocity: u8,
}

struct Engine {
    config_file: PathBuf,
    config: Config,
    events_tx: events::Sender,
    layout: Option<Arc<Layout>>,
    /// control key position -> selected layout
    assigned_layouts: HashMap<u8, Arc<Layout>>,
    notes: HashMap<u8, Option<PlayedNote>>,
    note_positions: HashMap<Pitch, HashSet<u8>>,
    sustain: bool,
    notes_on: HashMap<Pitch, u8>, // number of times a note is on
}

impl Engine {
    async fn reset(&mut self) -> anyhow::Result<()> {
        match Config::load(&self.config_file) {
            Ok(config) => self.config = config,
            Err(e) => {
                log::error!("error reloading config; retaining old: {e}");
            }
        }
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        controller::clear_lights(&tx).await?;
        // TODO: fix these
        for (color, positions) in [
            (
                Color::FifthOn, // green
                vec![63u8, 64, 65, 66, 52, 57, 42, 47, 32, 37, 23, 24, 25],
            ),
            (Color::FifthOff, vec![34, 35, 16, 17, 18]), // blue
            (Color::TonicOff, vec![26]),                 // cyan
            (Color::MajorThirdOff, vec![72, 73, 74, 75, 76, 77]), // purple
            (Color::MajorThirdOn, vec![83]),             // pink
            (Color::MinorThirdOn, vec![84]),             // orange
            (Color::TonicOn, vec![85]),                  // yellow
            (Color::MinorThirdOff, vec![86]),            //red
        ] {
            for position in positions {
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position,
                    color,
                    label1: String::new(),
                    label2: String::new(),
                }))?;
            }
        }
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position: keys::CLEAR,
            color: Color::Active,
            label1: "Reset".to_string(),
            label2: String::new(),
        }))?;
        let mut position = keys::LAYOUT_MIN;
        self.assigned_layouts.clear();
        for layout in self.config.layouts.iter().cloned() {
            tx.send(Event::AssignLayout(AssignLayoutEvent { position, layout }))?;
            position += 1;
            if position > keys::LAYOUT_MAX {
                //TODO: deal with scrolling. Key 109 is the scroll key and will assign layouts
                // starting from an offset to the lower numbered keys.
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position: keys::LAYOUT_SCROLL,
                    color: Color::Active,
                    label1: "Scroll".to_string(),
                    label2: "layouts".to_string(),
                }))?;
                break;
            }
        }
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity } = key_event;
        let off = velocity == 0;
        match key {
            keys::LAYOUT_MIN..=keys::LAYOUT_MAX if off => {
                if let Some(layout) = self.assigned_layouts.get(&key).cloned() {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { layout }))?;
                }
            }
            keys::CLEAR if off => {
                tx.send(Event::Reset)?;
            }
            keys::SUSTAIN if off => {
                self.sustain = !self.sustain;
                tx.send(self.sustain_event())?;
            }
            position if self.notes.contains_key(&position) => {
                if let Some(note) = self.notes.get(&position).unwrap() {
                    let pitch = &note.note.pitch;
                    let Some(others) = self.note_positions.get(pitch) else {
                        // This would indicate a bug in which we assigned something to notes
                        // without also assigning its position to note positions or otherwise
                        // allowed notes and note_positions to get out of sync.
                        return Err(anyhow!("note positions is missing for {pitch}"));
                    };
                    let note_count = self.notes_on.entry(pitch.clone()).or_default();
                    // When not in sustain mode, touch turns a note on, and release turns it off.
                    // Since the same note may appear in multiple locations, we keep a count, and on
                    // send a note event if we transition to or from 0. In sustain mode, "off"
                    // events are ignored. Touching a note in any of its positions toggles whether
                    // it's on or off. Changing scales, transposing, shifting, etc. doesn't affect
                    // which notes are on or off, making it possible to play a note in one scale,
                    // switch scales, and play a note in another scale.
                    if self.sustain {
                        if off {
                            return Ok(());
                        }
                        if *note_count > 0 {
                            *note_count = 0;
                        } else {
                            *note_count = 1;
                        }
                    } else if off {
                        if *note_count > 0 {
                            *note_count -= 1
                        }
                        if *note_count > 0 {
                            return Ok(());
                        }
                    } else {
                        *note_count += 1;
                        if *note_count > 1 {
                            return Ok(());
                        }
                    }
                    let velocity = if *note_count > 0 { 127 } else { 0 };
                    for position in others.iter().copied() {
                        tx.send(note.note.light_event(position, velocity))?;
                    }
                    tx.send(Event::PlayNote(PlayNoteEvent {
                        note: note.note.clone(),
                        velocity,
                    }))?;
                }
            }
            _ => {} // TODO
        }
        Ok(())
    }

    async fn update_note(&mut self, event: UpdateNoteEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let UpdateNoteEvent {
            position,
            played_note,
        } = event;
        self.notes.insert(position, played_note.clone());
        match played_note {
            Some(played_note) => {
                let note = played_note.note.clone();
                self.note_positions
                    .entry(note.pitch.clone())
                    .or_default()
                    .insert(position);
                tx.send(note.light_event(position, played_note.velocity))?;
            }
            None => {
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::Off,
                    position,
                    color: Color::Off,
                    label1: String::new(),
                    label2: String::new(),
                }))?;
            }
        }
        Ok(())
    }

    async fn draw_edo_layout(&mut self, layout: &Layout, scale: &Scale) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::EqualDivision(ed) = &scale.scale_type else {
            // Should not be possible
            return Err(anyhow!("draw_edo_layout called with non-EDO scale"));
        };
        let (llx, lly, urx, ury) = layout.bbox;
        let (steps_x, steps_y) = layout.steps;
        let (base_x, base_y) = layout.base;
        let (divisions, _, _) = ed.divisions;
        let divisions = divisions as i32;
        self.note_positions.clear();
        self.notes.clear();
        for row in 1..=8 {
            for col in 1..=8 {
                let played_note = if !(llx..=urx).contains(&col) || !(lly..=ury).contains(&row) {
                    None
                } else {
                    let steps = (steps_x * (col - base_x) + steps_y * (row - base_y)) as i32;
                    let cycle = steps / divisions;
                    let step = steps % divisions;
                    let note = scale.note(cycle as i8, step as i8);
                    let velocity =
                        if self.notes_on.get(&note.pitch).copied().unwrap_or_default() > 0 {
                            127
                        } else {
                            0
                        };
                    Some(PlayedNote {
                        note: Arc::new(note),
                        velocity,
                    })
                };
                let position = (10 * row + col) as u8;
                tx.send(Event::UpdateNote(UpdateNoteEvent {
                    position,
                    played_note,
                }))?;
            }
        }
        log::info!("layout: {}, scale: {}", layout.name, scale.name);
        Ok(())
    }

    fn sustain_event(&self) -> Event {
        let sustain_color = if self.sustain {
            Color::ToggleOn
        } else {
            Color::ToggleOff
        };
        Event::Light(LightEvent {
            mode: LightMode::On,
            position: keys::SUSTAIN,
            color: sustain_color,
            label1: "Sustain".to_string(),
            label2: String::new(),
        })
    }

    async fn select_layout(&mut self, event: SelectLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        self.layout = Some(event.layout);
        let layout = self.layout.clone().unwrap();
        if let Some(scale) = &layout.scale {
            match scale.scale_type {
                ScaleType::EqualDivision(_) => {
                    self.draw_edo_layout(&layout, scale.as_ref()).await?
                }
                ScaleType::_KeepClippyQuiet => unreachable!(),
            }
            tx.send(self.sustain_event())?;
        }
        Ok(())
    }

    async fn assign_layout(&mut self, layout_event: AssignLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // Activate the light for selecting the layout.
        let AssignLayoutEvent { position, layout } = layout_event;
        if !(keys::LAYOUT_MIN..=keys::LAYOUT_MAX).contains(&position) {
            return Ok(());
        }
        self.assigned_layouts.insert(position, layout.clone());
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color: Color::Active,
            label1: layout.name.clone(),
            label2: layout.scale_name.clone(),
        }))?;
        Ok(())
    }
}

pub async fn run(
    config_file: PathBuf,
    midi: bool,
    events_tx: events::Sender,
    mut rx: events::Receiver,
) -> anyhow::Result<()> {
    let config = Config::load(&config_file)?;
    let mut engine = Engine {
        config_file,
        config,
        events_tx: events_tx.clone(),
        layout: None,
        assigned_layouts: Default::default(),
        notes: Default::default(),
        note_positions: Default::default(),
        sustain: false,
        notes_on: Default::default(),
    };
    let rx2 = rx.resubscribe();
    let tx2 = events_tx.clone();
    if midi {
        tokio::spawn(async move {
            if let Err(e) = midi_player::play_midi(rx2).await {
                log::error!("midi player error: {e}");
            };
        });
    } else {
        tokio::spawn(async move {
            if let Err(e) = csound::run_csound(rx2, tx2).await {
                log::error!("csound player error: {e}");
            };
        });
    }
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::Reset)?;
    }
    while let Some(event) = events::receive_check_lag(&mut rx, Some("engine")).await {
        // Note: this event loop calls event handlers inline. Sometimes those event handlers
        // generate other events, which are piling up in our queue while we are handling earlier
        // events. As long as the backlog on the event receiver is high enough and/or we don't
        // care about missing some messages, we should be fine. In practice, the messages we would
        // be most likely to miss our the ones we just sent, but we could also miss other people's
        // responses to those. We are not as likely to miss user events.
        match event {
            Event::Shutdown => break,
            Event::Light(_) => {}
            Event::Key(e) => engine.handle_key(e).await?,
            Event::Pressure(_) => {}
            Event::Reset => engine.reset().await?,
            Event::AssignLayout(e) => engine.assign_layout(e).await?,
            Event::SelectLayout(e) => engine.select_layout(e).await?,
            Event::UpdateNote(e) => engine.update_note(e).await?,
            Event::PlayNote(_) => {}
        }
    }
    Ok(())
}
