use crate::scale::Scale;
use serde::Deserialize;

pub const NOTE_ROWS: i8 = 8;
pub const NOTE_COLS: i8 = 8;

#[derive(Deserialize, Debug, PartialEq)]
pub struct LayoutConfig {
    pub name: String,
    pub scale_name: String,
    /// row, column of base pitch (EDO only)
    pub base: Option<(i8, i8)>,
    /// horizontal, vertical steps (EDO only)
    pub steps: Option<(i8, i8)>,
}

// Don't derive Clone for Layout as we allow layouts to be mutated for transposition and shift.
#[derive(Debug)]
pub struct Layout {
    pub name: String,
    pub scale: Scale,
    pub base: Option<(i8, i8)>,
    pub steps: Option<(i8, i8)>,
}
