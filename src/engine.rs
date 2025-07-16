use crate::config::Config;
use crate::events::{
    AssignLayoutEvent, Color, Event, KeyEvent, LightEvent, LightMode, SelectLayoutEvent,
};
use crate::layout::Layout;
use crate::{controller, events};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

mod keys {
    pub const CLEAR: u8 = 60;
    pub const LAYOUT_MIN: u8 = 101;
    pub const LAYOUT_MAX: u8 = 108;
    pub const LAYOUT_SCROLL: u8 = 19;
}

struct Engine {
    config: Config,
    events_tx: events::Sender,
    layout: Option<Arc<Layout>>,
    /// control key position -> selected layout
    assigned_layouts: HashMap<u8, Arc<Layout>>,
}

impl Engine {
    async fn reset(&mut self) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        controller::clear_lights(&tx).await?;
        for (color, positions) in [
            (
                Color::Green,
                vec![63u8, 64, 65, 66, 52, 57, 42, 47, 32, 37, 23, 24, 25],
            ),
            (Color::Blue, vec![34, 35, 16, 17, 18]),
            (Color::Cyan, vec![26]),
            (Color::Purple, vec![72, 73, 74, 75, 76, 77]),
            (Color::Pink, vec![83]),
            (Color::Orange, vec![84]),
            (Color::Yellow, vec![85]),
            (Color::Red, vec![86]),
            (Color::White, vec![keys::CLEAR]),
        ] {
            for position in positions {
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position,
                    color,
                }))?;
            }
        }
        let mut position = keys::LAYOUT_MIN;
        self.assigned_layouts.clear();
        for layout in self.config.layouts.iter().cloned() {
            tx.send(Event::AssignLayout(AssignLayoutEvent { position, layout }))?;
            position += 1;
            if position > keys::LAYOUT_MAX {
                //TODO: deal with scrolling. Key 109 is the scroll key and will assign layouts
                // starting from an offset to the lower numbered keys.
                tx.send(Event::Light(LightEvent {
                    mode: LightMode::On,
                    position: keys::LAYOUT_SCROLL,
                    color: Color::White,
                }))?;
                break;
            }
        }
        Ok(())
    }

    async fn handle_key(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        let KeyEvent { key, velocity } = key_event;
        let off = velocity == 0;
        match key {
            keys::LAYOUT_MIN..=keys::LAYOUT_MAX if off => {
                if let Some(layout) = self.assigned_layouts.get(&key).cloned() {
                    tx.send(Event::SelectLayout(SelectLayoutEvent { layout }))?;
                }
            }
            keys::CLEAR if off => {
                tx.send(Event::Reset)?;
            }
            _ => (), // ignore
        }
        Ok(())
    }

    async fn select_layout(&mut self, event: SelectLayoutEvent) -> anyhow::Result<()> {
        self.layout = Some(event.layout);
        // TODO: draw the layout
        let layout = self.layout.as_ref().unwrap().as_ref();
        log::warn!("got layout {}", layout.name);
        Ok(())
    }

    async fn assign_layout(&mut self, layout_event: AssignLayoutEvent) -> anyhow::Result<()> {
        let Some(tx) = self.events_tx.upgrade() else {
            return Ok(());
        };
        // Activate the light for selecting the layout.
        let AssignLayoutEvent { position, layout } = layout_event;
        if !(keys::LAYOUT_MIN..=keys::LAYOUT_MAX).contains(&position) {
            return Ok(());
        }
        self.assigned_layouts.insert(position, layout);
        tx.send(Event::Light(LightEvent {
            mode: LightMode::On,
            position,
            color: Color::White,
        }))?;
        Ok(())
    }
}

pub async fn run(
    config_file: PathBuf,
    events_tx: events::Sender,
    mut rx: events::Receiver,
) -> anyhow::Result<()> {
    let config = Config::load(config_file)?;
    let mut engine = Engine {
        config,
        events_tx: events_tx.clone(),
        layout: None,
        assigned_layouts: Default::default(),
    };
    if let Some(tx) = events_tx.upgrade() {
        tx.send(Event::Reset)?;
    }
    while let Some(event) = events::receive_ignore_lag(&mut rx).await {
        // Note: this event loop calls event handlers inline. Sometimes those event handlers
        // generate other events, which are piling up in our queue while we are handling earlier
        // events. As long as the backlog on the event receiver is high enough and/or we don't
        // care about missing some messages, we should be fine. In practice, the messages we would
        // be most likely to miss our the ones we just sent, but we could also miss other people's
        // responses to those. We are not as likely to miss user events.
        match event {
            Event::Light(_) => {}
            Event::Key(e) => engine.handle_key(e).await?,
            Event::Pressure(_) => {}
            Event::Reset => engine.reset().await?,
            Event::AssignLayout(e) => engine.assign_layout(e).await?,
            Event::SelectLayout(e) => engine.select_layout(e).await?,
        }
    }
    Ok(())
}
