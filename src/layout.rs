use crate::scale::Scale;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct LayoutConfig {
    pub name: String,
    /// lower-left row, lower-left column, upper-right row, upper-right column
    /// Row 1 is at the bottom. Column 1 is at the left.
    pub bbox: (i8, i8, i8, i8),
    /// row, column of base pitch
    pub base: (i8, i8),
    pub scale_name: String,
    /// horizontal, vertical steps
    pub steps: (i8, i8),
}

// Don't derive Clone for Layout as we allow layouts to be mutated for transposition and shift.
#[derive(Debug)]
pub struct Layout {
    pub name: String,
    pub bbox: (i8, i8, i8, i8),
    pub base: (i8, i8),
    pub scale: Scale,
    pub steps: (i8, i8),
}
