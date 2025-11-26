use crate::DeviceType;
use crate::controller::Device;
use crate::engine::{Keyboard, SoundType};
use crate::events::{
    ButtonData, Color, EngineState, Event, Events, FromDevice, KeyData, KeyEvent, Note,
    RawLightEvent, StateView, TestEvent,
};
use crate::view::web;
use crate::{engine, events};
use std::sync::{Arc, LazyLock};
use syntoniq_common::parsing::{Coordinate, Layout};
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

struct TestKeyboard;
impl Keyboard for TestKeyboard {
    fn reset(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn layout_supported(&self, _layout: &Layout) -> bool {
        true
    }

    fn note_positions(&self, _keyboard: &str) -> &'static [Coordinate] {
        // This is gratuitously different from the launchpad implementation to avoid complaints
        // from the IDE about duplicated code.
        static COORDS: LazyLock<Vec<Coordinate>> = LazyLock::new(|| {
            let mut vec = Vec::with_capacity(64);
            for row in 1..9 {
                for col in 1..9 {
                    vec.push(Coordinate { row, col });
                }
            }
            vec
        });
        &COORDS
    }

    fn note_light_event(
        &self,
        _note: Option<&Note>,
        _position: Coordinate,
        _velocity: u8,
    ) -> RawLightEvent {
        RawLightEvent {
            button: ButtonData::Note {
                position: Coordinate { row: 0, col: 0 },
            },
            color: Color::Off,
            rgb_color: events::OFF_RGB.to_string(),
            label1: String::new(),
            label2: String::new(),
        }
    }

    fn make_device(&self) -> Arc<dyn Device> {
        // We never pass a controller to the test keyboard.
        unreachable!();
    }

    fn handle_raw_event(&self, _msg: FromDevice) -> anyhow::Result<()> {
        Ok(())
    }

    fn main_event_loop(&self, _event: Event) -> anyhow::Result<()> {
        Ok(())
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
        let lp = TestKeyboard;
        let keyboard = Arc::new(lp);
        let k2 = keyboard.clone();
        let engine_handle = tokio::spawn(async move {
            engine::run("test-data/keyboard.stq", SoundType::None, k2, tx2, rx2).await
        });
        let tx2 = events_tx_weak.clone();
        let rx2 = events_rx.resubscribe();
        let web_handle = tokio::spawn(async move {
            web::http_view(tx2, rx2, 0, DeviceType::Empty).await;
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

    pub async fn shift(&mut self) -> anyhow::Result<EngineState> {
        self.press_and_release_key(KeyData::Shift).await?;
        self.wait_for_test_event(TestEvent::HandledKey).await;
        Ok(self.get_engine_state().await)
    }

    pub async fn transpose(&mut self) -> anyhow::Result<EngineState> {
        self.press_and_release_key(KeyData::Transpose).await?;
        self.wait_for_test_event(TestEvent::HandledKey).await;
        Ok(self.get_engine_state().await)
    }

    pub async fn sync(&mut self) -> anyhow::Result<()> {
        self.events_tx.send(Event::TestSync)?;
        self.wait_for_test_event(TestEvent::Sync).await;
        Ok(())
    }
}
