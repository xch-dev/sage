use std::{fmt, future::Future, path::Path, pin::Pin, sync::Arc};

use async_trait::async_trait;
use sage_api::{ErrorKind, GetWallet};
use sage_auth::{
    Agent, AgentServerSettings, AgentStore, AgentToken, Capability, DEFAULT_AGENT_SERVER_PORT,
    Decision, WalletScope, authorize, unix_timestamp_ms,
};
use sage_mcp::{
    AgentServerHandle, AgentTool, AgentToolExecutor, ToolError, agent_tools, start_agent_server,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use specta::{Type, specta};
use tauri::{AppHandle, Manager, State, command};
use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::{Error, Result},
};

const MAX_AGENT_NAME_LEN: usize = 64;
const MIN_AGENT_SERVER_PORT: u16 = 1024;

type ServerStart = Pin<Box<dyn Future<Output = anyhow::Result<AgentServerHandle>> + Send>>;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentServerStatus {
    pub running: bool,
    pub enabled: bool,
    pub port: u16,
    /// Why the server failed to start with Sage, while it is still stopped.
    pub start_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityView {
    pub key: String,
    pub label: String,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAgent {
    pub agent: Agent,
    pub token: String,
}

impl fmt::Debug for CreatedAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreatedAgent")
            .field("agent", &self.agent)
            .field("token", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Default)]
struct AgentServerState {
    handle: Option<AgentServerHandle>,
    start_error: Option<String>,
}

/// The agent store and server, managed on desktop only.
#[derive(Debug)]
pub struct AgentsState {
    store: Option<AgentStore>,
    open_error: Option<String>,
    server: Mutex<AgentServerState>,
}

impl AgentsState {
    /// Opens the store in `path`, keeping the error instead if it can't be opened.
    pub async fn open(path: &Path) -> Self {
        let (store, open_error) = match AgentStore::open(path).await {
            Ok(store) => (Some(store), None),
            Err(err) => {
                error!("failed to open the agents database: {err:#}");
                (None, Some(format!("{err:#}")))
            }
        };

        Self {
            store,
            open_error,
            server: Mutex::default(),
        }
    }

    fn store(&self) -> Result<&AgentStore> {
        self.store.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Internal,
                format!(
                    "The agents database couldn't be opened: {}",
                    self.open_error.as_deref().unwrap_or_default()
                ),
            )
        })
    }
}

/// Runs agent tool calls against the wallet logged in to Sage.
#[derive(Debug, Clone)]
pub struct SageAgentExecutor {
    app_state: AppState,
}

impl SageAgentExecutor {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl AgentToolExecutor for SageAgentExecutor {
    async fn current_fingerprint(&self) -> Option<u32> {
        self.app_state.lock().await.config.global.fingerprint
    }

    async fn call(
        &self,
        agent: &Agent,
        tool: &AgentTool,
        arguments: Value,
    ) -> std::result::Result<Value, ToolError> {
        // Held until the endpoint returns, so that the wallet can't change after the check.
        let sage = self.app_state.lock().await;
        match authorize(
            &agent.identity(),
            tool.capability,
            sage.config.global.fingerprint,
        )? {
            Decision::Allowed => {}
            _ => return Err(ToolError::approval_unsupported()),
        }

        match tool.capability {
            // The logged in wallet, whichever fingerprint the agent asks for.
            Capability::WalletGetWallet => {
                respond(tool, sage.get_wallet(GetWallet { fingerprint: None }))
            }
            Capability::WalletGetSyncStatus => {
                respond(tool, sage.get_sync_status(params(tool, arguments)?).await)
            }
            Capability::WalletGetVersion => {
                respond(tool, sage.get_version(params(tool, arguments)?))
            }
            Capability::WalletGetToken => {
                respond(tool, sage.get_token(params(tool, arguments)?).await)
            }
            Capability::WalletGetCoins => {
                respond(tool, sage.get_coins(params(tool, arguments)?).await)
            }
            Capability::WalletGetCoinsByIds => {
                respond(tool, sage.get_coins_by_ids(params(tool, arguments)?).await)
            }
            Capability::WalletGetSpendableCoinCount => respond(
                tool,
                sage.get_spendable_coin_count(params(tool, arguments)?)
                    .await,
            ),
            Capability::WalletGetTransactions => {
                respond(tool, sage.get_transactions(params(tool, arguments)?).await)
            }
            Capability::WalletGetTransaction => {
                respond(tool, sage.get_transaction(params(tool, arguments)?).await)
            }
            Capability::WalletGetPendingTransactions => respond(
                tool,
                sage.get_pending_transactions(params(tool, arguments)?)
                    .await,
            ),
            Capability::WalletGetXchUsdPrice => {
                respond(tool, sage.get_xch_usd_price(params(tool, arguments)?).await)
            }
            Capability::WalletCheckAddress => {
                respond(tool, sage.check_address(params(tool, arguments)?).await)
            }
        }
    }
}

fn params<T: DeserializeOwned>(
    tool: &AgentTool,
    arguments: Value,
) -> std::result::Result<T, ToolError> {
    serde_json::from_value(arguments).map_err(|err| {
        ToolError::new(
            "invalid_request",
            format!("Failed to decode {} params: {err}", tool.name),
        )
    })
}

fn respond<T: Serialize>(
    tool: &AgentTool,
    result: sage::Result<T>,
) -> std::result::Result<Value, ToolError> {
    let response = result.map_err(|err| {
        ToolError::new(
            "internal_error",
            format!("failed to execute {}: {err}", tool.name),
        )
    })?;

    serde_json::to_value(response).map_err(|err| {
        ToolError::new(
            "internal_error",
            format!("failed to encode {} result: {err}", tool.name),
        )
    })
}

fn validate_agent_name(name: &str) -> std::result::Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Agent name can't be empty".to_string());
    }
    if name.chars().count() > MAX_AGENT_NAME_LEN {
        return Err(format!(
            "Agent name can't be longer than {MAX_AGENT_NAME_LEN} characters"
        ));
    }
    Ok(())
}

fn validate_capabilities(
    capabilities: &[Capability],
    grantable: &[Capability],
) -> std::result::Result<(), String> {
    match capabilities.iter().find(|c| !grantable.contains(c)) {
        Some(capability) => Err(format!(
            "Capability {} can't be granted to agents",
            capability.key()
        )),
        None => Ok(()),
    }
}

fn validate_wallet_scope(
    wallet_scope: &WalletScope,
    existing: &[u32],
) -> std::result::Result<(), String> {
    let WalletScope::SelectedWallets { fingerprints } = wallet_scope else {
        return Ok(());
    };

    match fingerprints.iter().find(|f| !existing.contains(f)) {
        Some(fingerprint) => Err(format!("Wallet {fingerprint} doesn't exist")),
        None => Ok(()),
    }
}

async fn wallet_fingerprints(app_state: &AppState) -> Vec<u32> {
    let sage = app_state.lock().await;
    sage.wallet_config
        .wallets
        .iter()
        .map(|wallet| wallet.fingerprint)
        .collect()
}

fn error(kind: ErrorKind, reason: impl Into<String>) -> Error {
    Error {
        kind,
        reason: reason.into(),
        handle_error_code: None,
        recipients: None,
    }
}

fn bad_request(reason: impl Into<String>) -> Error {
    error(ErrorKind::Api, reason)
}

fn not_found(id: &str) -> Error {
    error(ErrorKind::NotFound, format!("Agent {id} not found"))
}

#[allow(clippy::needless_pass_by_value)] // Used with `map_err`
fn internal(err: anyhow::Error) -> Error {
    error(ErrorKind::Internal, format!("{err:#}"))
}

fn server_status(settings: AgentServerSettings, server: &AgentServerState) -> AgentServerStatus {
    let running_port = server.handle.as_ref().map(AgentServerHandle::port);
    AgentServerStatus {
        running: running_port.is_some(),
        enabled: settings.enabled,
        port: running_port.unwrap_or(settings.port),
        start_error: match running_port {
            Some(_) => None,
            None => server.start_error.clone().filter(|_| settings.enabled),
        },
    }
}

/// The status while the store couldn't be opened, which reports why.
fn status_without_store(open_error: impl Into<String>) -> AgentServerStatus {
    AgentServerStatus {
        running: false,
        enabled: false,
        port: DEFAULT_AGENT_SERVER_PORT,
        start_error: Some(open_error.into()),
    }
}

/// The capabilities agents can be granted: those with a tool, in catalog order.
fn grantable_capabilities() -> Vec<Capability> {
    agent_tools().iter().map(|tool| tool.capability).collect()
}

fn capability_views() -> Vec<AgentCapabilityView> {
    grantable_capabilities()
        .into_iter()
        .map(|capability| AgentCapabilityView {
            key: capability.key().to_string(),
            label: capability.label().to_string(),
            description: capability.description().to_string(),
        })
        .collect()
}

/// Moves the server to `enabled`/`port`, leaving the old one running if the new bind fails.
async fn change_server<F, Fut>(
    server: &mut Option<AgentServerHandle>,
    enabled: bool,
    port: u16,
    start: F,
) -> anyhow::Result<()>
where
    F: FnOnce(u16) -> Fut,
    Fut: Future<Output = anyhow::Result<AgentServerHandle>>,
{
    if !enabled {
        if let Some(handle) = server.take() {
            handle.stop().await;
        }
        return Ok(());
    }

    if server.as_ref().is_some_and(|handle| handle.port() == port) {
        return Ok(());
    }

    let handle = start(port).await?;
    if let Some(old) = server.replace(handle) {
        old.stop().await;
    }
    Ok(())
}

/// Stores `next` and moves the server to it, restoring the stored settings if either step fails.
async fn apply_server_settings<F, Fut>(
    store: &AgentStore,
    server: &mut Option<AgentServerHandle>,
    next: AgentServerSettings,
    start: F,
) -> anyhow::Result<()>
where
    F: FnOnce(u16) -> Fut,
    Fut: Future<Output = anyhow::Result<AgentServerHandle>>,
{
    let previous = store.get_server_settings().await?;

    let result = match store.set_server_settings(next).await {
        Ok(_) => change_server(server, next.enabled, next.port, start).await,
        Err(err) => Err(err),
    };

    if result.is_err()
        && let Err(err) = store.set_server_settings(previous).await
    {
        error!("failed to restore agent server settings: {err}");
    }
    result
}

fn server_starter(app_handle: &AppHandle, store: &AgentStore) -> impl FnOnce(u16) -> ServerStart {
    let store = store.clone();
    let executor = Arc::new(SageAgentExecutor::new(
        app_handle.state::<AppState>().inner().clone(),
    ));
    let version = app_handle.package_info().version.to_string();
    move |port| Box::pin(start_agent_server(store, executor, version, port))
}

pub async fn start_agent_server_if_enabled(app_handle: AppHandle) {
    let state = app_handle.state::<AgentsState>();
    let Some(store) = &state.store else {
        return;
    };

    let wallets = wallet_fingerprints(&app_handle.state::<AppState>()).await;
    if let Err(err) = store.retain_agent_wallets(&wallets).await {
        error!("failed to drop deleted wallets from agent scopes: {err}");
    }

    let settings = match store.get_server_settings().await {
        Ok(settings) => settings,
        Err(err) => {
            error!("failed to read agent server settings: {err}");
            return;
        }
    };

    if !settings.enabled {
        return;
    }

    let mut server = state.server.lock().await;
    let start = server_starter(&app_handle, store);
    if let Err(err) = change_server(&mut server.handle, true, settings.port, start).await {
        error!("failed to start agent server: {err}");
        server.start_error = Some(err.to_string());
    }
}

#[command]
#[specta]
pub async fn agents_get_server_status(state: State<'_, AgentsState>) -> Result<AgentServerStatus> {
    let Some(store) = &state.store else {
        return Ok(status_without_store(
            state.open_error.clone().unwrap_or_default(),
        ));
    };

    let server = state.server.lock().await;
    let settings = store.get_server_settings().await.map_err(internal)?;
    Ok(server_status(settings, &server))
}

#[command]
#[specta]
pub async fn agents_set_server_settings(
    app_handle: AppHandle,
    state: State<'_, AgentsState>,
    enabled: bool,
    port: u16,
) -> Result<AgentServerStatus> {
    let store = state.store()?;

    if port < MIN_AGENT_SERVER_PORT {
        return Err(bad_request(format!(
            "Port must be between {MIN_AGENT_SERVER_PORT} and 65535"
        )));
    }

    let mut server = state.server.lock().await;
    let settings = AgentServerSettings { enabled, port };
    let start = server_starter(&app_handle, store);
    apply_server_settings(store, &mut server.handle, settings, start)
        .await
        .map_err(internal)?;
    server.start_error = None;
    Ok(server_status(settings, &server))
}

#[command]
#[specta]
pub async fn agents_list_agents(
    app_state: State<'_, AppState>,
    state: State<'_, AgentsState>,
) -> Result<Vec<Agent>> {
    let store = state.store()?;
    let wallets = wallet_fingerprints(&app_state).await;
    store.retain_agent_wallets(&wallets).await.map_err(internal)
}

#[command]
#[specta]
pub async fn agents_list_capabilities(
    state: State<'_, AgentsState>,
) -> Result<Vec<AgentCapabilityView>> {
    state.store()?;
    Ok(capability_views())
}

#[command]
#[specta]
pub async fn agents_create_agent(
    app_state: State<'_, AppState>,
    state: State<'_, AgentsState>,
    name: String,
    capabilities: Vec<Capability>,
    wallet_scope: WalletScope,
) -> Result<CreatedAgent> {
    let store = state.store()?;
    validate_agent_name(&name).map_err(bad_request)?;
    validate_capabilities(&capabilities, &grantable_capabilities()).map_err(bad_request)?;
    validate_wallet_scope(&wallet_scope, &wallet_fingerprints(&app_state).await)
        .map_err(bad_request)?;

    let now = unix_timestamp_ms();
    let agent = Agent {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        capabilities: capabilities.into_iter().collect(),
        wallet_scope,
        created_at_ms: now,
        updated_at_ms: now,
        last_used_at_ms: None,
    };
    let token = AgentToken::generate();

    store
        .insert_agent(&agent, &token.hash())
        .await
        .map_err(internal)?;

    Ok(CreatedAgent {
        agent,
        token: token.expose().to_string(),
    })
}

#[command]
#[specta]
pub async fn agents_update_agent(
    app_state: State<'_, AppState>,
    state: State<'_, AgentsState>,
    id: String,
    name: String,
    capabilities: Vec<Capability>,
    wallet_scope: WalletScope,
) -> Result<Agent> {
    let store = state.store()?;
    validate_agent_name(&name).map_err(bad_request)?;
    validate_capabilities(&capabilities, &grantable_capabilities()).map_err(bad_request)?;
    validate_wallet_scope(&wallet_scope, &wallet_fingerprints(&app_state).await)
        .map_err(bad_request)?;

    let capabilities = capabilities.into_iter().collect();
    if !store
        .update_agent(&id, name.trim(), &capabilities, &wallet_scope)
        .await
        .map_err(internal)?
    {
        return Err(not_found(&id));
    }

    store
        .get_agent(&id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found(&id))
}

#[command]
#[specta]
pub async fn agents_rotate_agent_token(
    state: State<'_, AgentsState>,
    id: String,
) -> Result<String> {
    let store = state.store()?;
    let token = AgentToken::generate();

    if !store
        .set_agent_token_hash(&id, &token.hash())
        .await
        .map_err(internal)?
    {
        return Err(not_found(&id));
    }

    Ok(token.expose().to_string())
}

#[command]
#[specta]
pub async fn agents_revoke_agent(state: State<'_, AgentsState>, id: String) -> Result<()> {
    let store = state.store()?;
    if !store.delete_agent(&id).await.map_err(internal)? {
        return Err(not_found(&id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, net::Ipv4Addr};

    use async_trait::async_trait;
    use rustls::crypto::aws_lc_rs::default_provider;
    use sage::Sage;
    use sage_api::{GenerateMnemonic, ImportWallet, IncreaseDerivationIndex, Login, Logout};
    use sage_mcp::find_agent_tool;
    use serde_json::{Value, json};
    use tempfile::{TempDir, tempdir};

    use super::*;

    struct NoopExecutor;

    #[async_trait]
    impl AgentToolExecutor for NoopExecutor {
        async fn current_fingerprint(&self) -> Option<u32> {
            None
        }

        async fn call(
            &self,
            _agent: &Agent,
            _tool: &AgentTool,
            _arguments: Value,
        ) -> std::result::Result<Value, ToolError> {
            Ok(Value::Null)
        }
    }

    fn starter(store: &AgentStore) -> impl FnOnce(u16) -> ServerStart {
        let store = store.clone();
        move |port| {
            Box::pin(start_agent_server(
                store,
                Arc::new(NoopExecutor),
                "test".to_string(),
                port,
            ))
        }
    }

    fn agent(capabilities: &[Capability], wallet_scope: WalletScope) -> Agent {
        Agent {
            id: "a".to_string(),
            name: "agent".to_string(),
            capabilities: capabilities.iter().copied().collect(),
            wallet_scope,
            created_at_ms: 1,
            updated_at_ms: 1,
            last_used_at_ms: None,
        }
    }

    /// A `Sage` in a temp dir with two imported wallets, logged out.
    async fn sage_with_two_wallets() -> (AppState, u32, u32, TempDir) {
        let _ = default_provider().install_default();

        let dir = tempdir().unwrap();
        let mut sage = Sage::new(dir.path(), true);
        sage.config.network.target_peers = 1;
        sage.config.network.discover_peers = false;
        sage.config.network.default_network = "testnet11".to_string();
        sage.config.global.log_level = "WARN".to_string();
        let _events = sage.initialize().await.unwrap();

        let mut fingerprints = Vec::new();
        for name in ["A", "B"] {
            let mnemonic = sage
                .generate_mnemonic(GenerateMnemonic {
                    use_24_words: false,
                })
                .unwrap()
                .mnemonic;
            let imported = sage
                .import_wallet(ImportWallet {
                    name: name.to_string(),
                    key: mnemonic,
                    derivation_index: 0,
                    hardened: None,
                    unhardened: None,
                    save_secrets: false,
                    login: false,
                    emoji: None,
                })
                .await
                .unwrap();
            fingerprints.push(imported.fingerprint);
        }
        sage.logout(Logout {}).await.unwrap();

        let state = Arc::new(Mutex::new(sage));
        (state, fingerprints[0], fingerprints[1], dir)
    }

    async fn login(state: &AppState, fingerprint: u32) {
        state
            .lock()
            .await
            .login(Login { fingerprint })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn failed_start_keeps_stored_settings() {
        let dir = tempdir().unwrap();
        let store = AgentStore::open(dir.path()).await.unwrap();
        let before = store.get_server_settings().await.unwrap();

        let occupied = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let busy_port = occupied.local_addr().unwrap().port();
        let next = AgentServerSettings {
            enabled: true,
            port: busy_port,
        };

        let mut server = None;
        assert!(
            apply_server_settings(&store, &mut server, next, starter(&store))
                .await
                .is_err()
        );
        assert!(server.is_none());
        assert_eq!(store.get_server_settings().await.unwrap(), before);
    }

    #[tokio::test]
    async fn failed_port_change_keeps_old_server() {
        let dir = tempdir().unwrap();
        let store = AgentStore::open(dir.path()).await.unwrap();
        let mut server = None;
        change_server(&mut server, true, 0, starter(&store))
            .await
            .unwrap();
        let old_port = server.as_ref().unwrap().port();

        let occupied = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let busy_port = occupied.local_addr().unwrap().port();
        assert!(
            change_server(&mut server, true, busy_port, starter(&store))
                .await
                .is_err()
        );
        assert_eq!(server.as_ref().map(AgentServerHandle::port), Some(old_port));

        let settings = AgentServerSettings {
            enabled: true,
            port: busy_port,
        };
        let mut state = AgentServerState {
            handle: server,
            start_error: Some("stale".to_string()),
        };
        let status = server_status(settings, &state);
        assert_eq!(status.port, old_port);
        assert_eq!(status.start_error, None);
        state.handle.take().unwrap().stop().await;
    }

    #[test]
    fn scope_naming_a_missing_wallet_is_rejected() {
        let scope = |fingerprints: &[u32]| WalletScope::SelectedWallets {
            fingerprints: fingerprints.to_vec(),
        };

        assert!(validate_wallet_scope(&WalletScope::AllWallets, &[]).is_ok());
        assert!(validate_wallet_scope(&scope(&[7]), &[7, 8]).is_ok());
        assert!(validate_wallet_scope(&scope(&[7, 9]), &[7, 8]).is_err());
    }

    #[test]
    fn start_error_is_reported_only_while_enabled_and_stopped() {
        let state = AgentServerState {
            handle: None,
            start_error: Some("address in use".to_string()),
        };
        let enabled = AgentServerSettings {
            enabled: true,
            port: 9258,
        };
        let disabled = AgentServerSettings {
            enabled: false,
            port: 9258,
        };

        assert_eq!(
            server_status(enabled, &state).start_error.as_deref(),
            Some("address in use")
        );
        assert_eq!(server_status(disabled, &state).start_error, None);
    }

    #[test]
    fn status_reports_the_open_error_even_when_disabled() {
        let status = status_without_store("disk full");
        assert!(!status.running && !status.enabled);
        assert_eq!(status.port, 9258);
        assert_eq!(status.start_error.as_deref(), Some("disk full"));
    }

    #[tokio::test]
    async fn port_change_replaces_server_and_disable_stops_it() {
        let dir = tempdir().unwrap();
        let store = AgentStore::open(dir.path()).await.unwrap();
        let mut server = None;
        change_server(&mut server, true, 0, starter(&store))
            .await
            .unwrap();
        let old_port = server.as_ref().unwrap().port();

        let free = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let new_port = free.local_addr().unwrap().port();
        drop(free);
        change_server(&mut server, true, new_port, starter(&store))
            .await
            .unwrap();
        assert_eq!(server.as_ref().map(AgentServerHandle::port), Some(new_port));
        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, old_port)).expect("old port freed");

        change_server(&mut server, false, new_port, starter(&store))
            .await
            .unwrap();
        assert!(server.is_none());
    }

    #[test]
    fn created_agent_debug_redacts_token() {
        let created = CreatedAgent {
            agent: Agent {
                capabilities: BTreeSet::new(),
                ..agent(&[], WalletScope::AllWallets)
            },
            token: "sage_agent_secret".to_string(),
        };
        assert!(!format!("{created:?}").contains("sage_agent_secret"));
    }

    #[test]
    fn validate_rejects_blank_name() {
        assert!(validate_agent_name("   ").is_err());
        assert!(validate_agent_name("").is_err());
        assert!(validate_agent_name(&"x".repeat(65)).is_err());
    }

    #[test]
    fn validate_accepts_a_name() {
        assert!(validate_agent_name("Claude").is_ok());
        assert!(validate_agent_name(&"x".repeat(64)).is_ok());
    }

    #[test]
    fn grantable_capabilities_are_the_tool_catalog() {
        let keys: Vec<_> = capability_views()
            .into_iter()
            .map(|view| view.key)
            .collect();
        let expected: Vec<_> = agent_tools()
            .iter()
            .map(|tool| tool.capability.key().to_string())
            .collect();
        assert_eq!(keys, expected);
    }

    #[test]
    fn capability_without_a_tool_is_rejected() {
        let grantable = [Capability::WalletGetVersion];
        assert!(validate_capabilities(&[], &grantable).is_ok());
        assert!(validate_capabilities(&[Capability::WalletGetVersion], &grantable).is_ok());
        assert!(
            validate_capabilities(
                &[Capability::WalletGetVersion, Capability::WalletGetCoins],
                &grantable
            )
            .is_err()
        );
        assert!(validate_capabilities(Capability::ALL, &grantable_capabilities()).is_ok());
    }

    /// Calls every tool through the executor, so that a mis-wired arm returns the wrong shape.
    #[tokio::test]
    async fn executor_dispatches_every_tool_to_its_endpoint() {
        let (state, a, _b, _dir) = sage_with_two_wallets().await;
        login(&state, a).await;
        // Derives addresses locally, since there is no peer to sync from.
        state
            .lock()
            .await
            .increase_derivation_index(IncreaseDerivationIndex {
                index: 1,
                hardened: Some(false),
                unhardened: Some(true),
            })
            .await
            .unwrap();

        let executor = SageAgentExecutor::new(state);
        let agent = agent(Capability::ALL, WalletScope::AllWallets);
        let call = async |name: &str, arguments: Value| {
            let tool = find_agent_tool(name).unwrap();
            executor.call(&agent, tool, arguments).await
        };

        let receive_address = call("get_sync_status", json!({})).await.unwrap()["receive_address"]
            .as_str()
            .unwrap()
            .to_string();

        let cases = [
            ("get_wallet", json!({}), &["wallet"][..]),
            (
                "get_sync_status",
                json!({}),
                &[
                    "selectable_balance",
                    "unit",
                    "synced_coins",
                    "total_coins",
                    "receive_address",
                    "burn_address",
                    "unhardened_derivation_index",
                    "hardened_derivation_index",
                    "checked_files",
                    "total_files",
                    "database_size",
                ][..],
            ),
            ("get_version", json!({}), &["version"][..]),
            ("get_token", json!({ "asset_id": null }), &["token"][..]),
            (
                "get_coins",
                json!({ "offset": 0, "limit": 10 }),
                &["coins", "total"][..],
            ),
            (
                "get_coins_by_ids",
                json!({ "coin_ids": [] }),
                &["coins"][..],
            ),
            (
                "get_spendable_coin_count",
                json!({ "asset_id": null }),
                &["count"][..],
            ),
            (
                "get_transactions",
                json!({ "offset": 0, "limit": 10, "ascending": false, "find_value": null }),
                &["transactions", "total"][..],
            ),
            (
                "get_transaction",
                json!({ "height": 1 }),
                &["transaction"][..],
            ),
            ("get_pending_transactions", json!({}), &["transactions"][..]),
            (
                "check_address",
                json!({ "address": receive_address }),
                &["valid"][..],
            ),
        ];

        // `get_xch_usd_price` fetches the price over the network, which tests must not use.
        let mut expected_tools: BTreeSet<_> = cases.iter().map(|(name, ..)| *name).collect();
        expected_tools.insert("get_xch_usd_price");
        let catalog: BTreeSet<_> = agent_tools().iter().map(|tool| tool.name).collect();
        assert_eq!(expected_tools, catalog);

        for (name, arguments, keys) in cases {
            let response = call(name, arguments)
                .await
                .unwrap_or_else(|err| panic!("{name}: {}: {}", err.code, err.message));
            let actual: BTreeSet<_> = response.as_object().unwrap().keys().cloned().collect();
            let expected: BTreeSet<_> = keys.iter().map(ToString::to_string).collect();
            assert_eq!(actual, expected, "{name}");
        }

        let wallet = call("get_wallet", json!({})).await.unwrap();
        assert_eq!(wallet["wallet"]["fingerprint"], json!(a));
        let owned = call("check_address", json!({ "address": receive_address }))
            .await
            .unwrap();
        assert_eq!(owned["valid"], json!(true));
    }

    #[tokio::test]
    async fn executor_rejects_malformed_arguments() {
        let (state, a, _b, _dir) = sage_with_two_wallets().await;
        login(&state, a).await;

        let executor = SageAgentExecutor::new(state);
        let tool = find_agent_tool("get_transaction").unwrap();
        let agent = agent(&[Capability::WalletGetTransaction], WalletScope::AllWallets);

        let err = executor
            .call(&agent, tool, json!({ "height": "not a number" }))
            .await
            .unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[tokio::test]
    async fn executor_ignores_a_fingerprint_argument_to_get_wallet() {
        let (state, a, b, _dir) = sage_with_two_wallets().await;
        login(&state, a).await;

        let executor = SageAgentExecutor::new(state);
        let tool = find_agent_tool("get_wallet").unwrap();
        let agent = agent(&[Capability::WalletGetWallet], WalletScope::AllWallets);

        let response = executor
            .call(&agent, tool, json!({ "fingerprint": b }))
            .await
            .unwrap();
        assert_eq!(response["wallet"]["fingerprint"], json!(a));
    }

    #[tokio::test]
    async fn executor_refuses_after_switching_out_of_scope() {
        let (state, a, b, _dir) = sage_with_two_wallets().await;
        login(&state, a).await;
        login(&state, b).await;

        let executor = SageAgentExecutor::new(state);
        let tool = find_agent_tool("get_wallet").unwrap();
        let agent = agent(
            &[Capability::WalletGetWallet],
            WalletScope::SelectedWallets {
                fingerprints: vec![a],
            },
        );

        let err = executor.call(&agent, tool, json!({})).await.unwrap_err();
        assert_eq!(err.code, "wallet_not_in_scope");
    }

    #[tokio::test]
    async fn executor_refuses_when_logged_out() {
        let (state, _a, _b, _dir) = sage_with_two_wallets().await;

        let executor = SageAgentExecutor::new(state);
        let tool = find_agent_tool("get_wallet").unwrap();
        let agent = agent(&[Capability::WalletGetWallet], WalletScope::AllWallets);

        let err = executor.call(&agent, tool, json!({})).await.unwrap_err();
        assert_eq!(err.code, "no_wallet");
    }
}
