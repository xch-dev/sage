use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default = "default_wallet_name")]
    pub name: String,

    #[serde(default = "default_derive_automatically")]
    pub derive_automatically: bool,

    #[serde(default = "default_derivation_batch_size")]
    pub derivation_batch_size: u32,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: default_wallet_name(),
            derive_automatically: default_derive_automatically(),
            derivation_batch_size: default_derivation_batch_size(),
        }
    }
}

fn default_wallet_name() -> String {
    "Unnamed Wallet".to_string()
}

fn default_derive_automatically() -> bool {
    true
}

fn default_derivation_batch_size() -> u32 {
    500
}
