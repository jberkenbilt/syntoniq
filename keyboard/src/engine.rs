use crate::controller::{Controller, Device};
#[cfg(feature = "csound")]
use crate::csound;
#[cfg(test)]
use crate::events::TestEvent;
use crate::events::{
    Color, EngineState, Event, FromDevice, KeyData, KeyEvent, LayoutNamesEvent, LightData,
    LightEvent, Note, NoteColors, PlayNoteEvent, RawLightEvent, SelectLayoutEvent, SpecificNote,
    ToDevice, UpdateNoteEvent,
};
use crate::{events, midi_player};
use anyhow::bail;
use chrono::SubsecRound;
use std::collections::HashSet;
use std::fs;
use std::ops::Deref;
use std::sync::Arc;
use syntoniq_common::parsing;
use syntoniq_common::parsing::{Coordinate, Layout, Layouts};
use tokio::task;
use tokio::task::JoinHandle;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum SoundType {
    None,
    Midi,
    #[cfg(feature = "csound")]
    Csound,
}

struct Engine {
    score_file: String,
    keyboard: Arc<dyn Keyboard>,
    events_tx: events::WeakSender,
    transient_state: EngineState,
}

fn load_layouts(score_file: &str) -> anyhow::Result<Layouts<'static>> {
    let data = fs::read(score_file)?;
    let src = str::from_utf8(&data)?;
    parsing::layouts(score_file, src, &parsing::Options::default())
}

pub trait Keyboard: Sync + Send {
    fn reset(&self) -> anyhow::Result<()>;
    fn multiple_keyboards(&self) -> bool;
    fn layout_supported(&self, layout: &Layout) -> bool;
    fn note_positions(&self, keyboard: &str) -> &'static [Coordinate];
    fn note_light_event(
        &self,
        note: Option<&Note>,
        position: Coordinate,
        velocity: u8,
    ) -> RawLightEvent;
    fn make_device(&self) -> Arc<dyn Device>;
    fn handle_raw_event(&self, msg: FromDevice) -> anyhow::Result<()>;
    fn main_event_loop(&self, event: Event) -> anyhow::Result<()>;
}

/// See toggle_move for interpretation of fields
struct ToggleMoveResult {
    changed: bool,
    val: Option<Option<SpecificNote>>,
}

impl Engine {
    async fn reset(&mut self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        self.keyboard.reset()?;
        let mut layouts = load_layouts(&self.score_file)?;
        layouts
            .layouts
            .retain(|layout| self.keyboard.layout_supported(layout));
        let multiple_keyboards = self.keyboard.multiple_keyboards();
        let names = layouts
            .layouts
            .iter()
            .map(|layout| {
                let mut name = layout.name.to_string();
                if multiple_keyboards {
                    name.push_str(&format!(" ({})", layout.keyboard));
                }
                name
            })
            .collect();
        tx.send(Event::SetLayoutNames(LayoutNamesEvent { names }))?;

        // TODO: have an event to turn off all notes; midi player send 120, 0.
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
        self.transient_state.layouts = Arc::new(layouts);

        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::ResetComplete);
        log::info!("Syntoniq Keyboard is initialized");
        Ok(())
    }

    fn send_transpose_light_event(&self, tx: &events::UpgradedSender) -> anyhow::Result<()> {
        Self::send_move_light_event(
            tx,
            &self.transient_state.transpose,
            LightData::Transpose,
            "Transpose",
        )
    }

    fn send_shift_light_event(&self, tx: &events::UpgradedSender) -> anyhow::Result<()> {
        Self::send_move_light_event(tx, &self.transient_state.shift, LightData::Shift, "Shift")
    }

    fn send_move_light_event(
        tx: &events::UpgradedSender,
        item: &Option<Option<SpecificNote>>,
        light: LightData,
        label: &str,
    ) -> anyhow::Result<()> {
        let color = match item {
            None => Color::ToggleOff,
            Some(None) => Color::ToggleOn,
            Some(Some(_)) => Color::NoteSelected,
        };
        tx.send(Event::LightEvent(LightEvent {
            light,
            color,
            label1: label.to_string(),
            label2: String::new(),
        }))?;
        Ok(())
    }

    fn toggle_move(&self, off: bool, old: Option<Option<SpecificNote>>) -> ToggleMoveResult {
        // A key down event turns on the behavior when off. A key up event turns off the behavior
        // when active. This allows move trigger buttons to act as modifier keys.

        let handle = match old {
            None => {
                // We are not in move mode. Enter on a down event
                !off
            }
            Some(None) => {
                // We are in move mode but haven't selected the first note. We have to ignore
                // key up events so we don't cancel just by releasing the button.
                !off
            }
            Some(Some(_)) => {
                // We have selected the first note and are canceling. Do this on a key up event.
                off
            }
        };
        if handle {
            // Move operations store state in Option<Option<SpecificNote>>. If None, no operation is in
            // flight. If Some(None), the operation has been triggered, but the first note has not been
            // selected. If Some(Some(_)), the value is the first key. This toggles a potentially
            // in-flight event.
            let val = match old {
                None => Some(None),
                Some(_) => None,
            };
            ToggleMoveResult { changed: true, val }
        } else {
            ToggleMoveResult {
                changed: false,
                val: old,
            }
        }
    }

    fn cancel_shift(&mut self, tx: &events::UpgradedSender) -> anyhow::Result<()> {
        if self.transient_state.shift.take().is_some() {
            self.send_shift_light_event(tx)?;
        }
        Ok(())
    }

    fn cancel_transpose(&mut self, tx: &events::UpgradedSender) -> anyhow::Result<()> {
        if self.transient_state.transpose.take().is_some() {
            self.send_transpose_light_event(tx)?;
        }
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity } = key_event;
        let off = velocity == 0;
        let have_layout = self.transient_state.layout.is_some();
        match key {
            KeyData::Shift => {
                // Shift and Transpose have identical logic.
                self.cancel_transpose(&tx)?;
                if have_layout {
                    let old = self.transient_state.shift.take();
                    let r = self.toggle_move(off, old);
                    self.transient_state.shift = r.val;
                    if r.changed {
                        self.send_shift_light_event(&tx)?;
                    }
                }
            }
            KeyData::Transpose => {
                // Shift and Transpose have identical logic.
                self.cancel_shift(&tx)?;
                if have_layout {
                    let old = self.transient_state.transpose.take();
                    let r = self.toggle_move(off, old);
                    self.transient_state.transpose = r.val;
                    if r.changed {
                        self.send_transpose_light_event(&tx)?;
                    }
                }
            }
            KeyData::Layout { idx } => {
                self.cancel_shift(&tx)?;
                if off
                    && let Some((idx, layout)) = self
                        .transient_state
                        .layouts
                        .layouts
                        .get(idx)
                        .map(|layout| (idx, layout.clone()))
                {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { idx, layout }))?;
                }
            }
            KeyData::Reset => {
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
            KeyData::OctaveShift { up } => {
                if off && let Some(layout) = self.transient_state.current_layout() {
                    layout.octave_shift(up);
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
        position: Coordinate,
        off: bool,
    ) -> anyhow::Result<()> {
        let layout = self
            .transient_state
            .layout
            .expect("handle_note_key called without current layout");
        let this_note = SpecificNote {
            layout_idx: layout,
            note: note.clone(),
            position,
        };
        if self.transient_state.transpose.is_some() {
            if off {
                match self.transient_state.transpose.take().unwrap() {
                    None => {
                        self.transient_state.transpose = Some(Some(this_note));
                    }
                    Some(note1) => {
                        self.handle_transpose(note1, this_note)?;
                    }
                }
                self.send_transpose_light_event(tx)?;
            }
        } else if self.transient_state.shift.is_some() {
            if off {
                match self.transient_state.shift.take().unwrap() {
                    None => {
                        self.transient_state.shift = Some(Some(this_note));
                    }
                    Some(note1) => {
                        self.handle_shift(note1, this_note)?;
                    }
                }
                self.send_shift_light_event(tx)?;
            }
        } else {
            self.handle_note_key_normal(tx, note, position, off)?;
        }

        #[cfg(test)]
        events::send_test_event(&self.events_tx, TestEvent::HandledNote);
        Ok(())
    }

    fn handle_shift(&mut self, note1: SpecificNote, note2: SpecificNote) -> anyhow::Result<()> {
        if note1.layout_idx != note2.layout_idx {
            log::info!("shift: note1 and note2 are from different layouts, so not shifting");
            #[cfg(test)]
            events::send_test_event(&self.events_tx, TestEvent::MoveCanceled);
            return Ok(());
        }
        let layout = &self.transient_state.layouts.layouts[note2.layout_idx];
        let update_layout = layout.shift(note1.position, note2.position);
        if update_layout && let Some(tx) = self.events_tx.upgrade() {
            tx.send(Event::SelectLayout(SelectLayoutEvent {
                idx: note2.layout_idx,
                layout: layout.clone(),
            }))?;
        } else {
            log::info!("shift: start and end keys must be in the same mapping");
            #[cfg(test)]
            events::send_test_event(&self.events_tx, TestEvent::MoveCanceled);
        }
        Ok(())
    }

    fn handle_transpose(&mut self, note1: SpecificNote, note2: SpecificNote) -> anyhow::Result<()> {
        // Give pitch of note1 to note2
        let layout = &self.transient_state.layouts.layouts[note2.layout_idx];
        let update_layout = layout.transpose(&note1.note.placed.pitch, note2.position);
        if update_layout {
            log::info!(
                "reset pitch of {} to {}",
                note2.note.placed.name,
                note1.note.placed.pitch
            );
            if let Some(tx) = self.events_tx.upgrade() {
                tx.send(Event::SelectLayout(SelectLayoutEvent {
                    idx: note2.layout_idx,
                    layout: layout.clone(),
                }))?;
            }
        } else {
            log::info!("transpose operation failed");
        }
        Ok(())
    }

    fn handle_note_key_normal(
        &mut self,
        tx: &events::UpgradedSender,
        note: Arc<Note>,
        position: Coordinate,
        off: bool,
    ) -> anyhow::Result<()> {
        let pitch = &note.placed.pitch;
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
            let light_events: Vec<_> = others
                .iter()
                .map(|&position| {
                    self.keyboard
                        .note_light_event(Some(note.deref()), position, velocity)
                })
                .collect();
            tx.send(Event::ToDevice(ToDevice::Light(light_events)))?;
            tx.send(Event::PlayNote(PlayNoteEvent {
                pitch: pitch.clone(),
                velocity,
                note: Some(note.clone()),
            }))?;
        }
        Ok(())
    }

    async fn update_note(&mut self, event: UpdateNoteEvent) -> anyhow::Result<()> {
        let UpdateNoteEvent { position, note } = event;
        self.transient_state.notes.insert(position, note.clone());
        if let Some(note) = note {
            self.transient_state
                .pitch_positions
                .entry(note.placed.pitch.clone())
                .or_default()
                .insert(position);
        }
        Ok(())
    }

    async fn handle_play_note(&mut self, e: PlayNoteEvent) -> anyhow::Result<()> {
        if let Some(note) = e.note
            && e.velocity > 0
        {
            if self.transient_state.sustain {
                self.print_notes();
            } else {
                println!("{note}");
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

    fn sustain_light_event(&self) -> Event {
        self.toggle_light_event(
            self.transient_state.sustain,
            LightData::Sustain,
            "Sustain",
            "",
        )
    }

    fn send_note(
        &self,
        tx: &events::UpgradedSender,
        position: Coordinate,
        note: Option<Arc<Note>>,
    ) -> anyhow::Result<RawLightEvent> {
        let velocity = note
            .as_ref()
            .map(|note| {
                if self
                    .transient_state
                    .pitch_on_count
                    .get(&note.placed.pitch)
                    .copied()
                    .unwrap_or_default()
                    > 0
                {
                    127
                } else {
                    0
                }
            })
            .unwrap_or_default();
        let light_event = self
            .keyboard
            .note_light_event(note.as_deref(), position, velocity);
        tx.send(Event::UpdateNote(UpdateNoteEvent { position, note }))?;
        Ok(light_event)
    }
    fn draw_layout(&self, layout: &Arc<Layout<'static>>) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let mut light_events = Vec::new();
        for &location in self.keyboard.note_positions(&layout.keyboard) {
            let note = layout.note_at_location(location).map(|placed| {
                let colors = if placed.isomorphic && placed.degree == 1 {
                    NoteColors {
                        off: Color::SingleStepOff,
                        on: Color::SingleStepOn,
                    }
                } else {
                    events::interval_color(placed.base_interval.as_float())
                };
                Arc::new(Note {
                    placed,
                    off_color: colors.off,
                    on_color: colors.on,
                })
            });
            light_events.push(self.send_note(&tx, location, note)?);
        }
        tx.send(Event::ToDevice(ToDevice::Light(light_events)))?;
        Ok(())
    }

    async fn select_layout(&mut self, event: SelectLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // For any keys that are held down, act like we released them. We will send new key events
        // at the end. This creates better behavior if you select a new layout (including octave
        // shift) while holding keys down.
        let notes_down = self.transient_state.positions_down.clone();
        let note_positions_before: HashSet<_> = notes_down.keys().copied().collect();
        for (position, note) in notes_down {
            self.handle_note_key_normal(&tx, note, position, true)?;
        }
        self.transient_state.layout = Some(event.idx);
        self.transient_state.pitch_positions.clear();
        self.transient_state.notes.clear();
        self.draw_layout(&event.layout)?;
        log::info!("selected layout: {}", event.layout.name,);
        tx.send(self.sustain_light_event())?;
        self.send_transpose_light_event(&tx)?;
        self.send_shift_light_event(&tx)?;
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
        log::trace!("engine handle event: {event:?}");
        match event {
            Event::Shutdown => return Ok(true),
            Event::Reset => self.reset().await?,
            Event::KeyEvent(e) => self.handle_key(e).await?,
            Event::SelectLayout(e) => self.select_layout(e).await?,
            Event::UpdateNote(e) => self.update_note(e).await?,
            Event::PlayNote(e) => self.handle_play_note(e).await?,
            Event::LightEvent(_) | Event::ToDevice(_) | Event::SetLayoutNames(_) => {}
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

pub async fn start_controller(
    keyboard: Arc<dyn Keyboard>,
    controller: Controller,
    mut events_rx: events::Receiver,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    // Communicating with the MIDI device must be sync. The rest of the application must be
    // async. To bridge the gap, we create flume channels to relay back and forth.
    let (to_device_tx, to_device_rx) = flume::unbounded::<ToDevice>();
    let (from_device_tx, from_device_rx) = flume::unbounded::<FromDevice>();
    tokio::spawn(async move {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("controller")).await
        {
            let Event::ToDevice(event) = event else {
                continue;
            };
            if let Err(e) = to_device_tx.send_async(event).await {
                log::error!("failed to relay message to device: {e}");
            }
        }
    });
    let device = keyboard.make_device();
    tokio::spawn(async move {
        while let Ok(msg) = from_device_rx.recv_async().await {
            if let Err(e) = keyboard.handle_raw_event(msg) {
                log::error!("error handling raw Launchpad event: {e}");
            }
        }
    });
    controller.run(to_device_rx, from_device_tx, device)
}

pub async fn start_keyboard(
    controller: Option<Controller>,
    keyboard: Arc<dyn Keyboard>,
    mut events_rx: events::Receiver,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let controller_h = match controller {
        None => None,
        Some(c) => {
            // Start controller doesn't return until the device is initialized.
            Some(start_controller(keyboard.clone(), c, events_rx.resubscribe()).await?)
        }
    };
    // Start the background task after the device is initialized so we're fully up before this
    // function returns.
    Ok(task::spawn(async move {
        while let Some(event) = events::receive_check_lag(&mut events_rx, Some("engine")).await {
            keyboard.main_event_loop(event)?;
        }
        if let Some(h) = controller_h {
            h.await??;
        }
        Ok(())
    }))
}

pub async fn run(
    score_file: &str,
    sound_type: SoundType,
    keyboard: Arc<dyn Keyboard>,
    events_tx: events::WeakSender,
    mut events_rx: events::Receiver,
) -> anyhow::Result<()> {
    let mut engine = Engine {
        score_file: score_file.to_string(),
        keyboard,
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
                log::error!("error: {e}");
            }
        };
    }
    Ok(())
}
