use crate::{app_state::AppState, error::Result};
use sage_permissions::WalletScope;
use sage_rpc::{
    ConnectionGrant, CredentialCapability, CredentialInfo, CredentialStore, IssuedCredential,
    PairingInfo, ScopedServer, credential_capabilities, credential_now, start_scoped_server,
};
use serde::Serialize;
use specta::{Type, specta};
use std::path::PathBuf;
use tauri::{State, command};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct Connections(pub Mutex<ConnectionState>);
#[derive(Default)]
pub struct ConnectionState {
    store: Option<CredentialStore>,
    root: PathBuf,
    server: Option<ScopedServer>,
    error: Option<String>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsView {
    pub running: bool,
    pub enabled: bool,
    pub port: u16,
    pub certificate_path: String,
    pub error: Option<String>,
    pub credentials: Vec<CredentialInfo>,
    pub pending: Vec<PairingInfo>,
    pub capabilities: Vec<CredentialCapability>,
    pub operations: Vec<sage_rpc::OperationReview>,
    pub owner_only_methods: Vec<String>,
}
fn internal(error: impl std::fmt::Display) -> crate::error::Error {
    std::io::Error::other(error.to_string()).into()
}
impl ConnectionState {
    fn store(&self) -> Result<&CredentialStore> {
        self.store.as_ref().ok_or_else(|| {
            internal(
                self.error
                    .as_deref()
                    .unwrap_or("Connections are not initialized"),
            )
        })
    }
}

pub async fn initialize(state: &Connections, sage: AppState) {
    let root = sage.lock().await.path.clone();
    let mut state = state.0.lock().await;
    state.root = root.clone();
    match CredentialStore::open(&root) {
        Ok(store) => {
            let (enabled, port) = store.settings();
            if enabled {
                match start_scoped_server(sage, store.clone(), &root, port).await {
                    Ok(server) => state.server = Some(server),
                    Err(error) => state.error = Some(error.to_string()),
                }
            }
            state.store = Some(store);
        }
        Err(error) => state.error = Some(error.to_string()),
    }
}

async fn validate_wallets(sage: &AppState, grant: &ConnectionGrant) -> Result<()> {
    grant.validate().map_err(internal)?;
    if let WalletScope::SelectedWallets { fingerprints } = &grant.wallet_scope {
        let sage = sage.lock().await;
        let wallets = sage
            .get_wallets(sage_api::GetWallets {})
            .map_err(crate::error::Error::from)?;
        if fingerprints.iter().any(|fp| {
            !wallets
                .wallets
                .iter()
                .any(|wallet| wallet.fingerprint == *fp)
        }) {
            return Err(internal(
                "A selected wallet no longer exists. Refresh and select wallets again.",
            ));
        }
    }
    Ok(())
}

#[command]
#[specta]
pub async fn connections_review(state: State<'_, Connections>) -> Result<ConnectionsView> {
    let state = state.0.lock().await;
    let store = state.store.as_ref();
    let (enabled, port) = store.map_or((false, 9258), CredentialStore::settings);
    Ok(ConnectionsView {
        running: state.server.as_ref().is_some_and(ScopedServer::is_running),
        enabled,
        port,
        certificate_path: state
            .root
            .join("scoped-rpc/server.crt")
            .to_string_lossy()
            .into(),
        error: state.error.clone(),
        credentials: store.map_or_else(Vec::new, CredentialStore::list),
        pending: store.map_or_else(Vec::new, |store| store.pending(credential_now())),
        capabilities: credential_capabilities(),
        operations: store.map_or_else(Vec::new, CredentialStore::operations),
        owner_only_methods: sage_permissions::RPC_POLICIES
            .iter()
            .filter(|p| p.capability.is_none())
            .map(|p| p.method.into())
            .collect(),
    })
}

#[command]
#[specta]
pub async fn connections_set_server(
    state: State<'_, Connections>,
    sage: State<'_, AppState>,
    enabled: bool,
    port: u16,
) -> Result<()> {
    if port < 1024 {
        return Err(internal("Port must be between 1024 and 65535"));
    }
    let mut state = state.0.lock().await;
    let store = state.store()?.clone();
    if enabled {
        if state
            .server
            .as_ref()
            .is_some_and(|s| s.is_running() && s.port == port)
        {
            return Ok(());
        }
        // Bind the replacement before stopping a running listener. On failure, keep the old one.
        let server = start_scoped_server((*sage).clone(), store.clone(), &state.root, port)
            .await
            .map_err(internal)?;
        if let Err(error) = store.set_settings(true, port) {
            server.stop();
            return Err(internal(error));
        }
        if let Some(previous) = state.server.replace(server) {
            previous.stop();
        }
    } else {
        store.set_settings(false, port).map_err(internal)?;
        if let Some(previous) = state.server.take() {
            previous.stop();
        }
        store.cancel_pairings();
    }
    state.error = None;
    Ok(())
}

#[command]
#[specta]
pub async fn connections_create(
    state: State<'_, Connections>,
    sage: State<'_, AppState>,
    grant: ConnectionGrant,
) -> Result<IssuedCredential> {
    validate_wallets(&sage, &grant).await?;
    state.0.lock().await.store()?.issue(grant).map_err(internal)
}
#[command]
#[specta]
pub async fn connections_revoke(state: State<'_, Connections>, id: String) -> Result<()> {
    state.0.lock().await.store()?.revoke(&id).map_err(internal)
}
#[command]
#[specta]
pub async fn connections_rotate(
    state: State<'_, Connections>,
    id: String,
) -> Result<IssuedCredential> {
    state.0.lock().await.store()?.rotate(&id).map_err(internal)
}
#[command]
#[specta]
pub async fn connections_decide_pairing(
    state: State<'_, Connections>,
    sage: State<'_, AppState>,
    id: String,
    grant: Option<ConnectionGrant>,
) -> Result<()> {
    if let Some(grant) = &grant {
        validate_wallets(&sage, grant).await?;
    }
    state
        .0
        .lock()
        .await
        .store()?
        .decide(&id, grant, credential_now())
        .map_err(internal)
}

#[command]
#[specta]
pub async fn connections_decide_operation(
    state: State<'_, Connections>,
    id: String,
    approved: bool,
) -> Result<()> {
    state
        .0
        .lock()
        .await
        .store()?
        .decide_operation(&id, approved)
        .map_err(internal)
}
