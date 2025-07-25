//! The content module is responsible for the objects that comprise content and their HTML
//! rendering.
use crate::events::StateView;
use crate::view::state;
use askama::Template;
use std::collections::HashMap;

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
    position: u8,
    color: String,
    cell_text: CellText,
}
impl Cell {
    pub(crate) fn new(position: u8, color: &str, label1: &str, label2: &str) -> Self {
        let row = position / 10;
        let col = position % 10;
        let in_note_grid = (1..=8).contains(&row) && (1..=8).contains(&col);
        let mut color = color.to_string();
        if color.is_empty() {
            if in_note_grid {
                color = "var(--off-background)".to_string();
            } else {
                color = "var(--control-background)".to_string();
            }
        }
        let populated = !label1.is_empty() || !label2.is_empty();
        Self {
            position,
            color,
            cell_text: CellText {
                populated,
                label1: label1.to_string(),
                label2: label2.to_string(),
            },
        }
    }

    pub fn empty() -> Self {
        Cell {
            position: 127,
            color: "var(--control-background)".to_string(),
            cell_text: Default::default(),
        }
    }

    pub fn element_id(&self) -> String {
        format!("cell-{}", self.position)
    }

    pub fn event_name(&self) -> String {
        format!("sse-cell-{}", self.position)
    }
}

#[derive(Template)]
#[template(path = "app.html")]
pub struct App<'a> {
    rows: u8,
    cols: u8,
    cells: &'a HashMap<u8, Cell>,
    state_view: &'a StateView,
}
impl<'a> App<'a> {
    pub fn new(cells: &'a HashMap<u8, Cell>, state_view: &'a StateView) -> Self {
        Self {
            rows: state::ROWS,
            cols: state::COLS,
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
        let empty = Cell::empty();
        let t = self.cells.get(&position).unwrap_or(&empty);
        t.render().unwrap()
    }
}
