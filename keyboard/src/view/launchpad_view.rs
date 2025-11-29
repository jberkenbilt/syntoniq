//! This module contains the Launchpad-specific parts of the web view.
use crate::events::StateView;
use crate::view::state::{Cell, LockedState};
use askama::Template;
use std::collections::HashMap;

pub const ROWS: u8 = 11;
pub const COLS: u8 = 10;

#[derive(Template)]
#[template(path = "launchpad.html")]
pub struct LaunchpadView<'a> {
    rows: u8,
    cols: u8,
    cells: &'a HashMap<u8, Cell>,
    state_view: &'a StateView,
}
impl<'a> LaunchpadView<'a> {
    pub async fn generate_view(state: LockedState) -> String {
        let s = state.read().await;
        LaunchpadView::new(s.get_cells(), s.get_state_view())
            .render()
            .unwrap()
    }

    pub fn new(cells: &'a HashMap<u8, Cell>, state_view: &'a StateView) -> Self {
        Self {
            rows: ROWS,
            cols: COLS,
            cells,
            state_view,
        }
    }

    fn get_cell(&self, grid_row: &u8, grid_col: &u8) -> String {
        // Launchpad rows are, from bottom to top, are 0, 10, 1..=9. Grid rows are
        // 0 to 10 from top to bottom.
        let pad_row = match grid_row {
            9 => 10,
            10 => 0,
            row => 9 - row,
        };
        let pad_col = *grid_col;
        let position = 10 * pad_row + pad_col;
        let empty = Cell::empty(position);
        let t = self.cells.get(&position).unwrap_or(&empty);
        t.render().unwrap()
    }
}
