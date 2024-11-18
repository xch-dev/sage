use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Wallet,
    Api,
    NotFound,
    Unauthorized,
    Internal,
}
