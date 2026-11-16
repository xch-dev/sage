use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::Capability;

/// Which wallets a grantee may act on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WalletScope {
    AllWallets,
    #[serde(rename_all = "camelCase")]
    SelectedWallets {
        fingerprints: Vec<u32>,
    },
}

impl WalletScope {
    /// Whether the wallet with this fingerprint is in scope.
    pub fn allows(&self, fingerprint: u32) -> bool {
        match self {
            Self::AllWallets => true,
            Self::SelectedWallets { fingerprints } => fingerprints.contains(&fingerprint),
        }
    }
}

impl Default for WalletScope {
    fn default() -> Self {
        Self::SelectedWallets {
            fingerprints: Vec::new(),
        }
    }
}

/// What a grantee is allowed to do and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub capabilities: BTreeSet<Capability>,
    pub wallet_scope: WalletScope,
}

/// Who is making a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    Owner,
    Grantee(Grant),
}

/// A registered agent and its grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub capabilities: BTreeSet<Capability>,
    pub wallet_scope: WalletScope,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_used_at_ms: Option<i64>,
}

impl Agent {
    /// The identity this agent acts under.
    pub fn identity(&self) -> Identity {
        Identity::Grantee(Grant {
            capabilities: self.capabilities.clone(),
            wallet_scope: self.wallet_scope.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn wallet_scope_serializes_in_the_sage_apps_shape() {
        assert_eq!(
            serde_json::to_value(WalletScope::AllWallets).unwrap(),
            json!({ "kind": "allWallets" })
        );
        assert_eq!(
            serde_json::to_value(WalletScope::SelectedWallets {
                fingerprints: vec![1, 2]
            })
            .unwrap(),
            json!({ "kind": "selectedWallets", "fingerprints": [1, 2] })
        );
    }

    #[test]
    fn agent_serializes_with_camel_case_keys() {
        let agent = Agent {
            id: "a".to_string(),
            name: "n".to_string(),
            capabilities: [Capability::WalletGetCoins].into(),
            wallet_scope: WalletScope::AllWallets,
            created_at_ms: 1,
            updated_at_ms: 2,
            last_used_at_ms: Some(3),
        };
        assert_eq!(
            serde_json::to_value(&agent).unwrap(),
            json!({
                "id": "a",
                "name": "n",
                "capabilities": ["wallet.get_coins"],
                "walletScope": { "kind": "allWallets" },
                "createdAtMs": 1,
                "updatedAtMs": 2,
                "lastUsedAtMs": 3,
            })
        );
    }
}
