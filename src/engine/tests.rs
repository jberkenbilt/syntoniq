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
        let engine_handle =
            tokio::spawn(
                async move { run("qlaunchpad.toml".into(), SoundType::None, tx2, rx2).await },
            );
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

    async fn wait_for_event<F>(&mut self, f: F) -> bool
    where
        F: Fn(&Event) -> bool,
    {
        while let Some(event) = events::receive_check_lag(&mut self.events_rx, None).await {
            if f(&event) {
                return true;
            }
        }
        false
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
        assert!(self.wait_for_event(|e| matches!(e, Event::Key(_))).await);
        Ok(())
    }

    async fn press_and_release_key(&mut self, position: u8) -> anyhow::Result<()> {
        self.send_key(position, true).await?;
        self.send_key(position, false).await
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
    assert_eq!(ws.base_pitch, "220*1\\4");
    tc.shutdown().await
}
