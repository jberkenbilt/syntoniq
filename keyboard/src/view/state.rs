//! The state module is responsible for keeping track of the live values for the app. It also
//! manages the broadcast channel used for SSE events so it can own the process of updating the
//! clients when state changes. This part is device-independent.
use crate::events;
use crate::events::{LayoutNamesEvent, RawLightEvent, SelectLayoutEvent, StateView};
use askama::Template;
use axum::response::sse::Event;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::broadcast::WeakSender;

#[derive(Template, Default, Clone, PartialEq)]
#[template(path = "cell-text.html")]
pub struct CellText {
    populated: bool,
    label1: String,
    label2: String,
}
#[derive(Template, Clone, PartialEq)]
#[template(path = "cell.html")]
pub struct Cell {
    key: u8,
    color: String,
    cell_text: CellText,
}

pub struct AppState {
    cells: HashMap<u8, Cell>,
    state_view: StateView,
    sse_tx: Option<broadcast::Sender<Event>>,
    events_tx: events::WeakSender,
}
pub type LockedState = Arc<RwLock<AppState>>;

impl Cell {
    pub(crate) fn new(key: u8, color: String, label1: &str, label2: &str) -> Self {
        let populated = !label1.is_empty() || !label2.is_empty();
        Self {
            key,
            color,
            cell_text: CellText {
                populated,
                label1: label1.to_string(),
                label2: label2.to_string(),
            },
        }
    }

    pub fn empty(key: u8) -> Self {
        Cell {
            key,
            color: "var(--control-background)".to_string(),
            cell_text: Default::default(),
        }
    }

    pub fn event_name(&self) -> String {
        format!("sse-cell-{}", self.key)
    }
}

impl AppState {
    pub fn new_locked(events_tx: events::WeakSender) -> LockedState {
        let (sse_tx, _) = broadcast::channel(1000);
        let tx = sse_tx.clone().downgrade();
        tokio::spawn(async move {
            Self::sse_keepalive(tx).await;
        });
        let app = Self {
            cells: Default::default(),
            state_view: Default::default(),
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

    pub fn get_state_view(&self) -> &StateView {
        &self.state_view
    }

    pub fn set_cell(&mut self, key: u8, color: String, top: &str, bottom: &str) {
        let cell = Cell::new(key, color, top, bottom);
        let old = self.cells.insert(key, cell.clone());
        let Some(tx) = self.sse_tx.clone() else {
            return;
        };
        if !matches!(old, Some(x) if x == cell) {
            let event = Event::default()
                .event(cell.event_name())
                .data(cell.render().unwrap());
            let _ = tx.send(event);
        }
    }

    pub fn handle_light_event(&mut self, events: &[RawLightEvent]) {
        for e in events {
            self.set_cell(e.key, e.rgb_color.clone(), &e.label1, &e.label2);
        }
    }

    pub fn clear_lights(&mut self) {
        let positions: Vec<_> = self.cells.keys().cloned().collect();
        for p in positions {
            self.set_cell(p, events::OFF_RGB.to_string(), "", "");
        }
    }

    async fn send_state_view(&mut self) {
        let Some(tx) = self.sse_tx.clone() else {
            return;
        };
        let event = Event::default()
            .event("side-info")
            .data(self.state_view.render().unwrap());
        let _ = tx.send(event);
    }

    pub async fn handle_select_layout(&mut self, e: SelectLayoutEvent) {
        self.state_view.selected_layout = e.layout.name.to_string();
        self.send_state_view().await;
    }

    pub async fn handle_layout_names(&mut self, e: LayoutNamesEvent) {
        self.state_view.layout_names = e.names.clone();
        self.send_state_view().await;
    }

    pub async fn handle_reset(&mut self) {
        self.state_view = Default::default();
        self.send_state_view().await;
    }
}
