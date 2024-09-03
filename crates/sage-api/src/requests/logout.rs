use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Logout {
    pub fingerprint: u32,
}
