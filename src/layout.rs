use crate::scale::Scale;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Layout {
    pub name: String,
    /// row (1 is bottom), column (1 is left) of lower left
    pub ll: (u8, u8),
    /// row (8 is top), column (8 is right) of upper left
    pub ur: (u8, u8),
    /// row, column of base pitch
    pub base: (u8, u8),
    pub scale_name: String,
    /// horizontal, vertical steps
    pub steps: (u8, u8),
    #[serde(skip)]
    pub scale: Option<Arc<Scale>>,
}
