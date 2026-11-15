use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SageAppWalletScope {
    AllWallets,
    SelectedWallets { fingerprints: Vec<u32> },
}

impl Default for SageAppWalletScope {
    fn default() -> Self {
        Self::SelectedWallets {
            fingerprints: vec![],
        }
    }
}

impl SageAppWalletScope {
    pub fn allows(&self, fingerprint: u32) -> bool {
        match self {
            Self::AllWallets => true,
            Self::SelectedWallets { fingerprints } => fingerprints.contains(&fingerprint),
        }
    }
}

/// The shared scope keeps its original serialized representation and SDK type name.
pub type WalletScope = SageAppWalletScope;
