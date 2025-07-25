use super::*;
use crate::events::Events;
use std::sync::LazyLock;

pub struct EngineChannel {
    pub tx: RwLock<tokio::sync::mpsc::Sender<EngineState>>,
    pub rx: RwLock<tokio::sync::mpsc::Receiver<EngineState>>,
}

pub static ENGINE_CHANNEL: LazyLock<Arc<EngineChannel>> = LazyLock::new(|| {
    let (tx, rx) = tokio::sync::mpsc::channel(1000);
    Arc::new(EngineChannel {
        tx: RwLock::new(tx),
        rx: RwLock::new(rx),
    })
});

async fn get_engine_state(tx: &events::UpgradedSender) -> EngineState {
    tx.send(events::Event::TestEngine).unwrap();
    ENGINE_CHANNEL.rx.write().await.recv().await.unwrap()
}

#[tokio::test]
async fn test_todo() -> anyhow::Result<()> {
    env_logger::init();
    let events = Events::new();
    let events_tx = events.sender().await;
    let events_rx = events.receiver();
    let tx2 = events_tx.clone();
    let rx2 = events_rx.resubscribe();
    let h =
        tokio::spawn(async move { run("qlaunchpad.toml".into(), SoundType::None, tx2, rx2).await });
    let tx = events_tx.upgrade().unwrap();
    let ts = get_engine_state(&tx).await;
    assert!(ts.layout.is_none());
    tx.send(Event::Shutdown)?;
    h.await?
}
