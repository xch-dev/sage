use std::collections::BTreeSet;

use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;

use crate::{SageAppWalletScope, UserBridgeCapability};

const TOKEN_PREFIX: &str = "sage_agent_";
const TOKEN_BODY_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAgent {
    pub id: String,
    pub name: String,
    pub capabilities: BTreeSet<UserBridgeCapability>,
    pub wallet_scope: SageAppWalletScope,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_used_at_ms: Option<i64>,
}

impl SageAgent {
    pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool {
        match &self.wallet_scope {
            SageAppWalletScope::AllWallets => true,
            SageAppWalletScope::SelectedWallets { fingerprints } => {
                fingerprints.contains(&fingerprint)
            }
        }
    }
}

/// Plaintext agent bearer token; deliberately has no `Debug` or `Display`.
#[derive(Clone)]
pub struct AgentToken(String);

impl AgentToken {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(format!("{TOKEN_PREFIX}{}", hex::encode(bytes)))
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let body = raw.strip_prefix(TOKEN_PREFIX)?;
        let valid = body.len() == TOKEN_BODY_LEN
            && body
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        valid.then(|| Self(raw.to_string()))
    }

    pub fn hash(&self) -> String {
        hex::encode(Sha256::digest(self.0.as_bytes()))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_round_trips_through_parse() {
        let t = AgentToken::generate();
        assert!(t.expose().starts_with("sage_agent_"));
        assert_eq!(t.expose().len(), 11 + 64);
        assert_eq!(AgentToken::parse(t.expose()).unwrap().hash(), t.hash());
    }

    #[test]
    fn parse_rejects_malformed() {
        let hex64 = "a".repeat(64);
        assert!(AgentToken::parse("").is_none());
        assert!(AgentToken::parse("sage_agent_").is_none());
        assert!(AgentToken::parse(&format!("sage_agent_{}", "A".repeat(64))).is_none());
        assert!(AgentToken::parse(&format!("sage_agent_{}", "x".repeat(64))).is_none());
        assert!(AgentToken::parse(&format!("sage_agent_{}", "a".repeat(63))).is_none());
        assert!(AgentToken::parse(&format!("other_{hex64}")).is_none());
        assert!(AgentToken::parse(&format!("sage_agent_{hex64}")).is_some());
    }

    #[test]
    fn two_tokens_differ() {
        assert_ne!(
            AgentToken::generate().expose(),
            AgentToken::generate().expose()
        );
    }
}
