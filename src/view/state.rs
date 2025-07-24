//! The state module is responsible for keeping track of the live values for the app. It also
//! manages the broadcast channel used for SSE events so it can own the process of updating the
//! clients when state changes.
use crate::events;
use crate::events::{LightEvent, SelectLayoutEvent};
use crate::view::content::{Cell, SideInfo};
use askama::Template;
use axum::response::sse::Event;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::broadcast::WeakSender;

pub const ROWS: u8 = 11;
pub const COLS: u8 = 10;

pub struct AppState {
    cells: HashMap<u8, Cell>,
    side_info: SideInfo,
    sse_tx: Option<broadcast::Sender<Event>>,
    events_tx: events::WeakSender,
}
pub type LockedState = Arc<RwLock<AppState>>;

impl AppState {
    pub fn new_locked(events_tx: events::WeakSender) -> LockedState {
        let (sse_tx, _) = broadcast::channel(1000);
        let tx = sse_tx.clone().downgrade();
        tokio::spawn(async move {
            Self::sse_keepalive(tx).await;
        });
        let app = Self {
            cells: Default::default(),
            side_info: Default::default(),
            sse_tx: Some(sse_tx),
            events_tx,
        };
        Arc::new(RwLock::new(app))
    }

    pub fn get_sse_tx(&self) -> Option<broadcast::Sender<Event>> {
        self.sse_tx.clone()
    }

    pub fn get_events_tx(&self) -> Option<events::UpgradedSender> {
        self.events_tx.upgrade()
    }

    pub fn shutdown(&mut self) {
        self.sse_tx.take();
    }

    async fn sse_keepalive(tx: WeakSender<Event>) {
        // Pattern: upgrade the sender to use it, then sleep without holding the upgrade.
        // This keeps us from interfering with shutdown.
        loop {
            match tx.upgrade() {
                Some(upgraded) => {
                    // Broadcast channels drop older, unread values when full. This is
                    // appropriate behavior for SSE events when no one is listening.
                    let _ = upgraded.send(Event::default().event("heartbeat").data("keep-alive"));
                }
                None => return,
            };
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    }

    pub fn get_cells(&self) -> &HashMap<u8, Cell> {
        &self.cells
    }

    pub fn get_side_info(&self) -> &SideInfo {
        &self.side_info
    }

    pub fn set_cell(&mut self, position: u8, color: &str, top: &str, bottom: &str) {
        let cell = Cell::new(position, color, top, bottom);
        let old = self.cells.insert(position, cell.clone());
        let Some(tx) = self.sse_tx.clone() else {
            return;
        };
        if let Some(old) = old
            && old != cell
        {
            let event = Event::default()
                .event(cell.event_name())
                .data(cell.render().unwrap());
            let _ = tx.send(event);
        }
    }

    pub fn handle_light_event(&mut self, e: LightEvent) {
        self.set_cell(e.position, e.color.rgb_color(), &e.label1, &e.label2);
    }

    async fn send_side_info(&mut self) {
        let Some(tx) = self.sse_tx.clone() else {
            return;
        };
        let event = Event::default()
            .event("side-info")
            .data(self.side_info.render().unwrap());
        let _ = tx.send(event);
    }

    pub async fn handle_select_layout(&mut self, e: SelectLayoutEvent) {
        {
            let layout = e.layout.read().await;
            self.side_info.base_pitch = layout.scale.base_pitch.to_string();
            self.side_info.selected_layout = layout.name.clone();
        }
        self.send_side_info().await;
    }

    pub async fn handle_reset(&mut self) {
        self.side_info = Default::default();
        self.send_side_info().await;
    }
}
