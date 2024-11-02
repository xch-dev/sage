use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetNftCollections {
    pub offset: u32,
    pub limit: u32,
    pub include_hidden: bool,
}
