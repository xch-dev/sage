use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
        request::Parts,
    },
    middleware::{self, Next},
    response::Response,
};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig,
        Tool,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle, time::timeout};
use tracing::{error, warn};

use crate::{
    AgentToken, AgentTool, AgentToolExecutor, AppsDb, SageAgent, find_agent_tool, tools_for_agent,
    unix_timestamp_ms,
};

const INSTRUCTIONS: &str = "Tools act on the wallet that is currently open in Sage. \
     Only the tools this agent has been granted are listed.";
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const LAST_USED_RESOLUTION_MS: i64 = 60_000;

#[derive(Debug, Clone)]
pub(crate) struct AuthenticatedAgent(pub SageAgent);

#[derive(Debug)]
pub(crate) struct AgentServerHandle {
    port: u16,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

impl AgentServerHandle {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn stop(self) {
        let _ = self.shutdown.send(());
        let mut task = self.task;
        match timeout(SHUTDOWN_TIMEOUT, &mut task).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => error!("agent server task failed: {err}"),
            Err(_) => {
                warn!("agent server did not shut down in time, aborting it");
                task.abort();
                let _ = task.await;
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn agent_router(
    db: AppsDb,
    executor: Arc<dyn AgentToolExecutor>,
    version: String,
) -> Router {
    build_router(db, executor, version, StreamableHttpServerConfig::default())
}

pub(crate) async fn start_agent_server(
    db: AppsDb,
    executor: Arc<dyn AgentToolExecutor>,
    version: String,
    port: u16,
) -> anyhow::Result<AgentServerHandle> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
    let port = listener.local_addr()?.port();

    let config = StreamableHttpServerConfig::default();
    let cancel = config.cancellation_token.clone();
    let router = build_router(db, executor, version, config);
    let (shutdown, shutdown_rx) = oneshot::channel::<()>();

    let task = tokio::spawn(async move {
        let signal = async move {
            let _ = shutdown_rx.await;
            cancel.cancel();
        };
        if let Err(err) = axum::serve(listener, router)
            .with_graceful_shutdown(signal)
            .await
        {
            error!("agent server stopped with an error: {err}");
        }
    });

    Ok(AgentServerHandle {
        port,
        shutdown,
        task,
    })
}

fn build_router(
    db: AppsDb,
    executor: Arc<dyn AgentToolExecutor>,
    version: String,
    config: StreamableHttpServerConfig,
) -> Router {
    let handler = AgentMcpHandler { executor, version };
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        LocalSessionManager::default().into(),
        config,
    );

    // rmcp's tower service mounts directly on axum 0.7 via nest_service.
    Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(db, authenticate))
}

async fn authenticate(State(db): State<AppsDb>, mut request: Request, next: Next) -> Response {
    let Some(token) = bearer_token(request.headers()) else {
        return unauthorized();
    };

    let agent = match db.find_agent_by_token_hash(&token.hash()).await {
        Ok(Some(agent)) => agent,
        Ok(None) => return unauthorized(),
        Err(err) => {
            error!("agent token lookup failed: {err:#}");
            return unavailable();
        }
    };

    let now = unix_timestamp_ms();
    if last_used_is_stale(agent.last_used_at_ms, now)
        && let Err(err) = db.touch_agent_last_used(&agent.id, now).await
    {
        warn!("failed to record agent {} last use: {err:#}", agent.id);
    }

    request.extensions_mut().insert(AuthenticatedAgent(agent));
    next.run(request).await
}

fn bearer_token(headers: &HeaderMap) -> Option<AgentToken> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?.trim();
    let (scheme, token) = value.split_once(char::is_whitespace)?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    AgentToken::parse(token.trim())
}

/// Last use is shown to the minute, so most requests skip the write.
fn last_used_is_stale(last_used_at_ms: Option<i64>, now_ms: i64) -> bool {
    last_used_at_ms.is_none_or(|last| now_ms.saturating_sub(last) >= LAST_USED_RESOLUTION_MS)
}

/// A failed lookup says nothing about the token, so it must not read as a revoked one.
fn unavailable() -> Response {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
    response
}

fn unauthorized() -> Response {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response
        .headers_mut()
        .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

#[derive(Clone)]
struct AgentMcpHandler {
    executor: Arc<dyn AgentToolExecutor>,
    version: String,
}

fn authenticated_agent(ctx: &RequestContext<RoleServer>) -> Result<SageAgent, ErrorData> {
    ctx.extensions
        .get::<Parts>()
        .and_then(|parts| parts.extensions.get::<AuthenticatedAgent>())
        .map(|agent| agent.0.clone())
        .ok_or_else(|| ErrorData::internal_error("request is not authenticated", None))
}

fn to_mcp_tool(tool: &AgentTool) -> Tool {
    let schema = match (tool.input_schema)() {
        Value::Object(schema) => schema,
        _ => JsonObject::new(),
    };
    Tool::new(tool.name, tool.description, schema)
}

fn tool_error(text: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(text)])
}

fn tool_success(value: Value) -> CallToolResult {
    let mut result = CallToolResult::success(vec![ContentBlock::text(value.to_string())]);
    // MCP types structuredContent as an object, so wrap anything else.
    result.structured_content = Some(match value {
        Value::Object(_) => value,
        other => json!({ "result": other }),
    });
    result
}

impl ServerHandler for AgentMcpHandler {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sage", self.version.clone()))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let agent = authenticated_agent(&ctx)?;
        let tools = tools_for_agent(&agent)
            .into_iter()
            .map(to_mcp_tool)
            .collect();
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let agent = authenticated_agent(&ctx)?;

        let Some(tool) = find_agent_tool(&request.name) else {
            return Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            ));
        };

        if !agent.capabilities.contains(&tool.capability) {
            return Ok(tool_error(format!(
                "permission_denied: agent is not granted access to {}",
                tool.name
            ))
            .into());
        }

        let arguments = Value::Object(request.arguments.unwrap_or_default());

        let result = match self.executor.call(&agent, tool, arguments).await {
            Ok(value) => tool_success(value),
            Err(err) => tool_error(format!("{}: {}", err.code, err.message)),
        };

        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::Duration};

    use async_trait::async_trait;
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{
            Request, Response, StatusCode,
            header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST},
        },
    };
    use futures::StreamExt;
    use serde_json::{Value, json};
    use tempfile::{TempDir, tempdir};
    use tokio::io::AsyncWriteExt;
    use tower::ServiceExt;

    use super::*;
    use crate::{BridgeMethodHandleError, SageAppWalletScope, UserBridgeCapability};

    const PROTOCOL_VERSION: &str = "2025-06-18";
    const VERSION: &str = "9.9.9-test";

    struct FakeExecutor;

    #[async_trait]
    impl AgentToolExecutor for FakeExecutor {
        async fn call(
            &self,
            _agent: &SageAgent,
            tool: &AgentTool,
            arguments: Value,
        ) -> Result<Value, BridgeMethodHandleError> {
            if tool.name == "get_sync_status" {
                return Err(BridgeMethodHandleError::new(
                    "no_wallet",
                    "no wallet is open",
                ));
            }
            if tool.name == "get_asset_coins" {
                return Ok(json!([{ "coin": 1 }, { "coin": 2 }]));
            }
            Ok(json!({ "tool": tool.name, "args": arguments }))
        }
    }

    struct Fixture {
        _dir: TempDir,
        db: AppsDb,
        router: Router,
        agent: SageAgent,
        token: AgentToken,
    }

    impl Fixture {
        fn bearer(&self) -> String {
            format!("Bearer {}", self.token.expose())
        }
    }

    async fn fixture(capabilities: &[UserBridgeCapability]) -> Fixture {
        let dir = tempdir().unwrap();
        let db = AppsDb::initialize(dir.path()).await.unwrap();
        let agent = SageAgent {
            id: uuid::Uuid::new_v4().to_string(),
            name: "agent".to_string(),
            capabilities: capabilities.iter().copied().collect(),
            wallet_scope: SageAppWalletScope::AllWallets,
            created_at_ms: 1,
            updated_at_ms: 1,
            last_used_at_ms: None,
        };
        let token = AgentToken::generate();
        db.insert_agent(&agent, &token.hash()).await.unwrap();
        let router = agent_router(db.clone(), Arc::new(FakeExecutor), VERSION.to_string());
        Fixture {
            _dir: dir,
            db,
            router,
            agent,
            token,
        }
    }

    fn unknown_token() -> String {
        format!("sage_agent_{}", "ab".repeat(32))
    }

    fn build_request(
        auth: Option<&str>,
        host: &str,
        session: Option<&str>,
        body: &Value,
    ) -> Request<Body> {
        let mut builder = Request::post("/mcp")
            .header(HOST, host)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream");
        if let Some(auth) = auth {
            builder = builder.header(AUTHORIZATION, auth);
        }
        if let Some(session) = session {
            builder = builder
                .header("mcp-session-id", session)
                .header("mcp-protocol-version", PROTOCOL_VERSION);
        }
        builder.body(Body::from(body.to_string())).unwrap()
    }

    async fn send(router: &Router, request: Request<Body>) -> Response<Body> {
        router.clone().oneshot(request).await.unwrap()
    }

    fn initialize_body() -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }
        })
    }

    async fn post_initialize(router: &Router, auth: Option<&str>) -> Response<Body> {
        send(
            router,
            build_request(auth, "127.0.0.1", None, &initialize_body()),
        )
        .await
    }

    async fn initialize(router: &Router, auth: &str) -> String {
        let response = post_initialize(router, Some(auth)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let session = response
            .headers()
            .get("mcp-session-id")
            .expect("session id header")
            .to_str()
            .unwrap()
            .to_string();
        let message = rpc_body(response).await;
        assert!(message.get("result").is_some(), "{message}");

        let initialized = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        let response = send(
            router,
            build_request(Some(auth), "127.0.0.1", Some(&session), &initialized),
        )
        .await;
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        session
    }

    async fn rpc(
        router: &Router,
        auth: &str,
        session: &str,
        method: &str,
        params: Value,
    ) -> Response<Body> {
        let body = json!({ "jsonrpc": "2.0", "id": 2, "method": method, "params": params });
        send(
            router,
            build_request(Some(auth), "127.0.0.1", Some(session), &body),
        )
        .await
    }

    async fn rpc_ok(
        router: &Router,
        auth: &str,
        session: &str,
        method: &str,
        params: Value,
    ) -> Value {
        let response = rpc(router, auth, session, method, params).await;
        assert_eq!(response.status(), StatusCode::OK);
        rpc_body(response).await
    }

    fn find_message(text: &str) -> Option<Value> {
        text.lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .filter_map(|data| serde_json::from_str::<Value>(data.trim()).ok())
            .find(|value| value.get("id").is_some())
    }

    async fn rpc_body(response: Response<Body>) -> Value {
        let is_sse = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream"));

        if !is_sse {
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            return serde_json::from_slice(&bytes).unwrap();
        }

        let mut stream = response.into_body().into_data_stream();
        let mut text = String::new();
        let read = async {
            while let Some(chunk) = stream.next().await {
                text.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
                if let Some(message) = find_message(&text) {
                    return message;
                }
            }
            panic!("SSE stream ended without a JSON-RPC message: {text}");
        };
        tokio::time::timeout(Duration::from_secs(5), read)
            .await
            .expect("timed out waiting for SSE message")
    }

    fn tool_names(list: &Value) -> Vec<String> {
        list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect()
    }

    #[tokio::test]
    async fn auth_header_variants() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;

        let unknown = format!("Bearer {}", unknown_token());
        for auth in [
            None,
            Some("Basic x"),
            Some("Bearer"),
            Some("Bearer not_a_token"),
            Some(unknown.as_str()),
        ] {
            let response = post_initialize(&f.router, auth).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{auth:?}");
            assert_eq!(
                response.headers().get("www-authenticate").unwrap(),
                "Bearer"
            );
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert!(body.is_empty());
        }

        for auth in [
            format!("bearer {}", f.token.expose()),
            format!("Bearer   {}  ", f.token.expose()),
        ] {
            let response = post_initialize(&f.router, Some(&auth)).await;
            assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn initialize_advertises_sage_server() {
        let f = fixture(&[]).await;
        let response = post_initialize(&f.router, Some(&f.bearer())).await;
        let message = rpc_body(response).await;
        assert_eq!(message["result"]["serverInfo"]["name"], "sage");
        assert_eq!(message["result"]["serverInfo"]["version"], VERSION);
        assert!(message["result"]["capabilities"]["tools"].is_object());
        assert!(message["result"]["instructions"].is_string());
    }

    #[tokio::test]
    async fn list_tools_only_shows_granted() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        let list = rpc_ok(&f.router, &auth, &session, "tools/list", json!({})).await;
        assert_eq!(tool_names(&list), vec!["get_wallet".to_string()]);
    }

    #[tokio::test]
    async fn call_ungranted_tool_is_tool_error() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        let message = rpc_ok(
            &f.router,
            &auth,
            &session,
            "tools/call",
            json!({ "name": "get_version", "arguments": {} }),
        )
        .await;
        assert_eq!(message["result"]["isError"], true);
        let text = message["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.starts_with("permission_denied"), "{text}");
    }

    #[tokio::test]
    async fn call_unknown_tool_is_protocol_error() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        let message = rpc_ok(
            &f.router,
            &auth,
            &session,
            "tools/call",
            json!({ "name": "no_such_tool", "arguments": {} }),
        )
        .await;
        assert!(message.get("result").is_none(), "{message}");
        assert_eq!(message["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn call_granted_tool_returns_executor_value() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;
        let expected = json!({ "tool": "get_wallet", "args": {} });

        for params in [
            json!({ "name": "get_wallet" }),
            json!({ "name": "get_wallet", "arguments": null }),
            json!({ "name": "get_wallet", "arguments": {} }),
        ] {
            let message = rpc_ok(&f.router, &auth, &session, "tools/call", params).await;
            assert_eq!(
                message["result"]["structuredContent"], expected,
                "{message}"
            );
            assert_ne!(message["result"]["isError"], true);
            let text = message["result"]["content"][0]["text"].as_str().unwrap();
            assert_eq!(serde_json::from_str::<Value>(text).unwrap(), expected);
        }
    }

    #[tokio::test]
    async fn non_object_result_is_wrapped_in_structured_content() {
        let f = fixture(&[UserBridgeCapability::WalletGetAssetCoins]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;
        let expected = json!([{ "coin": 1 }, { "coin": 2 }]);

        let message = rpc_ok(
            &f.router,
            &auth,
            &session,
            "tools/call",
            json!({ "name": "get_asset_coins", "arguments": {} }),
        )
        .await;
        assert!(
            message["result"]["structuredContent"].is_object(),
            "{message}"
        );
        assert_eq!(
            message["result"]["structuredContent"],
            json!({ "result": expected })
        );
        let text = message["result"]["content"][0]["text"].as_str().unwrap();
        assert_eq!(serde_json::from_str::<Value>(text).unwrap(), expected);
    }

    #[tokio::test]
    async fn get_and_delete_without_auth_are_401() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        for method in ["GET", "DELETE"] {
            let request = Request::builder()
                .method(method)
                .uri("/mcp")
                .header(HOST, "127.0.0.1")
                .header(ACCEPT, "text/event-stream")
                .body(Body::empty())
                .unwrap();
            let response = send(&f.router, request).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{method}");
        }
    }

    #[tokio::test]
    async fn executor_error_is_tool_error() {
        let f = fixture(&[UserBridgeCapability::WalletGetSyncStatus]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        let message = rpc_ok(
            &f.router,
            &auth,
            &session,
            "tools/call",
            json!({ "name": "get_sync_status", "arguments": {} }),
        )
        .await;
        assert_eq!(message["result"]["isError"], true);
        assert_eq!(
            message["result"]["content"][0]["text"],
            "no_wallet: no wallet is open"
        );
    }

    #[tokio::test]
    async fn revoked_agent_is_rejected_mid_session() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        assert!(f.db.delete_agent(&f.agent.id).await.unwrap());

        let response = rpc(&f.router, &auth, &session, "tools/list", json!({})).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn capability_removed_mid_session_hides_tool() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        assert!(
            f.db.update_agent(
                &f.agent.id,
                &f.agent.name,
                &BTreeSet::new(),
                &f.agent.wallet_scope,
            )
            .await
            .unwrap()
        );

        let list = rpc_ok(&f.router, &auth, &session, "tools/list", json!({})).await;
        assert!(tool_names(&list).is_empty());

        let message = rpc_ok(
            &f.router,
            &auth,
            &session,
            "tools/call",
            json!({ "name": "get_wallet" }),
        )
        .await;
        assert_eq!(message["result"]["isError"], true);
    }

    #[tokio::test]
    async fn rotated_token_rejects_old() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let old = f.bearer();
        let session = initialize(&f.router, &old).await;

        let rotated = AgentToken::generate();
        assert!(
            f.db.set_agent_token_hash(&f.agent.id, &rotated.hash())
                .await
                .unwrap()
        );

        let response = rpc(&f.router, &old, &session, "tools/list", json!({})).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let new = format!("Bearer {}", rotated.expose());
        let list = rpc_ok(&f.router, &new, &session, "tools/list", json!({})).await;
        assert_eq!(tool_names(&list), vec!["get_wallet".to_string()]);
    }

    #[tokio::test]
    async fn non_local_host_header_rejected() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let response = send(
            &f.router,
            build_request(Some(&f.bearer()), "evil.example", None, &initialize_body()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn host_header_with_port_is_checked() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let auth = f.bearer();

        for host in ["evil.example:9258", "127.0.0.1.evil.example"] {
            let response = send(
                &f.router,
                build_request(Some(&auth), host, None, &initialize_body()),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{host}");
        }

        for host in ["127.0.0.1:9258", "localhost:9258", "[::1]:9258"] {
            let response = send(
                &f.router,
                build_request(Some(&auth), host, None, &initialize_body()),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK, "{host}");
        }
    }

    #[tokio::test]
    async fn secret_and_signing_grants_expose_no_tools() {
        // Such grants can't be made through Sage, only by editing the database.
        let f = fixture(&[
            UserBridgeCapability::WalletGetSecretKey,
            UserBridgeCapability::WalletSignCoinSpends,
            UserBridgeCapability::WalletSendXch,
        ])
        .await;
        let auth = f.bearer();
        let session = initialize(&f.router, &auth).await;

        let list = rpc_ok(&f.router, &auth, &session, "tools/list", json!({})).await;
        assert!(tool_names(&list).is_empty(), "{list}");

        for name in ["get_secret_key", "sign_coin_spends", "send_xch"] {
            let message = rpc_ok(
                &f.router,
                &auth,
                &session,
                "tools/call",
                json!({ "name": name, "arguments": {} }),
            )
            .await;
            assert_eq!(message["error"]["code"], -32602, "{name}: {message}");
        }
    }

    #[tokio::test]
    async fn lookup_failure_is_not_reported_as_unauthorized() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        f.db.pool().close().await;

        let response = post_initialize(&f.router, Some(&f.bearer())).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn last_used_is_rewritten_once_a_minute() {
        assert!(last_used_is_stale(None, 0));
        assert!(!last_used_is_stale(
            Some(1_000),
            1_000 + LAST_USED_RESOLUTION_MS - 1
        ));
        assert!(last_used_is_stale(
            Some(1_000),
            1_000 + LAST_USED_RESOLUTION_MS
        ));
    }

    #[tokio::test]
    async fn last_used_is_recorded() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let before = f.db.get_agent(&f.agent.id).await.unwrap().unwrap();
        assert!(before.last_used_at_ms.is_none());

        post_initialize(&f.router, Some(&f.bearer())).await;

        let after = f.db.get_agent(&f.agent.id).await.unwrap().unwrap();
        assert!(after.last_used_at_ms.is_some());
    }

    #[tokio::test]
    async fn token_never_echoed() {
        let f = fixture(&[]).await;
        let token = unknown_token();
        let response = post_initialize(&f.router, Some(&format!("Bearer {token}"))).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        for (name, value) in response.headers() {
            assert!(!name.as_str().contains(&token));
            assert!(!String::from_utf8_lossy(value.as_bytes()).contains(&token));
        }
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(!String::from_utf8_lossy(&body).contains(&token));
    }

    #[tokio::test]
    async fn start_binds_loopback_and_stops() {
        let f = fixture(&[]).await;
        let handle =
            start_agent_server(f.db.clone(), Arc::new(FakeExecutor), VERSION.to_string(), 0)
                .await
                .unwrap();
        assert_ne!(handle.port(), 0);

        let occupied = start_agent_server(
            f.db.clone(),
            Arc::new(FakeExecutor),
            VERSION.to_string(),
            handle.port(),
        )
        .await;
        assert!(occupied.is_err());

        tokio::time::timeout(Duration::from_secs(5), handle.stop())
            .await
            .expect("server stops");
    }

    #[tokio::test]
    async fn start_fails_on_occupied_port() {
        let f = fixture(&[]).await;
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();

        let result = start_agent_server(
            f.db.clone(),
            Arc::new(FakeExecutor),
            VERSION.to_string(),
            port,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn start_then_stop_frees_port() {
        let f = fixture(&[]).await;
        let handle =
            start_agent_server(f.db.clone(), Arc::new(FakeExecutor), VERSION.to_string(), 0)
                .await
                .unwrap();
        let port = handle.port();

        tokio::time::timeout(Duration::from_secs(5), handle.stop())
            .await
            .expect("server stops");

        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).expect("port is free");
    }

    #[tokio::test]
    async fn stop_with_open_sse_stream_is_bounded() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let handle =
            start_agent_server(f.db.clone(), Arc::new(FakeExecutor), VERSION.to_string(), 0)
                .await
                .unwrap();
        let port = handle.port();
        let url = format!("http://127.0.0.1:{port}/mcp");
        let auth = f.bearer();
        let client = reqwest::Client::new();

        let response = client
            .post(&url)
            .header(AUTHORIZATION, &auth)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream")
            .body(initialize_body().to_string())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let session = response.headers()["mcp-session-id"]
            .to_str()
            .unwrap()
            .to_string();
        response.bytes().await.unwrap();

        let response = client
            .post(&url)
            .header(AUTHORIZATION, &auth)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", PROTOCOL_VERSION)
            .body(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let stream = client
            .get(&url)
            .header(AUTHORIZATION, &auth)
            .header(ACCEPT, "text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", PROTOCOL_VERSION)
            .send()
            .await
            .unwrap();
        assert_eq!(stream.status(), StatusCode::OK);

        tokio::time::timeout(SHUTDOWN_TIMEOUT + Duration::from_secs(2), handle.stop())
            .await
            .expect("stop is bounded by the shutdown timeout");
        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).expect("port is free");
        drop(stream);
    }

    #[tokio::test]
    async fn stop_with_stalled_request_is_bounded() {
        let f = fixture(&[UserBridgeCapability::WalletGetWallet]).await;
        let handle =
            start_agent_server(f.db.clone(), Arc::new(FakeExecutor), VERSION.to_string(), 0)
                .await
                .unwrap();
        let port = handle.port();

        // An authenticated request whose body never finishes keeps the connection in flight.
        let mut socket = tokio::net::TcpStream::connect((Ipv4Addr::LOCALHOST, port))
            .await
            .unwrap();
        let head = format!(
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: {}\r\n\
             Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n\
             Content-Length: 1000\r\n\r\n{{",
            f.bearer()
        );
        socket.write_all(head.as_bytes()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;

        tokio::time::timeout(SHUTDOWN_TIMEOUT + Duration::from_secs(2), handle.stop())
            .await
            .expect("stop is bounded by the shutdown timeout");
        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).expect("port is free");
        drop(socket);
    }
}
