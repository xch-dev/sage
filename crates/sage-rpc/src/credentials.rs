use anyhow::{Context, Result, bail};
use rand::{RngCore, rngs::OsRng};
use sage_permissions::{Caller, CallerId, WalletCapability, WalletScope};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PAIRING_LIFETIME: u64 = 300;
const MAX_PENDING: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionKind {
    Service,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionGrant {
    pub name: String,
    pub kind: ConnectionKind,
    pub capabilities: BTreeSet<WalletCapability>,
    pub wallet_scope: WalletScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CredentialInfo {
    pub id: String,
    pub grant: ConnectionGrant,
    pub created_at: u64,
}

// Never derive Debug for a result carrying a plaintext token.
#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IssuedCredential {
    pub credential: CredentialInfo,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PairingInfo {
    pub id: String,
    pub code: String,
    pub grant: ConnectionGrant,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct CredentialCapability {
    pub key: WalletCapability,
    pub label: String,
    pub description: String,
    pub category: String,
    pub approval: bool,
    pub methods: Vec<String>,
}

pub fn credential_capabilities() -> Vec<CredentialCapability> {
    WalletCapability::ALL
        .iter()
        .copied()
        .filter(|cap| credential_capability_supported(*cap))
        .map(|key| {
            let definition = sage_permissions::wallet_capability_definition(key);
            CredentialCapability {
                key,
                label: definition.label.into(),
                description: definition.description.into(),
                category: sage_permissions::RPC_POLICIES
                    .iter()
                    .find(|p| p.capability == Some(key))
                    .unwrap()
                    .category
                    .into(),
                approval: sage_permissions::RPC_POLICIES
                    .iter()
                    .any(|p| p.capability == Some(key) && p.approval),
                methods: sage_permissions::RPC_POLICIES
                    .iter()
                    .filter(|p| p.capability == Some(key))
                    .map(|p| p.method.into())
                    .collect(),
            }
        })
        .collect()
}

pub fn credential_capability_supported(cap: WalletCapability) -> bool {
    sage_permissions::RPC_POLICIES
        .iter()
        .any(|policy| policy.capability == Some(cap))
}

impl ConnectionGrant {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.chars().count() > 80
            || self.name.chars().any(char::is_control)
        {
            bail!("Enter a name of 1–80 characters");
        }
        if self.capabilities.is_empty()
            || self
                .capabilities
                .iter()
                .any(|cap| !credential_capability_supported(*cap))
        {
            bail!("Select supported RPC permissions");
        }
        if let WalletScope::SelectedWallets { fingerprints } = &self.wallet_scope {
            if fingerprints.is_empty() || fingerprints.len() > 100 {
                bail!("Select at least one wallet (up to 100)");
            }
        }
        Ok(())
    }
    fn is_subset_of(&self, requested: &Self) -> bool {
        self.kind == requested.kind
            && self.capabilities.is_subset(&requested.capabilities)
            && match (&self.wallet_scope, &requested.wallet_scope) {
                (_, WalletScope::AllWallets) => true,
                (
                    WalletScope::SelectedWallets {
                        fingerprints: chosen,
                    },
                    WalletScope::SelectedWallets {
                        fingerprints: requested,
                    },
                ) => chosen.iter().all(|fp| requested.contains(fp)),
                _ => false,
            }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct StoredCredential {
    pub(crate) info: CredentialInfo,
    token_hash: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredCredentials {
    version: u32,
    enabled: bool,
    port: u16,
    pub(crate) credentials: Vec<StoredCredential>,
}

#[derive(Clone)]
struct Pending {
    info: PairingInfo,
    secret_hash: String,
    decision: Option<Option<ConnectionGrant>>,
}
pub(crate) struct StoreState {
    pub(crate) saved: StoredCredentials,
    pending: HashMap<String, Pending>,
    pub(crate) operations: HashMap<String, crate::operation_approvals::PendingOperation>,
    attempts: VecDeque<u64>,
}

#[derive(Clone)]
pub struct CredentialStore {
    path: PathBuf,
    pub(crate) state: Arc<Mutex<StoreState>>,
}
impl std::fmt::Debug for CredentialStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialStore").finish_non_exhaustive()
    }
}

pub fn credential_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub(crate) fn random_hex(bytes: usize) -> String {
    let mut value = vec![0u8; bytes];
    OsRng.fill_bytes(&mut value);
    hex::encode(value)
}
fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

impl CredentialStore {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        let path = root.join("rpc-credentials.json");
        let saved: StoredCredentials = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .context("Could not read RPC credentials; the file has not been replaced")?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => StoredCredentials {
                version: 1,
                enabled: false,
                port: 9258,
                credentials: vec![],
            },
            Err(err) => return Err(err.into()),
        };
        if saved.version != 1 {
            bail!("Unsupported RPC credential store version");
        }
        for record in &saved.credentials {
            record.info.grant.validate()?;
        }
        Ok(Self {
            path,
            state: Arc::new(Mutex::new(StoreState {
                saved,
                pending: HashMap::new(),
                operations: HashMap::new(),
                attempts: VecDeque::new(),
            })),
        })
    }
    fn save(&self, saved: &StoredCredentials) -> Result<()> {
        let mut file = tempfile::NamedTempFile::new_in(self.path.parent().unwrap())?;
        // NamedTempFile uses owner-only permissions on Unix. Never persist plaintext tokens.
        file.write_all(&serde_json::to_vec_pretty(saved)?)?;
        file.as_file().sync_all()?;
        file.persist(&self.path).map_err(|err| err.error)?;
        Ok(())
    }
    pub fn settings(&self) -> (bool, u16) {
        let state = self.state.lock().unwrap();
        (state.saved.enabled, state.saved.port)
    }
    pub fn set_settings(&self, enabled: bool, port: u16) -> Result<()> {
        if port < 1024 {
            bail!("Port must be between 1024 and 65535");
        }
        let mut state = self.state.lock().unwrap();
        let mut next = state.saved.clone();
        next.enabled = enabled;
        next.port = port;
        self.save(&next)?;
        state.saved = next;
        Ok(())
    }
    pub fn list(&self) -> Vec<CredentialInfo> {
        self.state
            .lock()
            .unwrap()
            .saved
            .credentials
            .iter()
            .map(|record| record.info.clone())
            .collect()
    }
    fn issue_locked(
        &self,
        state: &mut StoreState,
        mut grant: ConnectionGrant,
    ) -> Result<IssuedCredential> {
        grant.validate()?;
        grant.name = grant.name.trim().into();
        let token = format!("sage_rpc_v1_{}", random_hex(32));
        let info = CredentialInfo {
            id: random_hex(16),
            grant,
            created_at: credential_now(),
        };
        let mut next = state.saved.clone();
        if next.credentials.len() >= 256 {
            bail!("Revoke an unused credential before creating another");
        }
        next.credentials.push(StoredCredential {
            info: info.clone(),
            token_hash: hash(&token),
        });
        self.save(&next)?;
        state.saved = next;
        Ok(IssuedCredential {
            credential: info,
            token,
        })
    }
    pub fn issue(&self, grant: ConnectionGrant) -> Result<IssuedCredential> {
        self.issue_locked(&mut self.state.lock().unwrap(), grant)
    }
    pub fn revoke(&self, id: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let mut next = state.saved.clone();
        next.credentials.retain(|record| record.info.id != id);
        self.save(&next)?;
        state.saved = next;
        state
            .operations
            .retain(|_, pending| pending.review.credential_id != id);
        Ok(())
    }
    pub fn rotate(&self, id: &str) -> Result<IssuedCredential> {
        let mut state = self.state.lock().unwrap();
        let mut next = state.saved.clone();
        let record = next
            .credentials
            .iter_mut()
            .find(|record| record.info.id == id)
            .context("Connection no longer exists")?;
        let token = format!("sage_rpc_v1_{}", random_hex(32));
        record.token_hash = hash(&token);
        let credential = record.info.clone();
        self.save(&next)?;
        state.saved = next;
        state
            .operations
            .retain(|_, pending| pending.review.credential_id != id);
        Ok(IssuedCredential { credential, token })
    }
    pub fn caller(&self, token: &str) -> Result<Caller> {
        if token.len() != 76 || !token.starts_with("sage_rpc_v1_") {
            bail!("Invalid credential");
        }
        let token_hash = hash(token);
        let state = self.state.lock().unwrap();
        let record = state
            .saved
            .credentials
            .iter()
            .find(|record| record.token_hash == token_hash)
            .context("Invalid or revoked credential")?;
        let grant = &record.info.grant;
        Ok(Caller {
            id: match grant.kind {
                ConnectionKind::Service => CallerId::Service(record.info.id.clone()),
                ConnectionKind::Agent => CallerId::Agent(record.info.id.clone()),
            },
            capabilities: grant.capabilities.clone(),
            wallet_scope: grant.wallet_scope.clone(),
        })
    }
    pub fn request_pairing(
        &self,
        grant: ConnectionGrant,
        secret_hash: String,
        now: u64,
    ) -> Result<PairingInfo> {
        grant.validate()?;
        if secret_hash.len() != 64 || !secret_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("secretHash must be a SHA-256 hex digest of a random client secret");
        }
        let mut state = self.state.lock().unwrap();
        state.pending.retain(|_, p| p.info.expires_at > now);
        while state
            .attempts
            .front()
            .is_some_and(|time| time.saturating_add(60) <= now)
        {
            state.attempts.pop_front();
        }
        if state.attempts.len() >= 5 || state.pending.len() >= MAX_PENDING {
            bail!("Too many pairing requests; wait a minute");
        }
        state.attempts.push_back(now);
        let info = PairingInfo {
            id: random_hex(16),
            code: random_hex(4).to_uppercase(),
            grant,
            expires_at: now + PAIRING_LIFETIME,
        };
        state.pending.insert(
            info.id.clone(),
            Pending {
                info: info.clone(),
                secret_hash: secret_hash.to_ascii_lowercase(),
                decision: None,
            },
        );
        Ok(info)
    }
    pub fn pending(&self, now: u64) -> Vec<PairingInfo> {
        let mut state = self.state.lock().unwrap();
        state.pending.retain(|_, p| p.info.expires_at > now);
        let mut pending: Vec<_> = state
            .pending
            .values()
            .filter(|p| p.decision.is_none())
            .map(|p| p.info.clone())
            .collect();
        pending.sort_by_key(|p| (p.expires_at, p.id.clone()));
        pending
    }
    pub fn decide(&self, id: &str, grant: Option<ConnectionGrant>, now: u64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let pending = state
            .pending
            .get_mut(id)
            .context("Pairing request no longer exists")?;
        if pending.info.expires_at <= now || pending.decision.is_some() {
            bail!("Pairing request expired or was already decided");
        }
        if let Some(grant) = &grant {
            grant.validate()?;
            if !grant.is_subset_of(&pending.info.grant) {
                bail!("Approval cannot expand the requested permissions or wallets");
            }
        }
        pending.decision = Some(grant);
        Ok(())
    }
    pub fn collect(&self, id: &str, secret: &str, now: u64) -> Result<Option<IssuedCredential>> {
        if !(32..=256).contains(&secret.len()) {
            bail!("Invalid pairing secret");
        }
        let mut state = self.state.lock().unwrap();
        let pending = state
            .pending
            .get(id)
            .context("Pairing request expired or was already collected")?;
        if pending.info.expires_at <= now {
            state.pending.remove(id);
            bail!("Pairing request expired");
        }
        if hash(secret) != pending.secret_hash {
            bail!("Invalid pairing secret");
        }
        match pending.decision.clone() {
            None => Ok(None),
            Some(None) => {
                bail!("Pairing request denied");
            }
            Some(Some(grant)) => {
                let issued = self.issue_locked(&mut state, grant)?;
                state.pending.remove(id);
                Ok(Some(issued))
            }
        }
    }
    pub fn cancel_pairings(&self) {
        let mut state = self.state.lock().unwrap();
        state.pending.clear();
        state.operations.clear();
    }
}

impl crate::RpcAuthenticator for CredentialStore {
    fn request_approval(
        &self,
        request: crate::RpcApprovalRequest,
    ) -> Result<
        tokio::sync::oneshot::Receiver<Result<sage_permissions::ApprovalBinding, String>>,
        String,
    > {
        self.enqueue_operation(request)
    }

    fn authenticate(&self, headers: &axum::http::HeaderMap) -> std::result::Result<Caller, String> {
        let token = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or("Bearer credential required")?;
        self.caller(token).map_err(|err| err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grant() -> ConnectionGrant {
        ConnectionGrant {
            name: "Reader".into(),
            kind: ConnectionKind::Service,
            capabilities: [WalletCapability::WalletGetVersion].into(),
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        }
    }
    #[test]
    fn persisted_credentials_are_hashed_and_rotation_and_revocation_take_effect() {
        let dir = tempfile::tempdir().unwrap();
        let store = CredentialStore::open(dir.path()).unwrap();
        let issued = store.issue(grant()).unwrap();
        assert!(
            !fs::read_to_string(&store.path)
                .unwrap()
                .contains(&issued.token)
        );
        assert!(store.caller(&issued.token).is_ok());
        let rotated = store.rotate(&issued.credential.id).unwrap();
        assert!(store.caller(&issued.token).is_err());
        let reopened = CredentialStore::open(dir.path()).unwrap();
        assert!(reopened.caller(&rotated.token).is_ok());
        reopened.revoke(&rotated.credential.id).unwrap();
        assert!(reopened.caller(&rotated.token).is_err());
    }
    #[test]
    fn pairing_requires_approval_and_secret_and_can_only_be_collected_once() {
        let dir = tempfile::tempdir().unwrap();
        let store = CredentialStore::open(dir.path()).unwrap();
        let secret = random_hex(32);
        let pending = store.request_pairing(grant(), hash(&secret), 100).unwrap();
        assert!(store.collect(&pending.id, &secret, 101).unwrap().is_none());
        assert!(store.collect(&pending.id, &random_hex(32), 101).is_err());
        let mut wider = grant();
        wider.wallet_scope = WalletScope::AllWallets;
        assert!(store.decide(&pending.id, Some(wider), 101).is_err());
        store.decide(&pending.id, Some(grant()), 101).unwrap();
        let issued = store.collect(&pending.id, &secret, 102).unwrap().unwrap();
        assert!(store.caller(&issued.token).is_ok());
        assert!(store.collect(&pending.id, &secret, 103).is_err());
    }
    #[test]
    fn pairing_expires_and_is_bounded_and_denials_never_issue_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let store = CredentialStore::open(dir.path()).unwrap();
        let secret = random_hex(32);
        let p = store.request_pairing(grant(), hash(&secret), 100).unwrap();
        store.decide(&p.id, None, 101).unwrap();
        assert!(store.collect(&p.id, &secret, 102).is_err());
        for _ in 0..4 {
            store.request_pairing(grant(), hash(&secret), 100).unwrap();
        }
        assert!(store.request_pairing(grant(), hash(&secret), 100).is_err());
        assert!(store.collect(&p.id, &secret, 401).is_err());
        assert!(store.list().is_empty());
    }
}
