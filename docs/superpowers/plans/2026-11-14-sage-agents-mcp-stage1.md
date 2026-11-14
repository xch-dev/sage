# Sage Agents over MCP — Stage 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A localhost MCP server inside the Sage desktop app. Agents authenticate with per-agent bearer tokens and can call a curated set of read-only wallet tools, gated by the existing Sage-apps capability and wallet-scope model.

**Architecture:** Agents become a new kind of caller in `crates/sage-apps`. They're stored in the apps database, carry `UserBridgeCapability` grants and a `SageAppWalletScope`, and their tool calls run the existing user `BridgeMethod`s through a new agent path in the bridge pipeline. An `rmcp` streamable-HTTP service on `127.0.0.1` sits behind an axum bearer-auth layer that resolves the agent on every request. Settings → Agents in the GUI manages the server and the agents through new `apps_*` Tauri commands.

**Tech Stack:** Rust (2024 edition), `rmcp` 3.10 (`server`, `transport-streamable-http-server`), axum 0.7, sqlx/SQLite, Tauri 2, React + Lingui.

**Spec:** `docs/superpowers/specs/2026-11-14-sage-agents-mcp-design.md` (stage 1 only).

## Global Constraints

- The user makes all git commits. No task commits. Each task ends with its verification step instead.
- Agents are desktop-only. Everything new lives in `sage-apps` (already `cfg(not(mobile))` at its call sites in `src-tauri`).
- Listen on `127.0.0.1` only. Default port **9258**. Plain HTTP. Server **off by default**.
- Token format: `sage_agent_` + lowercase hex of 32 bytes from an OS CSPRNG (`rand::rngs::OsRng`). Store only `hex(sha256(token))`. Look tokens up by hash; plaintext is never stored or compared.
- Never expose or grant `wallet.get_secret_key`, or any capability whose definition has `accesses_sensitive_secret`.
- Signing capabilities are not grantable to agents in stage 1.
- Every new Tauri command is added to `src-tauri/permissions/main-host.toml`. Otherwise it's silently denied.
- Wrap new UI strings with Lingui (`t`, `<Trans>`). Do **not** run `pnpm extract` or touch `src/locales/**/*.po`.
- Imports and modules follow `.claude/rules/imports.md`.
- Rustdoc comments stay to one line.
- Never hold the `Sage` mutex (`AppState`) across an `.await` that isn't the wallet call itself.
- CI gates are `cargo clippy --workspace --all-features --all-targets`, `cargo test --workspace --all-features`, `cargo fmt --all -- --check`, `pnpm lint`, `pnpm prettier:check` and `tsc -b`. All must pass at the end.
- macOS builds need `export SDKROOT="$(xcrun --show-sdk-path)"` before cargo.

## Review Focus

1. **Grants change during an open MCP session.** The user revokes an agent or removes a capability while the client stays connected. The very next request must fail with 401, or not list or allow the tool. The agent is resolved per HTTP request and never cached in the session. _(Task 5 test `revoked_agent_is_rejected_mid_session`, `capability_removed_mid_session_hides_tool`.)_
2. **Wallet switched to one outside the agent's scope.** Calls must fail with `wallet_not_in_scope`, not read the new wallet. _(Task 3 test `call_refused_when_current_wallet_out_of_scope`.)_
3. **No wallet logged in.** A tool call returns a `no_wallet` tool error, not a panic or 500. _(Task 3 test `call_refused_without_current_wallet`.)_
4. **Malformed `Authorization` headers.** Missing header, `Basic …`, lowercase `bearer`, extra whitespace, a token without the `sage_agent_` prefix or an unknown token. All give 401, and a lowercase scheme with a valid token is accepted. _(Task 5 test `auth_header_variants`.)_
5. **Port already in use when starting the server.** Start returns an error the UI shows, and `is_running` stays false. _(Task 6 test `start_fails_on_occupied_port`.)_

---

## File Structure

**Create (all under `crates/sage-apps/src/`):**

- `db/migrations/0005_agents.sql` — the `sage_agents` table.
- `db/agents.rs` — `AppsDb` CRUD for agents, plus agent-server settings.
- `types/agent.rs` — `SageAgent`, `AgentToken`, token generation and hashing.
- `agents.rs` + `agents/` — the agent feature module:
  - `agents/tools.rs` — the curated tool catalog and pure authorization.
  - `agents/executor.rs` — `AgentToolExecutor` trait, plus a Tauri implementation that runs bridge methods.
  - `agents/server.rs` — the rmcp `ServerHandler`, the axum router with bearer auth, and start/stop.
  - `agents/commands.rs` — `apps_*` Tauri commands for the server and agent management.
- `src/pages/settings/AgentSettings.tsx` (frontend, repo root `src/`) — the Settings → Agents section.

**Modify:**

- `crates/sage-apps/Cargo.toml` — add `rmcp`, `rand`, `axum`, `http`, and `tower` (dev).
- `crates/sage-apps/src/db.rs`, `types.rs` (or `types/mod` equivalent), `lib.rs` — module wiring and re-exports.
- `crates/sage-apps/src/bridge/methods/shared.rs`, `bridge/types.rs`, `bridge/bridge_request.rs` and every `ctx.app` call site — the `BridgeCaller` refactor.
- `crates/sage-apps/src/host.rs` — `AppsHostState` holds the agent-server handle.
- `src-tauri/src/lib.rs` — register commands. `src-tauri/src/commands.rs` — autostart in `initialize`.
- `src-tauri/permissions/main-host.toml` — the ACL entries.
- `src/bindings.ts` — regenerated with `pnpm generate:bindings`.
- `src/pages/Settings.tsx` — render `<AgentSettings />` in the Advanced tab after `<RpcSettings />`.

---

### Task 1: Agent storage and tokens

**Files:**

- Create: `crates/sage-apps/src/db/migrations/0005_agents.sql`, `crates/sage-apps/src/db/agents.rs`, `crates/sage-apps/src/types/agent.rs`
- Modify: `crates/sage-apps/src/db.rs`, the `types` module file, `crates/sage-apps/Cargo.toml` (add `rand = { workspace = true }`)
- Test: `#[cfg(test)] mod tests` in `db/agents.rs` and `types/agent.rs`

**Interfaces:**

- Produces:

  ```rust
  // types/agent.rs
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
  #[serde(rename_all = "camelCase")]
  pub struct SageAgent {
      pub id: String,                                  // uuid v4
      pub name: String,
      pub capabilities: BTreeSet<UserBridgeCapability>,
      pub wallet_scope: SageAppWalletScope,
      pub created_at_ms: i64,
      pub updated_at_ms: i64,
      pub last_used_at_ms: Option<i64>,
  }
  pub struct AgentToken(String);                       // the plaintext; never Debug-printed
  impl AgentToken {
      pub fn generate() -> Self;
      pub fn parse(raw: &str) -> Option<Self>;         // None unless "sage_agent_" + 64 lowercase hex
      pub fn hash(&self) -> String;                    // hex(sha256(token))
      pub fn expose(&self) -> &str;
  }
  impl SageAgent { pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool; }
  // db/agents.rs (impl AppsDb)
  pub async fn insert_agent(&self, agent: &SageAgent, token_hash: &str) -> Result<()>;
  pub async fn list_agents(&self) -> Result<Vec<SageAgent>>;            // ordered by name
  pub async fn get_agent(&self, id: &str) -> Result<Option<SageAgent>>;
  pub async fn find_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<SageAgent>>;
  pub async fn update_agent(&self, id: &str, name: &str, capabilities: &BTreeSet<UserBridgeCapability>, wallet_scope: &SageAppWalletScope) -> Result<bool>; // false if missing
  pub async fn set_agent_token_hash(&self, id: &str, token_hash: &str) -> Result<bool>;
  pub async fn delete_agent(&self, id: &str) -> Result<bool>;
  pub async fn touch_agent_last_used(&self, id: &str, at_ms: i64) -> Result<()>;
  pub async fn get_agent_server_settings(&self) -> Result<AgentServerSettings>;
  pub async fn set_agent_server_settings(&self, settings: AgentServerSettings) -> Result<AgentServerSettings>;
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
  #[serde(rename_all = "camelCase")]
  pub struct AgentServerSettings { pub enabled: bool, pub port: u16 }   // Default: { false, 9258 }
  ```

- [ ] **Step 1: Write the migration**

  `sage_agents(agent_id TEXT PRIMARY KEY, name TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE, capabilities_json TEXT NOT NULL, wallet_scope_json TEXT NOT NULL, created_at_ms INTEGER NOT NULL, updated_at_ms INTEGER NOT NULL, last_used_at_ms INTEGER)`. The server settings are stored in `sage_app_settings` under the keys `agent_server_enabled` and `agent_server_port`, following `db/settings.rs`.

- [ ] **Step 2: Write failing tests**

  In `types/agent.rs`:
  - `generated_token_round_trips_through_parse`: `AgentToken::parse(t.expose()).unwrap().hash() == t.hash()`. The token starts with `"sage_agent_"` and is `11 + 64` chars long.
  - `parse_rejects_malformed`: `None` for `""`, `"sage_agent_"`, `"sage_agent_XYZ…"` (uppercase or non-hex), a 63-hex body, and `"other_" + 64 hex`.
  - `two_tokens_differ`.

  In `db/agents.rs` (`AppsDb::initialize(tempdir)`, as in `lifecycle/registry.rs` tests):
  - `insert_then_find_by_hash`.
  - `find_by_unknown_hash_is_none`.
  - `update_changes_grants_and_scope`.
  - `rotate_invalidates_old_hash`: after `set_agent_token_hash`, finding by the old hash gives `None` and the new hash gives `Some`.
  - `delete_removes_agent`.
  - `list_is_sorted_by_name`.
  - `server_settings_default_off_9258`, then a set/get round-trip to `{ enabled: true, port: 9300 }`.

- [ ] **Step 3: Run the tests and verify they fail**

  Run: `cargo test -p sage-apps agent`. Expected: they fail to compile because the items are missing.

- [ ] **Step 4: Implement the types and DB methods**

  Serialize capabilities and scope with `serde_json`, like `granted_permissions_json` and `wallet_scope_json` in `db/app.rs`.

- [ ] **Step 5: Run the tests and verify they pass**

  Run: `cargo test -p sage-apps agent`. Expected: all pass.

---

### Task 2: `BridgeCaller` — let bridge methods run for agents

Today `BridgeContext { app: &SharedSageApp }`. Make the caller an app or an agent, without changing app behavior.

**Files:**

- Modify: `crates/sage-apps/src/bridge/methods/shared.rs` (`BridgeContext`), every `ctx.app` call site (`grep -rn "ctx\.app" crates/sage-apps/src`, about 20 sites across `bridge/methods/user/{app,bridge,wallet}*` and `bridge/methods/system/runtime_manager/*`), `bridge/bridge_request.rs` (constructing contexts)
- Test: `#[cfg(test)]` in `bridge/methods/shared.rs`

**Interfaces:**

- Consumes: `SageAgent` (Task 1).
- Produces:

  ```rust
  pub(crate) enum BridgeCaller<'a> { App(&'a SharedSageApp), Agent(&'a SageAgent) }
  pub(crate) struct BridgeContext<'a> { pub caller: BridgeCaller<'a> }
  impl BridgeContext<'_> {
      pub fn app(&self) -> Result<&SharedSageApp, BridgeMethodHandleError>; // Agent => code "app_only"
      pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool;
  }
  ```

  `BridgeMethodHandleError::new("app_only", "This method is only available to Sage apps")`.

- [ ] **Step 1: Write failing tests**
  - `agent_context_has_no_app`: `ctx.app()` is `Err` with `code == "app_only"`.
  - `agent_context_scope_uses_agent_scope`: an agent with `SelectedWallets { fingerprints: vec![7] }` → `is_wallet_in_scope(7)` is true and `is_wallet_in_scope(8)` is false.

- [ ] **Step 2: Run and verify they fail**

  Run: `cargo test -p sage-apps bridge_caller`. Expected: compile failure.

- [ ] **Step 3: Refactor**

  Add `BridgeCaller`. App-only methods use `ctx.app()?` in place of `ctx.app`. `require_scoped_fingerprint` in `bridge/methods/user/wallet.rs` uses `ctx.is_wallet_in_scope`. The pipeline builds `BridgeContext { caller: BridgeCaller::App(&origin.app) }`.

- [ ] **Step 4: Verify that the tests and existing behavior pass**

  Run: `cargo test -p sage-apps` and `cargo clippy -p sage-apps --all-targets --all-features`. Expected: all pass with no new warnings.

---

### Task 3: Tool catalog and agent authorization

**Files:**

- Create: `crates/sage-apps/src/agents.rs`, `crates/sage-apps/src/agents/tools.rs`
- Test: `#[cfg(test)]` in `agents/tools.rs`

**Interfaces:**

- Consumes: `SageAgent` (Task 1). `UserBridgeCapability` and `get_user_capability_definition` (existing).
- Produces:
  ```rust
  pub(crate) struct AgentTool {
      pub name: &'static str,             // MCP tool name
      pub bridge_method: &'static str,    // existing bridge method name
      pub capability: UserBridgeCapability,
      pub description: &'static str,
      pub input_schema: fn() -> serde_json::Value, // JSON Schema object
  }
  pub(crate) fn agent_tools() -> &'static [AgentTool];
  pub(crate) fn find_agent_tool(name: &str) -> Option<&'static AgentTool>;
  pub(crate) fn agent_grantable_capabilities() -> Vec<UserBridgeCapability>; // exactly the catalog's capabilities, deduped, sorted
  pub(crate) fn tools_for_agent(agent: &SageAgent) -> Vec<&'static AgentTool>;
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub(crate) struct AgentCallError { pub code: &'static str, pub message: String }
  pub(crate) fn authorize_agent_call(agent: &SageAgent, tool: &AgentTool, current_fingerprint: Option<u32>) -> Result<(), AgentCallError>;
  // codes: "permission_denied", "no_wallet", "wallet_not_in_scope"
  ```

The catalog: tool name → bridge method → capability.

| `get_wallet`               | `wallet.getWallet`              | `WalletGetWallet`              |
| -------------------------- | ------------------------------- | ------------------------------ |
| `get_sync_status`          | `wallet.getSyncStatus`          | `WalletGetSyncStatus`          |
| `get_version`              | `wallet.getVersion`             | `WalletGetVersion`             |
| `get_asset_balance`        | `wallet.getAssetBalance`        | `WalletGetAssetBalance`        |
| `get_asset_coins`          | `wallet.getAssetCoins`          | `WalletGetAssetCoins`          |
| `get_coins`                | `wallet.getCoins`               | `WalletGetCoins`               |
| `get_coins_by_ids`         | `wallet.getCoinsByIds`          | `WalletGetCoinsByIds`          |
| `get_spendable_coin_count` | `wallet.getSpendableCoinCount`  | `WalletGetSpendableCoinCount`  |
| `get_transactions`         | `wallet.getTransactions`        | `WalletGetTransactions`        |
| `get_transaction`          | `wallet.getTransaction`         | `WalletGetTransaction`         |
| `get_pending_transactions` | `wallet.getPendingTransactions` | `WalletGetPendingTransactions` |
| `get_xch_usd_price`        | `wallet.getXchUsdPrice`         | `WalletGetXchUsdPrice`         |
| `check_address`            | `wallet.checkAddress`           | `WalletCheckAddress`           |

Input schemas are hand-written with `serde_json::json!`. Their properties mirror the param type each bridge method parses: read it in that method's file, e.g. `WalletGetAssetBalanceParams` (camelCase, `type` and `assetId`), and the `sage_api` request types for the `read_methods.rs` macros. Tools without params get `{"type":"object","properties":{},"additionalProperties":false}`.

- [ ] **Step 1: Write failing tests**
  - `catalog_matches_bridge_registry`: every `bridge_method` exists in `BridgeRegistry::new(BridgeRegistryKind::User)`, and that method's `capability()` equals `BridgeMethodCapability::user(tool.capability)`.
  - `catalog_has_no_secret_or_signing_capabilities`: for every tool, the definition's `accesses_sensitive_secret()` is false, and the bridge method's `requires_signing()` is false.
  - `catalog_is_read_only`: no `tool.capability` is one of `WalletSendXch`, `WalletSendXchAutoSubmit`, `WalletSignCoinSpends`, `WalletSignMessage`, `WalletSendTransaction` or `WalletGetSecretKey`.
  - `schemas_are_objects`: each `input_schema()` has `"type": "object"`.
  - `tools_filtered_by_grants`: an agent granted `{WalletGetWallet, WalletGetSyncStatus}` gets exactly those two tool names.
  - `call_refused_without_grant` → `permission_denied`.
  - `call_refused_without_current_wallet` (`None`) → `no_wallet`.
  - `call_refused_when_current_wallet_out_of_scope` (scope `[7]`, current `Some(8)`) → `wallet_not_in_scope`.
  - `call_allowed_in_scope_with_grant` → `Ok(())`.

- [ ] **Step 2: Run and verify they fail**

  Run: `cargo test -p sage-apps agents::tools`. Expected: compile failure.

- [ ] **Step 3: Implement**

  Check order in `authorize_agent_call`: grant, then wallet presence, then scope.

- [ ] **Step 4: Verify they pass**

  Run: `cargo test -p sage-apps agents::tools`. Expected: all pass.

---

### Task 4: Agent executor (runs bridge methods inside Tauri)

**Files:**

- Create: `crates/sage-apps/src/agents/executor.rs`
- Modify: `crates/sage-apps/src/bridge/bridge_request.rs` (add `process_agent`)

**Interfaces:**

- Consumes: `BridgeCaller::Agent` (Task 2). `AgentTool`, `authorize_agent_call` and `AgentCallError` (Task 3).
- Produces:
  ```rust
  #[async_trait]
  pub(crate) trait AgentToolExecutor: Send + Sync + 'static {
      async fn call(&self, agent: &SageAgent, tool: &AgentTool, arguments: serde_json::Value)
          -> Result<serde_json::Value, AgentCallError>;
  }
  pub(crate) struct TauriAgentExecutor { app_handle: tauri::AppHandle }
  impl TauriAgentExecutor { pub fn new(app_handle: tauri::AppHandle) -> Self; }
  // bridge_request.rs
  pub(crate) async fn process_agent(app_handle: &AppHandle, app_state: &State<'_, AppState>, agent: &SageAgent, request: &RustBridgeRequest)
      -> RustBridgeResponse;
  ```

`TauriAgentExecutor::call`:

1. Reads the current fingerprint from `AppState` (`config.global.fingerprint`), then releases the lock.
2. Calls `authorize_agent_call`.
3. Builds a `RustBridgeRequest { bridge_version: Some("v1"), id: uuid, method: tool.bridge_method, params_json: Some(arguments.to_string()) }`.
4. Calls `process_agent`.
5. Maps a success to the JSON value and an error to `AgentCallError { code, message }`.

`process_agent` mirrors `process_shared` for an agent:

1. `assert_method` on the User registry.
2. Refuses with `permission_denied` if `request.method` isn't a catalog `bridge_method`, as defence in depth.
3. Refuses with `watch_only` if `requires_signing()` and the wallet is watch-only.
4. Calls `prepare_approval` with `BridgeCaller::Agent`. If it returns `Some(_)`, refuses with code `approval_unsupported` ("Agent approvals arrive in a later release").
5. Otherwise runs `method.handle` and converts with `bridge_handle_result_to_response`.

- [ ] **Step 1: Implement**

  The pipeline needs a real `AppHandle`, which this crate can't build in unit tests (no Tauri mock runtime in the workspace). Coverage comes from Task 3's pure checks, Task 5's fake executor, and Task 8's manual run.

- [ ] **Step 2: Verify that it compiles and lints**

  Run: `cargo clippy -p sage-apps --all-targets --all-features`. Expected: no warnings.

---

### Task 5: MCP server with bearer auth

**Files:**

- Create: `crates/sage-apps/src/agents/server.rs`
- Modify: `crates/sage-apps/Cargo.toml`:
  - `rmcp = { version = "3.10", default-features = false, features = ["server", "macros", "transport-streamable-http-server"] }`
  - `axum = { workspace = true }`
  - `http = "1"`
  - dev-dependency `tower = { workspace = true, features = ["util"] }`
- Test: `#[cfg(test)]` in `agents/server.rs`

**Interfaces:**

- Consumes: `AppsDb` agent methods (Task 1). `tools_for_agent`, `find_agent_tool` and `authorize_agent_call` (Task 3). `AgentToolExecutor` (Task 4).
- Produces:
  ```rust
  #[derive(Debug, Clone)] pub(crate) struct AuthenticatedAgent(pub SageAgent); // inserted into request extensions
  pub(crate) fn agent_router(db: AppsDb, executor: Arc<dyn AgentToolExecutor>) -> axum::Router; // MCP at "/mcp"
  pub(crate) struct AgentServerHandle { /* JoinHandle + shutdown */ }
  impl AgentServerHandle { pub fn port(&self) -> u16; pub async fn stop(self); }
  pub(crate) async fn start_agent_server(db: AppsDb, executor: Arc<dyn AgentToolExecutor>, port: u16) -> anyhow::Result<AgentServerHandle>;
  ```
  (`AppsDb` must be `Clone`. Its `SqlitePool` is cheap to clone, so derive `Clone` if it's missing.)

Behavior:

- **Auth layer** (axum middleware on the router, applied before the rmcp service):
  - Parse `Authorization` as `<scheme> <token>`, case-insensitive scheme `bearer`, trimming surrounding whitespace.
  - Run `AgentToken::parse`, then `find_agent_by_token_hash`.
  - Any failure → `401` with an empty body and `WWW-Authenticate: Bearer`.
  - On success: `touch_agent_last_used` (log failures, don't block the request), then insert `AuthenticatedAgent`.
- **rmcp handler:** a struct implementing `ServerHandler` by hand (no `#[tool]` macros, since the tools are data-driven).
  - `get_info` advertises the tools capability, with server name `"sage"`, version `env!("CARGO_PKG_VERSION")`, and short instructions saying that tools act on the wallet currently open in Sage.
  - `list_tools` reads `AuthenticatedAgent` from `ctx.extensions.get::<http::request::Parts>()?.extensions` and returns `tools_for_agent`.
  - `call_tool`:
    - Unknown tool name → MCP invalid-params error.
    - Known tool but no grant → tool result with `is_error: true` and text `"permission_denied: …"`.
    - Executor `Err(AgentCallError)` → `is_error: true` and text `"{code}: {message}"`.
    - `Ok(value)` → text content of `serde_json::to_string(&value)` plus `structured_content: Some(value)`.
- **Transport:** `StreamableHttpService::new(factory, LocalSessionManager::default().into(), StreamableHttpServerConfig::default())`. Keep rmcp's default `allowed_hosts` of localhost, `127.0.0.1` and `::1`, which defends against DNS rebinding. Mount it with `Router::nest_service("/mcp", service)`. If the axum 0.7 body type doesn't satisfy rmcp's service bounds, serve it with a hyper/`axum::serve` 1.x-compatible adapter instead. Record which in a one-line comment.
- **Start:**
  - Bind `TcpListener` to `127.0.0.1:{port}` _before_ returning. A bind error returns `Err`.
  - Serve with graceful shutdown on a `tokio::sync::oneshot`.

- [ ] **Step 1: Write failing tests**

  Drive `agent_router(db, Arc::new(FakeExecutor))` with `tower::ServiceExt::oneshot`. `FakeExecutor` echoes `{"tool": name, "args": arguments}`. Use a helper that runs MCP `initialize` → `tools/list` / `tools/call` JSON-RPC over POST `/mcp` with `Accept: application/json, text/event-stream` and `Host: 127.0.0.1`. Parse either a JSON or an SSE `data:` body.
  - `auth_header_variants`:
    - Missing header, `Basic x`, `Bearer`, `Bearer not_a_token`, and `Bearer sage_agent_<64 hex unknown>` → 401.
    - `bearer <valid>` and `Bearer   <valid>  ` → not 401.
  - `list_tools_only_shows_granted`: an agent granted `{WalletGetWallet}` → exactly `["get_wallet"]`.
  - `call_ungranted_tool_is_tool_error`: `is_error == true`, and the text starts with `"permission_denied"`.
  - `call_unknown_tool_is_protocol_error`.
  - `call_granted_tool_returns_executor_value`: `structured_content == {"tool":"get_wallet","args":{}}`.
  - `revoked_agent_is_rejected_mid_session`: initialize OK, `delete_agent`, then the next request → 401.
  - `capability_removed_mid_session_hides_tool`: initialize, `update_agent` with empty grants, then `tools/list` is empty.
  - `rotated_token_rejects_old`.
  - `non_local_host_header_rejected`: `Host: evil.example` → not 2xx.
  - `last_used_is_recorded`.
  - `token_never_echoed`: a request with `Bearer sage_agent_<64 hex unknown>` gets a 401 whose body and headers don't contain the token. The auth layer must never log the `Authorization` header or token, and `AgentToken` must not implement `Debug` or `Display`.

- [ ] **Step 2: Run and verify they fail**

  Run: `cargo test -p sage-apps agents::server`. Expected: compile failure.

- [ ] **Step 3: Implement**

- [ ] **Step 4: Verify they pass**

  Run: `cargo test -p sage-apps agents::server`. Expected: all pass.

---

### Task 6: Server lifecycle and Tauri commands

**Files:**

- Create: `crates/sage-apps/src/agents/commands.rs`
- Modify:
  - `crates/sage-apps/src/host.rs`: add `pub(crate) agent_server: tokio::sync::Mutex<Option<AgentServerHandle>>` to `AppsHostState`, initialized to `None`.
  - `crates/sage-apps/src/lib.rs`: export the commands and `start_agent_server_if_enabled`.
  - `src-tauri/src/lib.rs`: register the commands next to `apps::apps_get_auto_update_enabled`.
  - `src-tauri/src/commands.rs`: in `initialize`, after the RPC autostart, `#[cfg(not(mobile))]` call `sage_apps::start_agent_server_if_enabled(app_handle.clone()).await`. Log errors with `error!`; don't fail initialize.
  - `src-tauri/permissions/main-host.toml`: add every new command.
  - `src/bindings.ts`: regenerate.
- Test: `#[cfg(test)]` in `agents/server.rs` (lifecycle) and `agents/commands.rs` (pure helpers)

**Interfaces:**

- Consumes: Tasks 1, 3, 4 and 5.
- Produces these Tauri commands, all `#[command] #[specta::specta]`, returning `crate::Result<_>`:
  ```rust
  apps_get_agent_server_status() -> AgentServerStatus          // { running: bool, enabled: bool, port: u16 }
  apps_set_agent_server_settings(enabled: bool, port: u16) -> AgentServerStatus // persists; starts/stops/restarts to match
  apps_list_agents() -> Vec<SageAgent>
  apps_list_agent_capabilities() -> Vec<AgentCapabilityInfo>   // { capability, label, description } from agent_grantable_capabilities()
  apps_create_agent(name: String, capabilities: Vec<UserBridgeCapability>, wallet_scope: SageAppWalletScope) -> CreatedAgent // { agent, token: String }
  apps_update_agent(id: String, name: String, capabilities: Vec<UserBridgeCapability>, wallet_scope: SageAppWalletScope) -> SageAgent
  apps_rotate_agent_token(id: String) -> String
  apps_revoke_agent(id: String) -> ()
  pub async fn start_agent_server_if_enabled(app_handle: AppHandle);
  pub(crate) fn validate_agent_request(name: &str, capabilities: &[UserBridgeCapability]) -> Result<(), String>;
  ```

`validate_agent_request` rejects:

- an empty or whitespace name, or one longer than 64 chars;
- any capability not in `agent_grantable_capabilities()`, the server-side guard against a tampered UI.

Port `0` and ports `< 1024` are rejected by `apps_set_agent_server_settings`.

- [ ] **Step 1: Write failing tests**
  - `start_fails_on_occupied_port`: bind a `std::net::TcpListener` on `127.0.0.1:0`, then `start_agent_server(db, fake, that_port)` is `Err`.
  - `start_then_stop_frees_port`: start on a free port, `stop().await`, then binding that port again succeeds.
  - `validate_rejects_secret_capability` (`WalletGetSecretKey`), `validate_rejects_signing_capability` (`WalletSignCoinSpends`), `validate_rejects_blank_name`, `validate_accepts_read_set`.

- [ ] **Step 2: Run and verify they fail**

  Run: `cargo test -p sage-apps agents`. Expected: compile failure.

- [ ] **Step 3: Implement the commands, lifecycle and wiring**

  Create and rotate return the plaintext token only in their result. Then run `pnpm generate:bindings`.

- [ ] **Step 4: Verify**

  Run `cargo test -p sage-apps agents` and `cargo clippy --workspace --all-features --all-targets`. Then `grep -c apps_create_agent src-tauri/permissions/main-host.toml src/bindings.ts` should print a count of at least 1 for each file. Expected: tests pass, no warnings, and all eight commands appear in both files.

---

### Task 7: Settings → Agents UI

**Files:**

- Create: `src/pages/settings/AgentSettings.tsx` (if `src/pages/settings/` doesn't exist, put it at `src/components/AgentSettings.tsx`)
- Modify: `src/pages/Settings.tsx`: render `{!isMobile && <AgentSettings />}` right after `<RpcSettings />` in the Advanced tab

**Interfaces:**

- Consumes: the Task 6 commands through `commands.*` in `src/bindings.ts`.

What it contains (reuse `SettingsSection`, `SettingItem`, `Switch`, `Button` and the dialog components already used in `Settings.tsx`; mirror `RpcSettings` for status polling and `promptIfEnabled()` before start, create and rotate):

- **Server block:**
  - A running dot and label.
  - An enable switch that calls `apps_set_agent_server_settings`.
  - A port input validated as 1024–65535.
  - An amber warning `<Trans>The RPC server is also running. Its certificate on disk gives full wallet access and bypasses agent approvals.</Trans>`, shown while `commands.isRpcRunning()` is true.
- **Agent list:** each row shows the name, the scope summary ("This wallet" / "N wallets" / "All wallets"), the number of granted capabilities and last used. Each row has an Edit and a Revoke button, with a confirm step before revoking.
- **Add agent dialog:**
  - A name field.
  - Wallet scope, defaulting to `SelectedWallets { fingerprints: [current] }`, with an "All wallets" option.
  - Capability checkboxes from `apps_list_agent_capabilities` (label plus description), all checked by default.
  - On create it shows the token once, in a read-only field with a copy button. It also shows two copyable snippets, with the real port and token:
    - `claude mcp add --transport http sage http://127.0.0.1:<port>/mcp --header "Authorization: Bearer <token>"`
    - `{"mcpServers":{"sage":{"type":"http","url":"http://127.0.0.1:<port>/mcp","headers":{"Authorization":"Bearer <token>"}}}}`
  - The text says `<Trans>This token is shown only once.</Trans>`.
- **Edit dialog:** the same fields as Add, plus a "Rotate token" button that shows the new token the same way.

- [ ] **Step 1: Implement**

- [ ] **Step 2: Write the user doc `docs/agents.md`**

  Cover:
  - What agents are.
  - How to enable the server and add an agent.
  - Both client snippets.
  - The stage 1 tool list.
  - Revoke and rotate.
  - A **Security** section saying that a token only grants its capabilities, but an agent that can run shell commands as your OS user can do anything you can. That includes reading Sage's RPC certificate while the RPC server is enabled. Recommend running such agents sandboxed or as a different OS user, and keeping the RPC server off while using agents.

- [ ] **Step 3: Verify**

  Run: `pnpm lint && pnpm prettier:check && npx tsc -b`. Expected: clean. Don't run `pnpm extract`.

---

### Task 8: End-to-end check and full CI pass

- [ ] **Step 1: Run the full CI gate**

  Run with `SDKROOT` exported:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-features --all-targets`
  - `cargo test --workspace --all-features`
  - `pnpm lint`
  - `pnpm prettier:check`
  - `npx tsc -b`

  Expected: all clean.

- [ ] **Step 2: Manual run** (use the `run` skill, or `pnpm tauri dev`)
  1. Log in to a wallet. Go to Settings → Advanced → Agents and enable the server. It shows Running on 9258.
  2. Add an agent "test" with the defaults and copy the `claude mcp add …` command.
  3. In a terminal, run that command, then `claude mcp list`. `sage` shows as connected.
  4. In Claude Code, ask for the wallet's XCH balance. `get_asset_balance` succeeds and matches the UI.
  5. Edit the agent and uncheck "Read wallet". `get_wallet` disappears from the tools and calling it fails.
  6. Switch to a wallet outside the agent's scope. Tool calls return `wallet_not_in_scope`.
  7. Revoke the agent. The next call fails with 401.
  8. Start the RPC server. The warning appears.
  9. Restart Sage with the server enabled. It autostarts.

- [ ] **Step 3: Report**

  List any step that failed, with output. Mention that the next release will need `pnpm extract` for the new strings.
