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
    assert!(*ts.notes_on.get(&Pitch::must_parse("220*^3|12")).unwrap() > 0);
    tc.release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        *ts.notes_on.get(&Pitch::must_parse("220*^3|12")).unwrap(),
        0
    );

    // Enter sustain mode
    tc.press_and_release_key(keys::SUSTAIN).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.sustain);

    // Press and release middle C. Note stays on.
    tc.press_and_release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(*ts.notes_on.get(&Pitch::must_parse("220*^3|12")).unwrap() > 0);

    // Press and release middle C. Note turns off.
    tc.press_and_release_key(32).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        *ts.notes_on.get(&Pitch::must_parse("220*^3|12")).unwrap(),
        0
    );

    // Leave sustain mode
    tc.press_and_release_key(keys::SUSTAIN).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.sustain);

    tc.shutdown().await
}

#[tokio::test]
async fn test_move_cancels() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));

    // Touch move to end move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));

    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));
    // Touch a note
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::FirstSelected { .. }));
    // Move mode cancels
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));

    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));
    // Change layout
    tc.press_and_release_key(103).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Touch two different notes -- this cancels because we can't do a shift
    // after changing layouts.
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    tc.press_and_release_key(33).await?;
    tc.wait_for_test_event(TestEvent::MoveCanceled).await;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));

    // Select generic layout
    tc.press_and_release_key(105).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.layout.unwrap().read().await.scale.scale_type,
        ScaleType::Generic(_)
    ));
    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));
    // Can't move a generic layout
    tc.press_and_release_key(32).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    tc.press_and_release_key(33).await?;
    tc.wait_for_test_event(TestEvent::MoveCanceled).await;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));

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
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));

    // Touch a note twice to transpose
    tc.press_and_release_key(44).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    tc.press_and_release_key(44).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));
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

    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));

    // Select EDO-19
    tc.press_and_release_key(102).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let edo_19 = ts.layout.clone().unwrap();
    assert_eq!(ts.layout.unwrap().read().await.scale.name, "EDO-19");

    // Touch a note twice to transpose
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));
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
async fn test_move() -> anyhow::Result<()> {
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

    // Enter move mode
    tc.press_and_release_key(keys::MOVE).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Pending { .. }));

    // Touch a note, then another to shift
    tc.press_and_release_key(34).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    // Over 1 column, up 2 rows
    tc.press_and_release_key(55).await?;
    tc.wait_for_test_event(TestEvent::EngineStateChange).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.move_state, MoveState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    {
        let layout = ts.layout.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "EDO-31");
        assert_eq!(layout.base, Some((3, 4)));
    }

    tc.shutdown().await
}
