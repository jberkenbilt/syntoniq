use crate::scale::Scale;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Layout {
    pub name: String,
    /// lower-left row, lower-left column, upper-right row, upper-right column
    /// Row 1 is at the bottom. Column 1 is at the left.
    pub bbox: (i8, i8, i8, i8),
    /// row, column of base pitch
    pub base: (i8, i8),
    pub scale_name: String,
    /// horizontal, vertical steps
    pub steps: (i8, i8),
    #[serde(skip)]
    pub scale: Option<Arc<Scale>>,
}
