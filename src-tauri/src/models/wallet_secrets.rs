use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletSecrets {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
