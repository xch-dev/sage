use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SageAppWalletScope {
    AllWallets,
    SelectedWallets { fingerprints: Vec<u32> },
}

impl SageAppWalletScope {
    pub fn allows(&self, fingerprint: u32) -> bool {
        match self {
            Self::AllWallets => true,
            Self::SelectedWallets { fingerprints } => fingerprints.contains(&fingerprint),
        }
    }
}

impl Default for SageAppWalletScope {
    fn default() -> Self {
        Self::SelectedWallets {
            fingerprints: vec![],
        }
    }
}
