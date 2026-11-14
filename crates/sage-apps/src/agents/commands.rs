use std::{fmt, future::Future, sync::Arc};

use futures::{FutureExt, future::BoxFuture};
use sage_api::ErrorKind;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, command};
use tracing::error;
use uuid::Uuid;

use crate::{
    AgentServerHandle, AgentServerSettings, AgentToken, AppsHostState, Result, SageAgent,
    SageAppWalletScope, SageAppsError, TauriAgentExecutor, UserBridgeCapability,
    agent_grantable_capabilities, get_user_capability_definition, start_agent_server,
    unix_timestamp_ms,
};

const MAX_AGENT_NAME_LEN: usize = 64;
const MIN_AGENT_SERVER_PORT: u16 = 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentServerStatus {
    pub running: bool,
    pub enabled: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityInfo {
    pub capability: UserBridgeCapability,
    pub label: String,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAgent {
    pub agent: SageAgent,
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

pub(crate) fn validate_agent_request(
    name: &str,
    capabilities: &[UserBridgeCapability],
) -> std::result::Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Agent name can't be empty".to_string());
    }
    if name.chars().count() > MAX_AGENT_NAME_LEN {
        return Err(format!(
            "Agent name can't be longer than {MAX_AGENT_NAME_LEN} characters"
        ));
    }

    let grantable = agent_grantable_capabilities();
    if let Some(capability) = capabilities.iter().find(|c| !grantable.contains(c)) {
        return Err(format!(
            "Capability {capability:?} can't be granted to agents"
        ));
    }

    Ok(())
}

fn bad_request(reason: impl Into<String>) -> SageAppsError {
    SageAppsError {
        kind: ErrorKind::Api,
        reason: reason.into(),
    }
}

fn not_found(id: &str) -> SageAppsError {
    SageAppsError {
        kind: ErrorKind::NotFound,
        reason: format!("Agent {id} not found"),
    }
}

fn server_status(
    settings: AgentServerSettings,
    server: Option<&AgentServerHandle>,
) -> AgentServerStatus {
    let running_port = server.map(AgentServerHandle::port);
    AgentServerStatus {
        running: running_port.is_some(),
        enabled: settings.enabled,
        port: running_port.unwrap_or(settings.port),
    }
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

fn server_starter(
    app_handle: &AppHandle,
    state: &AppsHostState,
) -> impl FnOnce(u16) -> BoxFuture<'static, anyhow::Result<AgentServerHandle>> {
    let db = state.db.clone();
    let executor = Arc::new(TauriAgentExecutor::new(app_handle.clone()));
    let version = app_handle.package_info().version.to_string();
    move |port| start_agent_server(db, executor, version, port).boxed()
}

pub async fn start_agent_server_if_enabled(app_handle: AppHandle) {
    let state = app_handle.state::<AppsHostState>();

    let settings = match state.db.get_agent_server_settings().await {
        Ok(settings) => settings,
        Err(err) => {
            error!("failed to read agent server settings: {err}");
            return;
        }
    };

    if !settings.enabled {
        return;
    }

    let mut server = state.agent_server.lock().await;
    let start = server_starter(&app_handle, &state);
    if let Err(err) = change_server(&mut server, true, settings.port, start).await {
        error!("failed to start agent server: {err}");
    }
}

#[command]
#[specta::specta]
pub async fn apps_get_agent_server_status(
    apps_state: State<'_, AppsHostState>,
) -> Result<AgentServerStatus> {
    let server = apps_state.agent_server.lock().await;
    let settings = apps_state.db.get_agent_server_settings().await?;
    Ok(server_status(settings, server.as_ref()))
}

#[command]
#[specta::specta]
pub async fn apps_set_agent_server_settings(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    enabled: bool,
    port: u16,
) -> Result<AgentServerStatus> {
    let mut server = apps_state.agent_server.lock().await;

    if port < MIN_AGENT_SERVER_PORT {
        return Err(bad_request(format!(
            "Port must be between {MIN_AGENT_SERVER_PORT} and 65535"
        )));
    }

    let start = server_starter(&app_handle, &apps_state);
    change_server(&mut server, enabled, port, start).await?;

    let settings = apps_state
        .db
        .set_agent_server_settings(AgentServerSettings { enabled, port })
        .await?;
    Ok(server_status(settings, server.as_ref()))
}

#[command]
#[specta::specta]
pub async fn apps_list_agents(apps_state: State<'_, AppsHostState>) -> Result<Vec<SageAgent>> {
    Ok(apps_state.db.list_agents().await?)
}

#[command]
#[specta::specta]
pub async fn apps_list_agent_capabilities() -> Result<Vec<AgentCapabilityInfo>> {
    Ok(agent_grantable_capabilities()
        .into_iter()
        .map(|capability| {
            let definition = get_user_capability_definition(capability);
            AgentCapabilityInfo {
                capability,
                label: definition.label().to_string(),
                description: definition.description().to_string(),
            }
        })
        .collect())
}

#[command]
#[specta::specta]
pub async fn apps_create_agent(
    apps_state: State<'_, AppsHostState>,
    name: String,
    capabilities: Vec<UserBridgeCapability>,
    wallet_scope: SageAppWalletScope,
) -> Result<CreatedAgent> {
    validate_agent_request(&name, &capabilities).map_err(bad_request)?;

    let now = unix_timestamp_ms();
    let agent = SageAgent {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        capabilities: capabilities.into_iter().collect(),
        wallet_scope,
        created_at_ms: now,
        updated_at_ms: now,
        last_used_at_ms: None,
    };
    let token = AgentToken::generate();

    apps_state.db.insert_agent(&agent, &token.hash()).await?;

    Ok(CreatedAgent {
        agent,
        token: token.expose().to_string(),
    })
}

#[command]
#[specta::specta]
pub async fn apps_update_agent(
    apps_state: State<'_, AppsHostState>,
    id: String,
    name: String,
    capabilities: Vec<UserBridgeCapability>,
    wallet_scope: SageAppWalletScope,
) -> Result<SageAgent> {
    validate_agent_request(&name, &capabilities).map_err(bad_request)?;

    let capabilities = capabilities.into_iter().collect();
    if !apps_state
        .db
        .update_agent(&id, name.trim(), &capabilities, &wallet_scope)
        .await?
    {
        return Err(not_found(&id));
    }

    apps_state
        .db
        .get_agent(&id)
        .await?
        .ok_or_else(|| not_found(&id))
}

#[command]
#[specta::specta]
pub async fn apps_rotate_agent_token(
    apps_state: State<'_, AppsHostState>,
    id: String,
) -> Result<String> {
    let token = AgentToken::generate();

    if !apps_state
        .db
        .set_agent_token_hash(&id, &token.hash())
        .await?
    {
        return Err(not_found(&id));
    }

    Ok(token.expose().to_string())
}

#[command]
#[specta::specta]
pub async fn apps_revoke_agent(apps_state: State<'_, AppsHostState>, id: String) -> Result<()> {
    if !apps_state.db.delete_agent(&id).await? {
        return Err(not_found(&id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, net::Ipv4Addr};

    use async_trait::async_trait;
    use serde_json::Value;
    use tempfile::tempdir;

    use super::*;
    use crate::{AgentCallError, AgentTool, AgentToolExecutor, AppsDb};

    struct NoopExecutor;

    #[async_trait]
    impl AgentToolExecutor for NoopExecutor {
        async fn call(
            &self,
            _agent: &SageAgent,
            _tool: &AgentTool,
            _arguments: Value,
        ) -> std::result::Result<Value, AgentCallError> {
            Ok(Value::Null)
        }
    }

    fn starter(
        db: &AppsDb,
    ) -> impl FnOnce(u16) -> BoxFuture<'static, anyhow::Result<AgentServerHandle>> {
        let db = db.clone();
        move |port| start_agent_server(db, Arc::new(NoopExecutor), "test".to_string(), port).boxed()
    }

    #[tokio::test]
    async fn failed_port_change_keeps_old_server() {
        let dir = tempdir().unwrap();
        let db = AppsDb::initialize(dir.path()).await.unwrap();
        let mut server = None;
        change_server(&mut server, true, 0, starter(&db))
            .await
            .unwrap();
        let old_port = server.as_ref().unwrap().port();

        let occupied = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let busy_port = occupied.local_addr().unwrap().port();
        assert!(
            change_server(&mut server, true, busy_port, starter(&db))
                .await
                .is_err()
        );
        assert_eq!(server.as_ref().map(AgentServerHandle::port), Some(old_port));

        let settings = AgentServerSettings {
            enabled: true,
            port: busy_port,
        };
        assert_eq!(server_status(settings, server.as_ref()).port, old_port);
        server.take().unwrap().stop().await;
    }

    #[tokio::test]
    async fn port_change_replaces_server_and_disable_stops_it() {
        let dir = tempdir().unwrap();
        let db = AppsDb::initialize(dir.path()).await.unwrap();
        let mut server = None;
        change_server(&mut server, true, 0, starter(&db))
            .await
            .unwrap();
        let old_port = server.as_ref().unwrap().port();

        let free = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let new_port = free.local_addr().unwrap().port();
        drop(free);
        change_server(&mut server, true, new_port, starter(&db))
            .await
            .unwrap();
        assert_eq!(server.as_ref().map(AgentServerHandle::port), Some(new_port));
        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, old_port)).expect("old port freed");

        change_server(&mut server, false, new_port, starter(&db))
            .await
            .unwrap();
        assert!(server.is_none());
    }

    #[test]
    fn created_agent_debug_redacts_token() {
        let created = CreatedAgent {
            agent: SageAgent {
                id: "a".to_string(),
                name: "agent".to_string(),
                capabilities: BTreeSet::new(),
                wallet_scope: SageAppWalletScope::AllWallets,
                created_at_ms: 1,
                updated_at_ms: 1,
                last_used_at_ms: None,
            },
            token: "sage_agent_secret".to_string(),
        };
        assert!(!format!("{created:?}").contains("sage_agent_secret"));
    }

    #[test]
    fn validate_rejects_secret_capability() {
        assert!(validate_agent_request("a", &[UserBridgeCapability::WalletGetSecretKey]).is_err());
    }

    #[test]
    fn validate_rejects_signing_capability() {
        assert!(
            validate_agent_request("a", &[UserBridgeCapability::WalletSignCoinSpends]).is_err()
        );
    }

    #[test]
    fn validate_rejects_blank_name() {
        assert!(validate_agent_request("   ", &[]).is_err());
        assert!(validate_agent_request("", &[]).is_err());
        assert!(validate_agent_request(&"x".repeat(65), &[]).is_err());
    }

    #[test]
    fn validate_accepts_read_set() {
        let caps = agent_grantable_capabilities();
        assert!(!caps.is_empty());
        assert!(validate_agent_request("Claude", &caps).is_ok());
    }
}
