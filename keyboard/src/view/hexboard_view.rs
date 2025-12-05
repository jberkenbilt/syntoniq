//! This module contains the HexBoard-specific parts of the web view.

use crate::events::StateView;
use crate::hexboard::CommandKey;
use crate::view::state;
use crate::view::state::{Cell, LockedState, SkipSse};
use askama::Template;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "hexboard.html")]
pub struct HexBoardView<'a> {
    state_view: &'a StateView,
    kbd: HexBoardKeyboard<'a>,
}

#[derive(Template)]
#[template(path = "hexboard-keyboard.html")]
pub struct HexBoardKeyboard<'a> {
    cells: &'a HashMap<u8, Cell>,
}

impl<'a> HexBoardView<'a> {
    pub async fn generate_view(state: LockedState) -> String {
        let s = state.read().await;
        HexBoardView::new(s.get_cells(), s.get_state_view())
            .render()
            .unwrap()
    }

    pub async fn generate_board(state: LockedState) -> String {
        let s = state.read().await;
        HexBoardView::new(s.get_cells(), s.get_state_view())
            .kbd
            .render_with_values(&SkipSse)
            .unwrap()
    }

    pub fn new(cells: &'a HashMap<u8, Cell>, state_view: &'a StateView) -> Self {
        Self {
            kbd: HexBoardKeyboard { cells },
            state_view,
        }
    }

    pub fn command_key(i: &u8) -> String {
        // Used in the template expansion
        let cmd = CommandKey::try_from(*i).unwrap();
        match cmd {
            CommandKey::Reset => "Reset",
            CommandKey::Layout => "Select Layout",
            CommandKey::Sustain => "Toggle Sustain",
            CommandKey::OctaveUp => "Octave Up",
            CommandKey::OctaveDown => "Octave Down",
            CommandKey::Shift => "Shift Layout",
            CommandKey::Transpose => "Transpose",
        }
        .to_string()
    }
}

impl<'a> HexBoardKeyboard<'a> {
    fn get_cell(&self, grid_row: &u8, grid_col: &u8, skip_sse: bool) -> String {
        let key_col = grid_col + 1 - grid_row % 2;
        let key = key_col + grid_row * 10;
        let empty = Cell::empty(key);
        let t = self.cells.get(&key).unwrap_or(&empty);
        state::maybe_strip_sse(t.render().unwrap(), skip_sse)
    }
}
