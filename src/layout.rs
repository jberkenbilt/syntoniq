use crate::scale::Scale;
use serde::Deserialize;

pub const NOTE_ROWS: i8 = 8;
pub const NOTE_COLS: i8 = 8;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct RowCol {
    pub row: i8,
    pub col: i8,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct HorizVert {
    pub h: i8,
    pub v: i8,
}

// Don't derive Clone for Layout as we allow layouts to be mutated for transposition and shift.
#[derive(Debug)]
pub struct Layout {
    pub name: String,
    pub scale: Scale,
    pub base: Option<RowCol>,
    pub steps: Option<HorizVert>,
}
