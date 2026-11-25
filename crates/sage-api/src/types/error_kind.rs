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
    /// The database shared by the wallets on a network, which holds the address book and handle
    /// mappings, couldn't be migrated. Deleting the wallet's own database doesn't fix this.
    SharedDatabaseMigration,
    Nfc,
    /// One or more recipients couldn't be used. See the attached recipient errors.
    Recipient,
    AlreadyRunning,
}
