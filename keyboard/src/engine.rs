use crate::config::Config;
#[cfg(feature = "csound")]
use crate::csound;
#[cfg(test)]
use crate::events::TestEvent;
use crate::events::{
    Color, EngineState, Event, KeyData, KeyEvent, LayoutNamesEvent, LightData, LightEvent,
    PlayNoteEvent, RawLightEvent, SelectLayoutEvent, ShiftKeyState, ShiftLayoutState, SpecificNote,
    ToDevice, TransposeState, UpdateNoteEvent,
};
use crate::layout::{HorizVert, Layout, RowCol};
use crate::scale::{Note, ScaleType};
use crate::{events, midi_player};
use anyhow::{anyhow, bail};
use chrono::SubsecRound;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use syntoniq_common::pitch::{Factor, Pitch};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum SoundType {
    None,
    Midi,
    #[cfg(feature = "csound")]
    Csound,
}

#[derive(Debug, Clone)]
pub struct PlayedNote {
    pub note: Arc<Note>,
    pub velocity: u8,
}

struct Engine {
    config_file: PathBuf,
    events_tx: events::WeakSender,
    transient_state: EngineState,
}

impl Engine {
    async fn reset(&mut self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        tx.send(Event::ResetDevice)?;
        let config =
            Config::load(&self.config_file).map_err(|e| anyhow!("error reloading config: {e}"))?;
        let mut names = Vec::new();
        for layout in &config.layouts {
            names.push(layout.read().await.name.clone());
        }
        tx.send(Event::SetLayoutNames(LayoutNamesEvent { names }))?;

        // Turn off all notes
        for (pitch, count) in &self.transient_state.pitch_on_count {
            if *count > 0 {
                tx.send(Event::PlayNote(PlayNoteEvent {
                    pitch: pitch.clone(),
                    velocity: 0,
                    note: None,
                }))?;
            }
        }
        self.transient_state = Default::default();
        self.transient_state.layouts = config.layouts;

        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::ResetComplete);
        log::info!("Syntoniq Keyboard is initialized");
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity } = key_event;
        let off = velocity == 0;
        let have_layout = self.transient_state.layout.is_some();
        if !off && matches!(self.transient_state.shift_key_state, ShiftKeyState::Down) {
            // Update shift state -- see below for behavior of shift key.
            self.set_shift(ShiftKeyState::On, &tx)?;
        }
        match key {
            KeyData::Shift => {
                if have_layout {
                    // Behavior of shift key:
                    // - When pressed, state transitions from Off to Down
                    // - If any other key is pressed while in Down state, state transitions to On
                    // - If shift is released when Down, it changes to On
                    // - If shift is pressed or released when On, it changes to Off
                    // Effect: pressing and releasing the shift key without touching other keys toggles
                    // its state. Touching another key while holding it makes it act like a modifier.
                    let shift = match (self.transient_state.shift_key_state, off) {
                        (ShiftKeyState::Off, false) => ShiftKeyState::Down,
                        (ShiftKeyState::Down, true) => ShiftKeyState::On,
                        (ShiftKeyState::On, _) => ShiftKeyState::Off,
                        _ => self.transient_state.shift_key_state,
                    };
                    self.set_shift(shift, &tx)?;
                }
            }
            KeyData::Layout { idx } => {
                if off
                    && let Some((idx, layout)) = self
                        .transient_state
                        .layouts
                        .get(idx)
                        .map(|layout| (idx, layout.clone()))
                {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { idx, layout }))?;
                }
            }
            KeyData::Clear => {
                if off {
                    tx.send(Event::Reset)?;
                }
            }
            KeyData::Sustain => {
                if off {
                    self.transient_state.sustain = !self.transient_state.sustain;
                    tx.send(self.sustain_light_event())?;
                }
            }
            KeyData::Transpose => {
                if off && have_layout {
                    self.transient_state.transpose_state =
                        match self.transient_state.transpose_state {
                            TransposeState::Off => TransposeState::Pending {
                                initial_layout: self.transient_state.layout.unwrap(),
                            },
                            _ => TransposeState::Off,
                        };
                    tx.send(self.transpose_light_event())?;
                }
            }
            KeyData::OctaveShift { up } => {
                if off && have_layout {
                    // 2025-07-22, rust 1.88: "if let guards" are experimental. When stable, we can
                    // use one instead of is_some above and get rid of this unwrap.
                    let layout = self.transient_state.current_layout().unwrap();
                    {
                        let locked = &mut *layout.write().await;
                        let transposition = if up {
                            Pitch::new(vec![Factor::new(2, 1, 1, 1)?])
                        } else {
                            Pitch::new(vec![Factor::new(1, 2, 1, 1)?])
                        };
                        locked.scale.transpose(&transposition);
                    }
                    tx.send(Event::SelectLayout(SelectLayoutEvent {
                        idx: self.transient_state.layout.unwrap(),
                        layout,
                    }))?;
                }
            }
            KeyData::Print => {
                if off && have_layout {
                    self.print_notes();
                }
            }
            KeyData::Note { position } => {
                if have_layout
                    && self.transient_state.notes.contains_key(&position)
                    && let Some(note) = self.transient_state.notes.get(&position).unwrap()
                {
                    self.handle_note_key(&tx, note.clone(), position, off)
                        .await?;
                };
            }
        }
        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::HandledKey);
        Ok(())
    }

    fn print_notes(&self) {
        println!(
            "----- Current Notes ({}) -----",
            chrono::offset::Local::now().trunc_subsecs(0)
        );
        for s in self.transient_state.current_played_notes() {
            println!("{s}");
        }
    }

    async fn handle_note_key(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        position: u8,
        off: bool,
    ) -> anyhow::Result<()> {
        let is_transpose = !matches!(self.transient_state.transpose_state, TransposeState::Off);
        let is_shift = !matches!(self.transient_state.shift_key_state, ShiftKeyState::Off);
        let mut play_note = !is_transpose && !is_shift;
        let layout = self
            .transient_state
            .layout
            .expect("handle_note_key called without current layout");
        if is_transpose {
            let note = note.clone();
            match self.transient_state.transpose_state.clone() {
                TransposeState::Off => unreachable!(),
                TransposeState::Pending { initial_layout } => {
                    if !off {
                        self.transient_state.transpose_state = TransposeState::FirstSelected {
                            initial_layout,
                            note1: SpecificNote {
                                layout_idx: layout,
                                note,
                                position,
                            },
                        };
                    }
                }
                TransposeState::FirstSelected {
                    initial_layout,
                    note1,
                } => {
                    if !off {
                        self.handle_transpose(
                            initial_layout,
                            note1,
                            SpecificNote {
                                layout_idx: layout,
                                note,
                                position,
                            },
                        )
                        .await?;
                    }
                }
            }
        } else if is_shift && !off {
            self.handle_shift(
                layout,
                SpecificNote {
                    layout_idx: layout,
                    note: note.clone(),
                    position,
                },
            )
            .await?;
        }
        if is_transpose
            && matches!(
                self.transient_state.transpose_state,
                TransposeState::FirstSelected { .. }
            )
        {
            play_note = true;
        }
        if play_note {
            self.handle_note_key_normal(tx, note, position, off)?;
        }

        if is_transpose && let Some(tx) = self.events_tx.upgrade() {
            tx.send(self.transpose_light_event())?;
        }

        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::HandledNote);
        Ok(())
    }

    async fn handle_shift(&mut self, layout_idx: usize, note: SpecificNote) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let mut update_layout = false;
        match self.transient_state.shift_layout_state.clone() {
            ShiftLayoutState::Off => {
                self.transient_state.shift_layout_state = ShiftLayoutState::FirstSelected(note);
            }
            ShiftLayoutState::FirstSelected(note1) => {
                self.transient_state.shift_layout_state = ShiftLayoutState::Off;
                if note1.layout_idx != layout_idx {
                    log::info!("move: note1 and note2 are from different layouts, so not shifting");
                    #[cfg(test)]
                    events::send_test_event(&self.events_tx, TestEvent::MoveCanceled);
                } else {
                    let mut layout = self.transient_state.layouts[layout_idx].write().await;
                    if let Some(base) = layout.base {
                        let note1_col = note1.position % 10;
                        let note1_row = note1.position / 10;
                        let note2_col = note.position % 10;
                        let note2_row = note.position / 10;
                        let dy = note2_row as i8 - note1_row as i8;
                        let dx = note2_col as i8 - note1_col as i8;
                        log::info!("shifting layout {} by dy={dy}, dx={dx}", layout.name);
                        let RowCol {
                            col: old_x,
                            row: old_y,
                        } = base;
                        layout.base = Some(RowCol {
                            col: old_x + dx,
                            row: old_y + dy,
                        });
                        update_layout = true;
                    } else {
                        log::info!("move: can't shift non-EDO layout");
                        #[cfg(test)]
                        events::send_test_event(&self.events_tx, TestEvent::MoveCanceled);
                    };
                }
            }
        };

        if update_layout {
            tx.send(Event::SelectLayout(SelectLayoutEvent {
                idx: layout_idx,
                layout: self.transient_state.layouts[layout_idx].clone(),
            }))?;
        }
        Ok(())
    }

    async fn handle_transpose(
        &mut self,
        initial_layout: usize,
        note1: SpecificNote,
        note2: SpecificNote,
    ) -> anyhow::Result<()> {
        let mut update_layout = false;
        if note1.note == note2.note {
            self.transient_state.transpose_state = TransposeState::Off;
            let mut layout = self.transient_state.layouts[initial_layout].write().await;
            log::info!(
                "resetting base pitch of {} to {}",
                layout.scale.name,
                note1.note.pitch
            );
            layout.scale.base_pitch = note1.note.pitch.clone();
            update_layout = true;
        } else {
            // Reset note1 to current note
            self.transient_state.transpose_state = TransposeState::FirstSelected {
                initial_layout,
                note1: note2,
            };
        }
        if update_layout && let Some(tx) = self.events_tx.upgrade() {
            tx.send(Event::SelectLayout(SelectLayoutEvent {
                idx: initial_layout,
                layout: self.transient_state.layouts[initial_layout].clone(),
            }))?;
        }
        Ok(())
    }

    fn note_light_event(note: &Note, position: u8, velocity: u8) -> Event {
        let appearance = note.appearance(velocity != 0);
        Event::ToDevice(ToDevice::Light(RawLightEvent {
            position,
            color: appearance.color,
            label1: appearance.name.to_string(),
            label2: appearance.description.to_string(),
        }))
    }

    fn handle_note_key_normal(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        position: u8,
        off: bool,
    ) -> anyhow::Result<()> {
        let pitch = &note.pitch;
        let Some(others) = self.transient_state.pitch_positions.get(pitch) else {
            // This would indicate a bug in which we assigned something to notes
            // without also assigning its position to note positions or otherwise
            // allowed notes and note_positions to get out of sync.
            bail!("note positions is missing for {pitch}");
        };
        if off {
            let old = self.transient_state.positions_down.remove(&position);
            if old.is_none() {
                // We got key off event on a note square, but the note was not on. This happens
                // if you touch a key that has no note, and while continuing to touch the key,
                // select a new layout where that key does have a note. Just ignore the event.
                return Ok(());
            }
        } else {
            self.transient_state
                .positions_down
                .insert(position, note.clone());
        }

        let pitch_count = self
            .transient_state
            .pitch_on_count
            .entry(pitch.clone())
            .or_default();
        // When not in sustain mode, touch turns a note on, and release turns it off.
        // Since the same note may appear in multiple locations, we keep a count, and on
        // send a note event if we transition to or from 0. In sustain mode, "off"
        // events are ignored. Touching a note in any of its positions toggles whether
        // it's on or off. Changing scales, transposing, shifting, etc. doesn't affect
        // which notes are on or off, making it possible to play a note in one scale,
        // switch scales, and play a note in another scale.
        let was_on = *pitch_count > 0;
        if self.transient_state.sustain {
            if !off {
                if *pitch_count > 0 {
                    *pitch_count = 0;
                } else {
                    *pitch_count = 1;
                }
            }
        } else if off {
            if *pitch_count > 0 {
                *pitch_count -= 1
            }
        } else {
            *pitch_count += 1;
        }
        let pitch_count = *pitch_count;
        if pitch_count == 0 {
            self.transient_state.pitch_on_count.remove(pitch);
            self.transient_state.last_note_for_pitch.remove(pitch);
        } else {
            self.transient_state
                .last_note_for_pitch
                .insert(pitch.clone(), note.clone());
        }
        let is_on = pitch_count > 0;
        if is_on != was_on {
            let velocity = if is_on { 127 } else { 0 };
            for position in others.iter().copied() {
                tx.send(Self::note_light_event(&note, position, velocity))?;
            }
            tx.send(Event::PlayNote(PlayNoteEvent {
                pitch: pitch.clone(),
                velocity,
                note: Some(note.clone()),
            }))?;
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
        self.transient_state
            .notes
            .insert(position, played_note.as_ref().map(|x| x.note.clone()));
        match played_note {
            Some(played_note) => {
                let note = played_note.note.clone();
                self.transient_state
                    .pitch_positions
                    .entry(note.pitch.clone())
                    .or_default()
                    .insert(position);
                tx.send(Self::note_light_event(
                    &note,
                    position,
                    played_note.velocity,
                ))?;
            }
            None => {
                tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
                    position,
                    color: Color::Off,
                    label1: String::new(),
                    label2: String::new(),
                })))?;
            }
        }
        Ok(())
    }

    async fn handle_play_note(&mut self, e: PlayNoteEvent) -> anyhow::Result<()> {
        if let Some(note) = e.note
            && e.velocity > 0
        {
            println!("{note}");
        }
        Ok(())
    }

    fn send_note(
        &self,
        tx: &events::UpgradedSender,
        row: i8,
        col: i8,
        note: Arc<Note>,
    ) -> anyhow::Result<()> {
        let velocity = if self
            .transient_state
            .pitch_on_count
            .get(&note.pitch)
            .copied()
            .unwrap_or_default()
            > 0
        {
            127
        } else {
            0
        };
        let played_note = Some(PlayedNote { note, velocity });
        let position = (10 * row + col) as u8;
        tx.send(Event::UpdateNote(UpdateNoteEvent {
            position,
            played_note,
        }))?;
        Ok(())
    }

    async fn draw_edo_layout(&mut self, layout: &mut Layout) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::EqualDivision(ed) = &layout.scale.scale_type else {
            // Should not be possible
            bail!("draw_edo_layout called with non-EDO scale");
        };
        let HorizVert {
            h: steps_x,
            v: steps_y,
        } = layout.steps.unwrap(); // checked to be Some in config
        let RowCol {
            col: base_x,
            row: base_y,
        } = layout.base.unwrap(); // checked to be Some in config
        let (divisions, _, _) = ed.divisions;
        let divisions = divisions as i32;
        for row in 1..=8 {
            for col in 1..=8 {
                let steps = (steps_x * (col - base_x) + steps_y * (row - base_y)) as i32;
                let cycle = steps / divisions;
                let step = steps % divisions;
                let note = layout.scale.edo_note(cycle as i8, step as i8);
                self.send_note(&tx, row, col, Arc::new(note))?;
            }
        }
        Ok(())
    }

    async fn draw_generic_layout(&mut self, layout: &mut Layout) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let ScaleType::Generic(g) = &layout.scale.scale_type else {
            // Should not be possible
            bail!("draw_generic_layout called with non-Generic scale");
        };
        let mut cache = HashMap::new();
        for row in 1..=8 {
            for col in 1..=8 {
                if let Some(note) = layout.scale.generic_note(&mut cache, g, row, col)? {
                    self.send_note(&tx, row, col, note)?;
                } else {
                    tx.send(Event::ToDevice(ToDevice::Light(RawLightEvent {
                        position: (10 * row + col) as u8,
                        color: Color::Off,
                        label1: "".to_string(),
                        label2: "".to_string(),
                    })))?;
                }
            }
        }
        Ok(())
    }

    fn toggle_light_event(&self, on: bool, light: LightData, label1: &str, label2: &str) -> Event {
        let color = if on {
            Color::ToggleOn
        } else {
            Color::ToggleOff
        };
        Event::LightEvent(LightEvent {
            light,
            color,
            label1: label1.to_string(),
            label2: label2.to_string(),
        })
    }

    fn transpose_light_event(&self) -> Event {
        let color = match self.transient_state.transpose_state {
            TransposeState::Off => Color::ToggleOff,
            TransposeState::Pending { .. } => Color::ToggleOn,
            TransposeState::FirstSelected { .. } => Color::NoteSelected,
        };
        Event::LightEvent(LightEvent {
            light: LightData::Transpose,
            color,
            label1: "Transpose".to_string(),
            label2: String::new(),
        })
    }

    fn sustain_light_event(&self) -> Event {
        self.toggle_light_event(
            self.transient_state.sustain,
            LightData::Sustain,
            "Sustain",
            "",
        )
    }

    fn set_shift(
        &mut self,
        shift: ShiftKeyState,
        tx: &events::UpgradedSender,
    ) -> anyhow::Result<()> {
        if shift == self.transient_state.shift_key_state {
            return Ok(());
        }
        self.transient_state.shift_key_state = shift;
        if matches!(shift, ShiftKeyState::Off) {
            self.transient_state.shift_layout_state = ShiftLayoutState::Off;
        }
        tx.send(self.shift_light_event())?;
        Ok(())
    }

    fn shift_light_event(&self) -> Event {
        let color = match self.transient_state.shift_key_state {
            ShiftKeyState::Off => Color::ToggleOff,
            _ => Color::ToggleOn,
        };
        Event::LightEvent(LightEvent {
            light: LightData::Shift,
            color,
            label1: "Shift".to_string(),
            label2: String::new(),
        })
    }

    async fn select_layout(&mut self, event: SelectLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // For any keys that are held down, act like we released them. We will send new key events
        // at the end. This creates better behavior if you select a new layout (including octave
        // shift) while holding keys down.
        let notes_down = self.transient_state.positions_down.clone();
        let note_positions_before: HashSet<u8> = notes_down.keys().copied().collect();
        for (position, note) in notes_down {
            self.handle_note_key_normal(&tx, note, position, true)?;
        }
        let layout_lock = event.layout;
        self.transient_state.layout = Some(event.idx);
        let layout = &mut *layout_lock.write().await;
        self.transient_state.pitch_positions.clear();
        self.transient_state.notes.clear();
        match layout.scale.scale_type {
            ScaleType::EqualDivision(_) => self.draw_edo_layout(layout).await?,
            ScaleType::Generic(_) => self.draw_generic_layout(layout).await?,
        }
        log::info!(
            "selected layout: {}, scale: {}",
            layout.name,
            layout.scale.name
        );
        tx.send(self.sustain_light_event())?;
        tx.send(self.transpose_light_event())?;
        tx.send(self.shift_light_event())?;
        // Re-touch all the notes that we previously untouched.
        for position in note_positions_before {
            tx.send(Event::KeyEvent(KeyEvent {
                velocity: 127,
                key: KeyData::Note { position },
            }))?;
        }
        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::LayoutSelected);
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<bool> {
        match event {
            Event::Shutdown => return Ok(true),
            Event::Reset => self.reset().await?,
            Event::KeyEvent(e) => self.handle_key(e).await?,
            Event::SelectLayout(e) => self.select_layout(e).await?,
            Event::UpdateNote(e) => self.update_note(e).await?,
            Event::PlayNote(e) => self.handle_play_note(e).await?,
            Event::ResetDevice
            | Event::LightEvent(_)
            | Event::ToDevice(_)
            | Event::SetLayoutNames(_) => {}
            #[cfg(test)]
            Event::TestEngine(test_tx) => test_tx.send(self.transient_state.clone()).await?,
            #[cfg(test)]
            Event::TestSync => events::send_test_event(&self.events_tx, TestEvent::Sync),
            #[cfg(test)]
            Event::TestWeb(_) | Event::TestEvent(_) => {}
        };
        Ok(false)
    }
}

pub async fn run(
    config_file: PathBuf,
    sound_type: SoundType,
    events_tx: events::WeakSender,
    mut events_rx: events::Receiver,
) -> anyhow::Result<()> {
    let mut engine = Engine {
        config_file,
        events_tx: events_tx.clone(),
        transient_state: Default::default(),
    };
    let rx2 = events_rx.resubscribe();
    match sound_type {
        SoundType::None => {}
        SoundType::Midi => {
            tokio::spawn(async move {
                if let Err(e) = midi_player::play_midi(rx2).await {
                    log::error!("midi player error: {e}");
                };
            });
        }
        #[cfg(feature = "csound")]
        SoundType::Csound => {
            let tx2 = events_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = csound::run_csound(rx2, tx2).await {
                    log::error!("csound player error: {e}");
                };
            });
        }
    }
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::Reset)?;
    }
    while let Some(event) = events::receive_check_lag(&mut events_rx, Some("engine")).await {
        // Note: this event loop calls event handlers inline. Sometimes those event handlers
        // generate other events, which are piling up in our queue while we are handling earlier
        // events. As long as the backlog on the event receiver is high enough and/or we don't
        // care about missing some messages, we should be fine. In practice, the messages we would
        // be most likely to miss our the ones we just sent, but we could also miss other people's
        // responses to those. We are not as likely to miss user events.
        match engine.handle_event(event).await {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(e) => {
                log::error!("error handling event: {e}");
            }
        };
    }
    Ok(())
}
