use super::*;
use crate::events::{Events, StateView};
use crate::view::web;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct TestController {
    events_tx: events::UpgradedSender,
    events_rx: events::Receiver,
    engine_channel: EngineChannel,
    web_channel: WebChannel,
    engine_handle: JoinHandle<anyhow::Result<()>>,
    web_handle: JoinHandle<()>,
}
impl TestController {
    async fn new() -> Self {
        let _ = env_logger::try_init();
        let events = Events::new();
        let events_tx_weak = events.sender().await;
        let events_tx = events_tx_weak.upgrade().unwrap();
        let events_rx = events.receiver();
        let engine_channel = EngineChannel::default();
        let web_channel = WebChannel::default();
        let tx2 = events_tx_weak.clone();
        let rx2 = events_rx.resubscribe();
        let engine_handle = tokio::spawn(async move {
            run("testdata/conf.toml".into(), SoundType::None, tx2, rx2).await
        });
        let tx2 = events_tx_weak.clone();
        let rx2 = events_rx.resubscribe();
        let web_handle = tokio::spawn(async move {
            web::http_view(tx2, rx2, 0).await;
        });
        let mut rx2 = events_rx.resubscribe();
        tokio::spawn(async move {
            while let Some(event) = events::receive_check_lag(&mut rx2, None).await {
                log::trace!("event: {event:?}")
            }
        });
        let mut tc = Self {
            events_tx,
            events_rx,
            engine_channel,
            web_channel,
            engine_handle,
            web_handle,
        };
        tc.wait_for_test_event(TestEvent::ResetComplete).await;
        tc
    }

    async fn shutdown(mut self) -> anyhow::Result<()> {
        self.events_tx.send(Event::Shutdown)?;
        while events::receive_check_lag(&mut self.events_rx, None)
            .await
            .is_some()
        {}
        self.web_handle.await?;
        self.engine_handle.await?
    }

    async fn get_engine_state(&mut self) -> EngineState {
        self.events_tx
            .send(Event::TestEngine(self.engine_channel.tx.clone()))
            .unwrap();
        self.engine_channel.rx.recv().await.unwrap()
    }

    async fn get_web_state(&mut self) -> StateView {
        self.events_tx
            .send(Event::TestWeb(self.web_channel.tx.clone()))
            .unwrap();
        self.web_channel.rx.recv().await.unwrap()
    }

    async fn wait_for_event<F>(&mut self, f: F) -> Option<Event>
    where
        F: Fn(&Event) -> bool,
    {
        while let Some(event) = events::receive_check_lag(&mut self.events_rx, None).await {
            if f(&event) {
                return Some(event);
            }
        }
        None
    }

    async fn wait_for_test_event(&mut self, test_event: TestEvent) {
        self.wait_for_event(|e| matches!(e, Event::TestEvent(t) if *t == test_event))
            .await;
    }

    async fn send_key(&mut self, position: u8, on: bool) -> anyhow::Result<()> {
        self.events_tx.send(Event::Key(KeyEvent {
            key: position,
            velocity: if on { 127 } else { 0 },
            synthetic: false,
        }))?;
        assert!(
            self.wait_for_event(|e| matches!(e, Event::Key(_)))
                .await
                .is_some()
        );
        Ok(())
    }

    async fn press_key(&mut self, position: u8) -> anyhow::Result<()> {
        self.send_key(position, true).await
    }

    async fn release_key(&mut self, position: u8) -> anyhow::Result<()> {
        self.send_key(position, false).await
    }

    async fn press_and_release_key(&mut self, position: u8) -> anyhow::Result<()> {
        self.press_key(position).await?;
        self.release_key(position).await
    }
}

pub struct EngineChannel {
    pub tx: mpsc::Sender<EngineState>,
    pub rx: mpsc::Receiver<EngineState>,
}
impl Default for EngineChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self { tx, rx }
    }
}

pub struct WebChannel {
    pub tx: mpsc::Sender<StateView>,
    pub rx: mpsc::Receiver<StateView>,
}
impl Default for WebChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self { tx, rx }
    }
}

#[tokio::test]
async fn test_test_controller() -> anyhow::Result<()> {
    // Make sure basic test controller startup/shutdown works.
    let mut tc = TestController::new().await;
    let ts = tc.get_engine_state().await;
    assert!(ts.layout.is_none());
    let ws = tc.get_web_state().await;
    assert!(ws.selected_layout.is_empty());
    loop {
        let ws = tc.get_web_state().await;
        if ws.layout_names.is_empty() {
            continue;
        }
        break;
    }
    assert_eq!(ws.layout_names[0], "EDO-12-2x1");
    tc.shutdown().await
}

#[tokio::test]
async fn test_layout_selection() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-12
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(ts.layout.unwrap().read().await.name, "EDO-12-2x1");
    let ws = tc.get_web_state().await;
    assert_eq!(ws.selected_layout, "EDO-12-2x1");
    assert_eq!(ws.base_pitch, "220*^1|4");
    tc.shutdown().await
}

#[tokio::test]
async fn test_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-12
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // Press and release middle C. Note is on after press and off after release.
    tc.press_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    let middle_c = Pitch::must_parse("220*^3|12");
    assert!(*ts.pitch_on_count.get(&middle_c).unwrap() > 0);
    assert_eq!(ts.last_note_for_pitch.get(&middle_c).unwrap().name, "C");
    tc.release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.pitch_on_count.contains_key(&middle_c));

    // Enter sustain mode
    tc.press_and_release_key(keys::SUSTAIN).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.sustain);

    // Press and release middle C. Note stays on.
    tc.press_and_release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(*ts.pitch_on_count.get(&middle_c).unwrap() > 0);

    // Press and release middle C. Note turns off.
    tc.press_and_release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.pitch_on_count.contains_key(&middle_c));

    // Leave sustain mode
    tc.press_and_release_key(keys::SUSTAIN).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.sustain);

    tc.shutdown().await
}

#[tokio::test]
async fn test_shift_key() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout.
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Press and release shift with no other notes.
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    tc.press_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Down));
    tc.release_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Press and release shift again to toggle.
    tc.press_and_release_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    // Press shift key without releasing
    tc.press_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Down));
    // Press some other key
    tc.press_key(11).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Release other key
    tc.release_key(11).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Release shift key
    tc.release_key(keys::SHIFT).await?;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    Ok(())
}

#[tokio::test]
async fn test_transpose_cancels() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // Enter transpose mode
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Touch transpose to cancel
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));

    // Enter transpose mode
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));
    // Touch a note
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    // Transpose key cancels
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));

    tc.shutdown().await
}

#[tokio::test]
async fn test_layout_shift_cancels() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;

    // Start shift
    tc.press_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    // Touch a note
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    // Release shift; cancels operation
    tc.release_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));

    // Select generic layout
    tc.press_and_release_key(105).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.layout.unwrap().read().await.scale.scale_type,
        ScaleType::Generic(_)
    ));
    // Enter shift mode
    tc.press_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Can't shift a generic layout
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(33).await?;
    tc.wait_for_test_event(TestEvent::MoveCanceled).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));

    tc.shutdown().await
}

#[tokio::test]
async fn test_octave_transpose() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.layout.unwrap().read().await.scale.base_pitch.to_string(),
        "220*^1|4"
    );

    // Go down an octave
    tc.press_and_release_key(keys::DOWN_ARROW).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.layout.unwrap().read().await.scale.base_pitch.to_string(),
        "110*^1|4"
    );

    // Go up two octaves
    tc.press_and_release_key(keys::UP_ARROW).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.press_and_release_key(keys::UP_ARROW).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.layout.unwrap().read().await.scale.base_pitch.to_string(),
        "440*^1|4"
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_transpose_same_layout() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(ts.layout.unwrap().read().await.scale.name, "EDO-19");

    // Enter move mode
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Touch a note twice to transpose
    tc.press_and_release_key(88).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    // Touch a different note, changing first note
    tc.press_and_release_key(44).await?;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(44).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    // Transpose up 8 EDO-19 steps: 1/4 + 8/19 = 51/76
    assert_eq!(
        ts.layout.unwrap().read().await.scale.base_pitch.to_string(),
        "220*^51|76"
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_transpose_different_layout() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-12
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(ts.layout.unwrap().read().await.scale.name, "EDO-12");

    // Enter transpose mode
    tc.press_and_release_key(keys::TRANSPOSE).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let edo_19 = ts.layout.clone().unwrap();
    assert_eq!(ts.layout.unwrap().read().await.scale.name, "EDO-19");

    // Touch a note twice to transpose
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    // EDO-19 didn't change, but...
    {
        let layout = edo_19.read().await;
        assert_eq!(layout.scale.name, "EDO-19");
        assert_eq!(layout.scale.base_pitch.to_string(), "220*^1|4");
    }
    // EDO-12 did by 4 EDO-19 steps. 1/4 + 6/19 = 43. Also, this is now the selected layout.
    {
        let layout = ts.layout.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "EDO-12");
        assert_eq!(layout.scale.base_pitch.to_string(), "220*^43|76");
    }
    tc.shutdown().await
}

#[tokio::test]
async fn test_shift_layout() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-31
    tc.press_and_release_key(103).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    {
        let layout = ts.layout.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "EDO-31");
        assert_eq!(layout.base, Some((2, 2)));
    }

    // Enter shift mode
    tc.press_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Touch a note, then another to shift
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.shift_layout_state,
        ShiftLayoutState::FirstSelected { .. }
    ));
    // Over 1 column, up 2 rows
    tc.press_and_release_key(55).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    {
        let layout = ts.layout.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "EDO-31");
        assert_eq!(layout.base, Some((3, 4)));
    }
    // Release shift
    tc.release_key(keys::SHIFT).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_non_note_to_note() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Press any key on the start screen.
    tc.press_key(55).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    // Select a layout
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Still not seen as down.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    // Release the key
    tc.release_key(55).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_note_to_non_note() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Press a note key
    tc.press_key(18).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&18));
    let layout_101_pos_18 = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &1);
    // Select a different layout that has the note
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the pitch has changed.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&18));
    let layout_102_pos_18 = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_ne!(layout_101_pos_18, layout_102_pos_18);
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_102_pos_18).unwrap_or(&0), &1);
    // Select a layout that doesn't have the note
    tc.press_and_release_key(105).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_102_pos_18).unwrap_or(&0), &0);
    tc.release_key(18).await?;

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_with_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(101).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Enter sustain
    tc.press_and_release_key(keys::SUSTAIN).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Press a note key
    tc.press_key(18).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&18));
    let pos_18a = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    // Select a different layout that has the note by shifting down an octave
    tc.press_and_release_key(keys::DOWN_ARROW).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the new and old pitches are both playing.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&18));
    let pos_18b = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_ne!(pos_18a, pos_18b);
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);
    // Release the key. The notes are still both on.
    tc.release_key(18).await?;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);

    tc.shutdown().await
}
