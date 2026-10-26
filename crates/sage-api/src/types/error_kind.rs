use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Wallet,
    Api,
    NotFound,
    Unauthorized,
    Internal,
    DatabaseMigration,
    Nfc,
    /// One or more recipients couldn't be used. See the attached recipient errors.
    Recipient,
    AlreadyRunning,
}
