use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SageAppStorage {
    AppleDataStore { identifier_hex: String },
    WindowsProfile { directory_name: String },
    Unmanaged,
}
