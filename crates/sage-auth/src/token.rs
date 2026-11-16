use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

const TOKEN_PREFIX: &str = "sage_agent_";
const TOKEN_BODY_LEN: usize = 64;

/// Plaintext agent bearer token; deliberately has no `Debug` or `Display`.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
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
