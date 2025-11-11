use crate::engine::SoundType;
use crate::events::{EngineState, Event, Events, KeyData, KeyEvent, StateView, TestEvent};
use crate::view::web;
use crate::{engine, events};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub(crate) struct ChannelPair<T> {
    pub tx: mpsc::Sender<T>,
    pub rx: mpsc::Receiver<T>,
}
impl<T> Default for ChannelPair<T> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self { tx, rx }
    }
}

pub struct TestController {
    events_tx: events::UpgradedSender,
    events_rx: events::Receiver,
    engine_channel: ChannelPair<EngineState>,
    web_channel: ChannelPair<StateView>,
    engine_handle: JoinHandle<anyhow::Result<()>>,
    web_handle: JoinHandle<()>,
}
impl TestController {
    pub async fn new() -> Self {
        let _ = env_logger::try_init();
        let events = Events::new();
        let events_tx_weak = events.sender().await;
        let events_tx = events_tx_weak.upgrade().unwrap();
        let events_rx = events.receiver();
        let tx2 = events_tx_weak.clone();
        let rx2 = events_rx.resubscribe();
        let engine_handle = tokio::spawn(async move {
            engine::run("testdata/conf.toml".into(), SoundType::None, tx2, rx2).await
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
            engine_channel: Default::default(),
            web_channel: Default::default(),
            engine_handle,
            web_handle,
        };
        tc.wait_for_test_event(TestEvent::ResetComplete).await;
        tc
    }

    pub fn tx(&self) -> events::UpgradedSender {
        self.events_tx.clone()
    }

    pub fn rx(&self) -> events::Receiver {
        self.events_rx.resubscribe()
    }

    pub async fn shutdown(mut self) -> anyhow::Result<()> {
        self.events_tx.send(Event::Shutdown)?;
        while events::receive_check_lag(&mut self.events_rx, None)
            .await
            .is_some()
        {}
        self.web_handle.await?;
        self.engine_handle.await?
    }

    pub async fn get_engine_state(&mut self) -> EngineState {
        self.events_tx
            .send(Event::TestEngine(self.engine_channel.tx.clone()))
            .unwrap();
        self.engine_channel.rx.recv().await.unwrap()
    }

    pub async fn get_web_state(&mut self) -> StateView {
        self.events_tx
            .send(Event::TestWeb(self.web_channel.tx.clone()))
            .unwrap();
        self.web_channel.rx.recv().await.unwrap()
    }

    pub async fn wait_for_event<F>(&mut self, f: F) -> Option<Event>
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

    pub async fn wait_for_test_event(&mut self, test_event: TestEvent) {
        self.wait_for_event(|e| matches!(e, Event::TestEvent(t) if *t == test_event))
            .await;
    }

    pub async fn send_key(&mut self, key: KeyData, on: bool) -> anyhow::Result<()> {
        self.events_tx.send(Event::KeyEvent(KeyEvent {
            key,
            velocity: if on { 127 } else { 0 },
        }))?;
        assert!(
            self.wait_for_event(|e| matches!(e, Event::KeyEvent(_)))
                .await
                .is_some()
        );
        Ok(())
    }

    pub async fn press_key(&mut self, key: KeyData) -> anyhow::Result<()> {
        self.send_key(key, true).await
    }

    pub async fn release_key(&mut self, key: KeyData) -> anyhow::Result<()> {
        self.send_key(key, false).await
    }

    pub async fn press_and_release_key(&mut self, key: KeyData) -> anyhow::Result<()> {
        self.press_key(key).await?;
        self.release_key(key).await
    }

    pub async fn sync(&mut self) -> anyhow::Result<()> {
        self.events_tx.send(Event::TestSync)?;
        self.wait_for_test_event(TestEvent::Sync).await;
        Ok(())
    }
}
