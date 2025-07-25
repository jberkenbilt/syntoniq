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
        env_logger::init();
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
        Self {
            events_tx,
            events_rx,
            engine_channel,
            web_channel,
            engine_handle,
            web_handle,
        }
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
