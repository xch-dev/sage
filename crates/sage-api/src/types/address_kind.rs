use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AddressKind {
    Own,
    Burn,
    Launcher,
    Offer,
    External,
    Unknown,
}
