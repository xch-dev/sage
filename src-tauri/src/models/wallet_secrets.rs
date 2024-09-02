use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSecrets {
    pub mnemonic: Option<String>,
    #[serde_as(as = "Hex")]
    pub secret_key: [u8; 32],
}
