# Agents on a Shared Authorization Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Sage Agents (PR #1005) off the Sage Apps bridge onto a shared authorization crate, with agent tools calling `sage_api` endpoints and `sage-apps` returned to `main`.

**Architecture:** A new `sage-auth` crate holds identities, capabilities, wallet scope, the `authorize` check and the agent store (`agents.sqlite`). A new `sage-mcp` crate holds the MCP server, moved out of `sage-apps`. `sage-rpc` calls `authorize` with the owner identity; `src-tauri` owns the agent commands and the executor that runs endpoints under one `Sage` lock.

**Tech Stack:** Rust (axum 0.7, rmcp 3.10, sqlx/SQLite, utoipa, specta), Tauri 2, React/TypeScript, vitest.

**Spec:** `docs/superpowers/specs/2026-11-15-agents-shared-authorization-design.md`. Read it before any task.

## Global Constraints

- Work on the existing `agents-mcp` branch. **Never commit, stage for commit, push, or amend.** The user handles all git commits. Where a task says "Checkpoint", stop and report; do not commit.
- No change an existing RPC client could observe: same URLs, bodies, credentials, status codes.
- `crates/sage-apps` must end identical to the merge base with `upstream/main` (`BASE=$(git merge-base HEAD upstream/main)`), verified by `git diff --stat $BASE -- crates/sage-apps` printing nothing.
- `sage-auth` must not depend on `sage-apps`, `tauri` or `rmcp`. `sage-mcp` must not depend on `sage-apps` or `tauri`.
- An endpoint with no capability in the map is denied for every identity except `Owner`.
- Token format unchanged: `sage_agent_` + 64 lowercase hex characters; only the SHA-256 hash is stored; the token is never logged or returned after creation or rotation.
- MCP server binds `127.0.0.1` only, default port `9258`, minimum port `1024`.
- MCP error codes unchanged: `permission_denied`, `no_wallet`, `wallet_not_in_scope`.
- Never edit `src/bindings.ts` by hand; regenerate with `pnpm generate:bindings`. Never launch the Sage GUI.
- Never run `pnpm extract` or touch `src/locales/**/*.po`. New user-facing strings are still wrapped with Lingui.
- Rustdoc comments are one line.
- Cargo may need `export SDKROOT="$(xcrun --show-sdk-path)"` on macOS ("stdlib.h not found").
- New Tauri commands must be listed in `src-tauri/permissions/main-host.toml` or they are denied at runtime.

## Review Focus

1. **An agent passes a `fingerprint` argument to `get_wallet`.** It must get the logged-in wallet, never another one. Pinned in Task 3.
2. **The wallet is switched or logged out between the check and the endpoint.** The call must be refused or run on an in-scope wallet, never an out-of-scope one. Pinned in Task 4 (one lock) and Task 3 (server-side check).
3. **`agents.sqlite` is missing, corrupt or locked.** Sage must still open the wallet; agent commands must report the reason. Pinned in Task 4.
4. **A capability key stored for an agent is not in the enum** (older or newer build). The agent must still load, without the unknown capability, not fail the whole list. Pinned in Task 2.
5. **A wallet is deleted over RPC or the CLI, then re-imported.** The agent must not regain access. Pinned in Task 2 (prune) and Task 4 (prune on start and on list).

## File Structure

```
crates/sage-auth/
  Cargo.toml
  src/lib.rs            re-exports
  src/capability.rs     Capability enum, labels, endpoint map
  src/identity.rs       Identity, Agent, WalletScope
  src/authorize.rs      authorize, Decision, AuthError
  src/token.rs          AgentToken (moved from sage-apps/src/types/agent.rs)
  src/store.rs          AgentStore, AgentServerSettings (moved from sage-apps/src/db/agents.rs)
  src/migrations/0001_agents.sql
crates/sage-mcp/
  Cargo.toml
  src/lib.rs            re-exports
  src/schema.rs         moved from sage-apps/src/agents/schema.rs
  src/tools.rs          tool catalog over sage_api endpoints
  src/executor.rs       AgentToolExecutor trait
  src/server.rs         moved from sage-apps/src/agents/server.rs
src-tauri/src/agents.rs agent commands, server lifecycle, SageAgentExecutor (desktop only)
```

---

### Task 1: `sage-auth` core: capabilities, identities, the check

**Files:**

- Create: `crates/sage-auth/Cargo.toml`, `src/lib.rs`, `src/capability.rs`, `src/identity.rs`, `src/authorize.rs`
- Modify: root `Cargo.toml` (`[workspace.dependencies]`: add `sage-auth = { path = "./crates/sage-auth" }`, matching how the other `sage-*` crates are listed)

**Interfaces:**

- Produces:

```rust
// capability.rs — variant names and keys match sage-apps' UserBridgeCapability so Apps can adopt it
pub enum Capability {
    WalletGetWallet, WalletGetSyncStatus, WalletGetVersion, WalletGetToken, WalletGetCoins,
    WalletGetCoinsByIds, WalletGetSpendableCoinCount, WalletGetTransactions, WalletGetTransaction,
    WalletGetPendingTransactions, WalletGetXchUsdPrice, WalletCheckAddress,
}
impl Capability {
    pub const ALL: &'static [Capability];
    pub fn key(self) -> &'static str;                 // "wallet." + endpoint, e.g. "wallet.get_coins"
    pub fn from_key(key: &str) -> Option<Self>;
    pub fn endpoint(self) -> Option<&'static str>;    // sage-api endpoint name; Some for all twelve today
    pub fn label(self) -> &'static str;
    pub fn description(self) -> &'static str;
}
pub fn endpoint_capability(endpoint: &str) -> Option<Capability>;

// identity.rs
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WalletScope { AllWallets, SelectedWallets { fingerprints: Vec<u32> } }
impl WalletScope { pub fn allows(&self, fingerprint: u32) -> bool; }
impl Default for WalletScope // SelectedWallets with no fingerprints

pub struct Grant { pub capabilities: BTreeSet<Capability>, pub wallet_scope: WalletScope }
pub enum Identity { Owner, Grantee(Grant) }

#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String, pub name: String,
    pub capabilities: BTreeSet<Capability>, pub wallet_scope: WalletScope,
    pub created_at_ms: i64, pub updated_at_ms: i64, pub last_used_at_ms: Option<i64>,
}
impl Agent { pub fn identity(&self) -> Identity; }   // Identity::Grantee with the agent's grant

// authorize.rs
#[non_exhaustive] pub enum Decision { Allowed }
pub enum AuthError { PermissionDenied { what: String }, NoWallet, WalletNotInScope }
impl AuthError { pub fn code(&self) -> &'static str; } // "permission_denied" | "no_wallet" | "wallet_not_in_scope"
pub fn authorize(identity: &Identity, capability: Capability, current_fingerprint: Option<u32>)
    -> Result<Decision, AuthError>;
pub fn authorize_endpoint(identity: &Identity, endpoint: &str, current_fingerprint: Option<u32>)
    -> Result<Decision, AuthError>;
```

`Capability` derives `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize`, serializing each variant as its key (`#[serde(rename = "wallet.get_coins")]`, with the same `#[specta(rename = ..)]`), the way `define_bridge_capabilities!` does in `crates/sage-apps/src/capabilities/list.rs`. `WalletGetWallet` also accepts the alias `"wallet.get_key"`, as Apps does. `WalletScope`, `Grant`, `Agent` derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`. All serialized types derive `specta::Type` under feature `tauri = ["dep:specta"]`, mirroring `crates/sage-api/Cargo.toml`. `AuthError` uses `thiserror`.

`authorize` is the core check and knows nothing about endpoints, so a caller with no RPC endpoint (an app signing a message, say) can use it later. `authorize_endpoint` is the helper RPC uses: `Owner` is allowed for any endpoint; a grantee needs the endpoint to be in the map and is then passed to `authorize`.

- [ ] **Step 1: Write the failing tests** in `src/capability.rs` and `src/authorize.rs`:

```rust
// capability.rs
#[test] fn keys_are_wallet_dot_endpoint_and_round_trip() {
    for &c in Capability::ALL {
        let endpoint = c.endpoint().unwrap();
        assert_eq!(c.key(), format!("wallet.{endpoint}"));
        assert_eq!(Capability::from_key(c.key()), Some(c));
        assert_eq!(endpoint_capability(endpoint), Some(c));
        assert_eq!(serde_json::to_value(c).unwrap(), serde_json::json!(c.key()));
        assert!(!c.label().is_empty() && !c.description().is_empty());
    }
    assert_eq!(Capability::ALL.len(), 12);
}
#[test] fn old_apps_alias_still_parses() {
    assert_eq!(Capability::from_key("wallet.get_key"), Some(Capability::WalletGetWallet));
}
#[test] fn endpoints_are_real_sage_api_endpoints() {
    let endpoints: serde_json::Value =
        serde_json::from_str(include_str!("../../sage-api/endpoints.json")).unwrap();
    for &c in Capability::ALL {
        let e = c.endpoint().unwrap();
        assert!(endpoints.get(e).is_some(), "{e}");
    }
}
#[test] fn write_and_secret_endpoints_are_unmapped() {
    for e in ["send_xch", "sign_coin_spends", "get_secret_key", "delete_wallet", "login", "submit_transaction"] {
        assert_eq!(endpoint_capability(e), None, "{e}");
    }
}

// authorize.rs — grantee(caps, scope) builds an Identity::Grantee
use Capability::*;
#[test] fn owner_is_allowed_everything() {
    assert!(authorize(&Identity::Owner, WalletGetCoins, None).is_ok());
    for e in ["get_coins", "send_xch", "not_an_endpoint"] {
        assert!(authorize_endpoint(&Identity::Owner, e, None).is_ok(), "{e}");
    }
}
#[test] fn grantee_on_unmapped_endpoint_is_denied() {
    let g = grantee(Capability::ALL, WalletScope::AllWallets);
    assert_eq!(authorize_endpoint(&g, "send_xch", Some(1)).unwrap_err().code(), "permission_denied");
}
#[test] fn grantee_without_the_capability_is_denied() {
    let g = grantee(&[WalletGetVersion], WalletScope::AllWallets);
    assert_eq!(authorize(&g, WalletGetCoins, Some(1)).unwrap_err().code(), "permission_denied");
    assert_eq!(authorize_endpoint(&g, "get_coins", Some(1)).unwrap_err().code(), "permission_denied");
}
#[test] fn grantee_with_no_wallet_logged_in_is_refused() {
    let g = grantee(&[WalletGetCoins], WalletScope::AllWallets);
    assert_eq!(authorize(&g, WalletGetCoins, None).unwrap_err().code(), "no_wallet");
}
#[test] fn grantee_outside_its_scope_is_refused() {
    let g = grantee(&[WalletGetCoins], WalletScope::SelectedWallets { fingerprints: vec![7] });
    assert_eq!(authorize(&g, WalletGetCoins, Some(8)).unwrap_err().code(), "wallet_not_in_scope");
    assert!(authorize(&g, WalletGetCoins, Some(7)).is_ok());
    assert!(authorize_endpoint(&g, "get_coins", Some(7)).is_ok());
}
#[test] fn capability_is_checked_before_wallet() {
    let g = grantee(&[], WalletScope::AllWallets);
    assert_eq!(authorize(&g, WalletGetCoins, None).unwrap_err().code(), "permission_denied");
}
#[test] fn default_scope_allows_nothing() { assert!(!WalletScope::default().allows(1)); }
#[test] fn agent_identity_carries_its_grant() {
    let a = Agent { id: "a".into(), name: "n".into(), capabilities: [WalletGetCoins].into(),
        wallet_scope: WalletScope::AllWallets, created_at_ms: 0, updated_at_ms: 0, last_used_at_ms: None };
    assert!(authorize(&a.identity(), WalletGetCoins, Some(1)).is_ok());
    assert!(authorize(&a.identity(), WalletGetVersion, Some(1)).is_err());
}
```

- [ ] **Step 2: Run** `cargo test -p sage-auth` — expected: fails to compile (crate has no items).
- [ ] **Step 3: Implement** the items in the Interfaces block. `authorize` applies the spec's "The check" rules in order. Labels and descriptions: take the wording from the matching `Wallet*` entries in `crates/sage-apps/src/capabilities/definitions.rs`, replacing "the app" with "the caller"; write new ones for `WalletGetToken`.
- [ ] **Step 4: Run** `cargo test -p sage-auth` — expected: all pass. Run `cargo clippy -p sage-auth --all-features --all-targets` — expected: no warnings.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 2: `sage-auth` store and tokens

**Files:**

- Create: `crates/sage-auth/src/token.rs`, `src/store.rs`, `src/migrations/0001_agents.sql`
- Modify: `crates/sage-auth/Cargo.toml` (sqlx with the same features `crates/sage-apps/Cargo.toml` uses, `rand`, `sha2`, `hex`, `anyhow`; `tempfile` and `tokio` as dev-dependencies), `src/lib.rs`
- Source to move from: `crates/sage-apps/src/types/agent.rs` (`AgentToken` and its tests), `crates/sage-apps/src/db/agents.rs` (queries and tests), `crates/sage-apps/src/db/connection.rs:23-48` (pool setup pattern)

**Interfaces:**

- Consumes: `Agent`, `Capability`, `WalletScope` from Task 1.
- Capabilities are stored as a JSON list of keys (`"wallet.get_coins"`).
- Produces:

```rust
// token.rs — moved unchanged, including "no Debug or Display"
pub struct AgentToken(String);
impl AgentToken { pub fn generate() -> Self; pub fn parse(raw: &str) -> Option<Self>;
                  pub fn hash(&self) -> String; pub fn expose(&self) -> &str; }

// store.rs
pub const DEFAULT_AGENT_SERVER_PORT: u16 = 9258;
#[serde(rename_all = "camelCase")]
pub struct AgentServerSettings { pub enabled: bool, pub port: u16 } // Default: false, 9258
#[derive(Debug, Clone)] pub struct AgentStore { /* SqlitePool */ }
impl AgentStore {
    pub async fn open(base_path: &Path) -> anyhow::Result<Self>;   // file "agents.sqlite"
    pub async fn insert_agent(&self, agent: &Agent, token_hash: &str) -> anyhow::Result<()>;
    pub async fn list_agents(&self) -> anyhow::Result<Vec<Agent>>;
    pub async fn get_agent(&self, id: &str) -> anyhow::Result<Option<Agent>>;
    pub async fn find_agent_by_token_hash(&self, token_hash: &str) -> anyhow::Result<Option<Agent>>;
    pub async fn update_agent(&self, ...) -> anyhow::Result<...>;          // same signature as AppsDb::update_agent
    pub async fn retain_agent_wallets(&self, existing: &[u32]) -> anyhow::Result<Vec<Agent>>;
    pub async fn set_agent_token_hash(&self, id: &str, token_hash: &str) -> anyhow::Result<bool>;
    pub async fn delete_agent(&self, id: &str) -> anyhow::Result<bool>;
    pub async fn touch_agent_last_used(&self, id: &str, at_ms: i64) -> anyhow::Result<()>;
    pub async fn get_server_settings(&self) -> anyhow::Result<AgentServerSettings>;
    pub async fn set_server_settings(&self, settings: AgentServerSettings) -> anyhow::Result<AgentServerSettings>;
}
pub fn unix_timestamp_ms() -> i64;
```

Migration `0001_agents.sql`: the `sage_agents` table exactly as in `crates/sage-apps/src/db/migrations/0005_agents.sql`, plus `CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);`. Settings keys stay `agent_server_enabled` and `agent_server_port`.

- [ ] **Step 1: Move the tests** from `types/agent.rs` and `db/agents.rs` into `token.rs` and `store.rs`, renaming `AppsDb` → `AgentStore`, `SageAgent` → `Agent`, `SageAppWalletScope` → `WalletScope`, `UserBridgeCapability::WalletX` → the matching `Capability`, and opening the store in a `tempfile::TempDir`. Add:

```rust
#[tokio::test] async fn unknown_stored_capability_is_dropped_not_fatal() {
    let (store, _dir) = store().await;
    store.insert_agent(&agent("a", &[Capability::WalletGetCoins]), "h").await.unwrap();
    sqlx::query("UPDATE sage_agents SET capabilities_json = '[\"wallet.get_coins\",\"wallet.send_xch\"]'")
        .execute(store.pool()).await.unwrap();
    let agents = store.list_agents().await.unwrap();
    assert_eq!(agents[0].capabilities, BTreeSet::from([Capability::WalletGetCoins]));
}
#[tokio::test] async fn open_fails_on_a_corrupt_file() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("agents.sqlite"), b"not a database").unwrap();
    assert!(AgentStore::open(dir.path()).await.is_err());
}
#[tokio::test] async fn pruned_wallet_does_not_return_when_reimported() {
    let (store, _dir) = store().await;
    store.insert_agent(&scoped("a", &[7, 8]), "h").await.unwrap();
    store.retain_agent_wallets(&[8]).await.unwrap();
    store.retain_agent_wallets(&[7, 8]).await.unwrap();
    let scope = store.get_agent("a").await.unwrap().unwrap().wallet_scope;
    assert_eq!(scope, WalletScope::SelectedWallets { fingerprints: vec![8] });
}
#[tokio::test] async fn settings_default_then_round_trip() {
    let (store, _dir) = store().await;
    assert_eq!(store.get_server_settings().await.unwrap(), AgentServerSettings::default());
    let next = AgentServerSettings { enabled: true, port: 9300 };
    store.set_server_settings(next).await.unwrap();
    assert_eq!(store.get_server_settings().await.unwrap(), next);
}
```

- [ ] **Step 2: Run** `cargo test -p sage-auth` — expected: compile failure on the missing items.
- [ ] **Step 3: Implement** by moving the code. Capabilities are read from JSON as a list of strings and each parsed individually, skipping names that don't parse (Review Focus 4). `pool()` is `#[cfg(test)] pub(crate)`.
- [ ] **Step 4: Run** `cargo test -p sage-auth` and `cargo clippy -p sage-auth --all-features --all-targets` — expected: pass, no warnings.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 3: `sage-mcp`: server and tool catalog over endpoints

**Files:**

- Create: `crates/sage-mcp/Cargo.toml`, `src/lib.rs`, `src/schema.rs`, `src/tools.rs`, `src/executor.rs`, `src/server.rs`
- Modify: root `Cargo.toml` (add `sage-mcp` to `[workspace.dependencies]`)
- Source to move from: `crates/sage-apps/src/agents/{schema,tools,executor,server}.rs`; dependency lines for `rmcp`, `axum`, `utoipa` from `crates/sage-apps/Cargo.toml:38-40`

**Interfaces:**

- Consumes: `sage_auth::{Agent, AgentStore, AgentToken, Capability, AuthError, authorize, unix_timestamp_ms}`; `sage_api` request types.
- Produces:

```rust
// tools.rs
pub struct AgentTool {
    pub name: &'static str,            // equals capability.endpoint().unwrap()
    pub capability: Capability,
    pub description: &'static str,
    pub input_schema: fn() -> serde_json::Value,
}
pub fn agent_tools() -> &'static [AgentTool];
pub fn find_agent_tool(name: &str) -> Option<&'static AgentTool>;
pub fn tools_for_agent(agent: &Agent) -> Vec<&'static AgentTool>;

// executor.rs
pub struct ToolError { pub code: String, pub message: String }
impl From<AuthError> for ToolError // code = err.code()
#[async_trait] pub trait AgentToolExecutor: Send + Sync + 'static {
    async fn current_fingerprint(&self) -> Option<u32>;
    async fn call(&self, agent: &Agent, tool: &AgentTool, arguments: serde_json::Value)
        -> Result<serde_json::Value, ToolError>;
}

// server.rs
pub struct AgentServerHandle; // port(), stop() as today
pub async fn start_agent_server(store: AgentStore, executor: Arc<dyn AgentToolExecutor>,
    version: String, port: u16) -> anyhow::Result<AgentServerHandle>;
```

Tool table (name = endpoint; schema type):

| Tool                       | Schema                               |
| -------------------------- | ------------------------------------ |
| `get_wallet`               | `no_params`                          |
| `get_sync_status`          | `no_params`                          |
| `get_version`              | `no_params`                          |
| `get_token`                | `schema_of::<GetToken>`              |
| `get_coins`                | `schema_of::<GetCoins>`              |
| `get_coins_by_ids`         | `schema_of::<GetCoinsByIds>`         |
| `get_spendable_coin_count` | `schema_of::<GetSpendableCoinCount>` |
| `get_transactions`         | `schema_of::<GetTransactions>`       |
| `get_transaction`          | `schema_of::<GetTransaction>`        |
| `get_pending_transactions` | `no_params`                          |
| `get_xch_usd_price`        | `no_params`                          |
| `check_address`            | `schema_of::<CheckAddress>`          |

Descriptions carry over from the current catalog; `get_token`: "Get an asset's details and confirmed and spendable balance (XCH by default)."

- [ ] **Step 1: Move the server tests** (`server.rs:300-1069`) and schema tests, adapting the fixture to `AgentStore` in a temp dir and `Capability`. In `call_tool`, the server calls `sage_auth::authorize(&agent.identity(), tool.capability, executor.current_fingerprint().await)` before the executor, replacing `authorize_agent_call`. Replace `secret_and_signing_grants_expose_no_tools` and the registry tests with:

```rust
#[test] fn catalog_has_one_tool_per_capability() {
    let names: Vec<_> = agent_tools().iter().map(|t| t.name).collect();
    let endpoints: Vec<_> = Capability::ALL.iter().map(|c| c.endpoint().unwrap()).collect();
    assert_eq!(names, endpoints);
    for t in agent_tools() { assert_eq!(Some(t.name), t.capability.endpoint()); }
}
#[test] fn get_wallet_takes_no_arguments() {
    let schema = (find_agent_tool("get_wallet").unwrap().input_schema)();
    assert_eq!(schema["properties"], serde_json::json!({}));
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
}
```

Keep `switching_out_of_scope_refuses_calls` (it also covers logout), `call_ungranted_tool_is_tool_error`, `capability_removed_mid_session_hides_tool`, `token_never_echoed`, the Host header tests and the start/stop tests.

- [ ] **Step 2: Run** `cargo test -p sage-mcp` — expected: compile failure.
- [ ] **Step 3: Implement** by moving the four modules and applying the Interfaces block. `BridgeMethodHandleError` becomes `ToolError`; drop every `BridgeRegistry` use.
- [ ] **Step 4: Run** `cargo test -p sage-mcp` and `cargo clippy -p sage-mcp --all-features --all-targets` — expected: pass, no warnings.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 4: `src-tauri`: agent commands, executor, lifecycle

**Files:**

- Create: `src-tauri/src/agents.rs` (whole module under `#[cfg(not(mobile))]`)
- Modify: `src-tauri/Cargo.toml` (add `sage-auth` with `features = ["tauri"]` and `sage-mcp` to the non-mobile target block at line 58), `src-tauri/src/lib.rs` (register commands, open the store, replace `apps::apps_*agent*` entries), `src-tauri/src/commands.rs` (the `sage_apps::start_agent_server_if_enabled` call becomes `agents::start_agent_server_if_enabled`), `src-tauri/permissions/main-host.toml`
- Source to move from: `crates/sage-apps/src/agents/commands.rs`, `crates/sage-apps/src/agents/executor.rs`

**Interfaces:**

- Consumes: Tasks 1–3.
- Produces Tauri commands (specta-exported, so the front end sees `commands.agentsX`):

```rust
agents_get_server_status() -> Result<AgentServerStatus>
agents_set_server_settings(enabled: bool, port: u16) -> Result<AgentServerStatus>
agents_list_agents() -> Result<Vec<Agent>>
agents_list_capabilities() -> Result<Vec<AgentCapabilityView>>
agents_create_agent(name: String, capabilities: Vec<Capability>, wallet_scope: WalletScope) -> Result<CreatedAgent>
agents_update_agent(id: String, name: String, capabilities: Vec<Capability>, wallet_scope: WalletScope) -> Result<Agent>
agents_rotate_agent_token(id: String) -> Result<String>
agents_revoke_agent(id: String) -> Result<()>
```

```rust
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityView { pub key: String, pub label: String, pub description: String }
pub struct AgentServerStatus { running, enabled, port, start_error }   // unchanged shape
pub struct CreatedAgent { pub agent: Agent, pub token: String }        // Debug redacts token
pub struct AgentsState { store: Option<AgentStore>, open_error: Option<String>, server: Mutex<AgentServerState> }
pub async fn start_agent_server_if_enabled(app_handle: AppHandle);
```

Errors use `crate::error::Error` with `ErrorKind::Api` / `ErrorKind::NotFound` / `ErrorKind::Internal`, as the moved code did through `SageAppsError`.

**Executor** (`SageAgentExecutor`, holding `AppState`): `call` locks `Sage` once, reads `sage.config.global.fingerprint`, calls `sage_auth::authorize(&agent.identity(), tool.capability, fingerprint)`, then dispatches on `tool.capability` with one arm per capability: deserialize `arguments` into the endpoint's request type, call the `Sage` method, serialize the response. `GetWallet` ignores `arguments` and always passes `GetWallet { fingerprint: None }`. The lock is held from the fingerprint read until the endpoint returns.

**Store open** (in `lib.rs` setup, desktop only): `AgentStore::open(&path)`; on error log it and manage `AgentsState { store: None, open_error: Some(format!("{err:#}")), .. }`. Never return early or panic.

**When `store` is `None`:** `agents_get_server_status` returns `{ running: false, enabled: false, port: 9258, start_error: open_error }`; every other command returns `ErrorKind::Internal` with the reason; `start_agent_server_if_enabled` returns without starting.

**Pruning:** `retain_agent_wallets` runs in `start_agent_server_if_enabled` and in `agents_list_agents`, as today.

- [ ] **Step 1: Move the tests** from `commands.rs` (validation, `server_status`, `change_server`, `apply_server_settings` rollback) into `agents.rs`. Add:

```rust
#[test] fn status_reports_the_open_error_even_when_disabled() {
    let status = status_without_store("disk full");
    assert!(!status.running && !status.enabled);
    assert_eq!(status.port, 9258);
    assert_eq!(status.start_error.as_deref(), Some("disk full"));
}
#[test] fn every_capability_is_grantable_and_listed() {
    let views = capability_views();
    assert_eq!(views.len(), Capability::ALL.len());
    assert_eq!(views[0].key, Capability::ALL[0].key());
}
#[tokio::test] async fn executor_ignores_a_fingerprint_argument_to_get_wallet() { /* two wallets in a
    temp-dir Sage, logged in to A; call get_wallet with {"fingerprint": B}; response is A */ }
#[tokio::test] async fn executor_refuses_after_switching_out_of_scope() { /* agent scoped to A; log in
    to B; call → Err code "wallet_not_in_scope" */ }
#[tokio::test] async fn executor_refuses_when_logged_out() { /* → Err code "no_wallet" */ }
```

For the three executor tests, build `Sage` the way `crates/sage-rpc/src/tests.rs` does (temp dir, imported test mnemonics). If that helper needs dev-dependencies `src-tauri` lacks, add them under `[dev-dependencies]`.

- [ ] **Step 2: Run** `cargo test -p sage-tauri agents` — expected: compile failure.
- [ ] **Step 3: Implement** the module and wiring. In `main-host.toml`, replace the eight `apps_*agent*` names with the eight `agents_*` names above. Leave `crates/sage-apps` untouched in this task; Task 6 reverts that crate.
- [ ] **Step 4: Run** `cargo test -p sage-tauri agents` and `cargo check -p sage-tauri` — expected: pass. `sage-apps` still compiles with its now-unused agent code; that is expected until Task 6.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 5: `sage-rpc` owner check

**Files:**

- Modify: `crates/sage-rpc/Cargo.toml` (add `sage-auth = { workspace = true }`), `crates/sage-rpc/src/lib.rs:36-39`
- Test: `crates/sage-rpc/src/tests.rs`

**Interfaces:**

- Consumes: `sage_auth::{authorize_endpoint, Identity}`.

The generated handler becomes: lock `Sage`; `authorize_endpoint(&Identity::Owner, endpoint_string, sage.config.global.fingerprint)`; on `Err` respond `403` with the error text; otherwise run the endpoint as today. The owner path cannot produce `Err`, so no existing response changes.

- [ ] **Step 1: Write the test** in `tests.rs`, using the file's existing router helper:

```rust
#[tokio::test] async fn owner_reaches_unmapped_and_mapped_endpoints_without_a_wallet() {
    // No wallet logged in. get_version (mapped) and get_networks (unmapped) both return 200.
}
```

- [ ] **Step 2: Run** `cargo test -p sage-rpc owner_reaches` — expected: passes already (behaviour is unchanged); it pins the constraint. Note the full `cargo test -p sage-rpc` count before the change.
- [ ] **Step 3: Implement** the handler change.
- [ ] **Step 4: Run** `cargo test -p sage-rpc` — expected: same count as Step 2, all pass, no existing test edited. `cargo clippy -p sage-rpc --all-features --all-targets` — no warnings.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 6: Return `sage-apps` and the Apps database work to `main`

**Files:**

- Restore to merge base: `crates/sage-apps/**`, `crates/sage/src/sage.rs`, `crates/sage-api/src/types/error_kind.rs`, `src-tauri/src/app_state.rs`
- Modify: `crates/sage-rpc/src/lib.rs` (drop the `ErrorKind::AppsDatabase` arm), `src-tauri/src/commands.rs` (remove `reset_apps_database`, the `AppsDbFailure` branch in `initialize`, and the now-unused imports; keep the `agents::start_agent_server_if_enabled` call), `src-tauri/src/lib.rs` (remove `commands::reset_apps_database`; restore the `.expect("failed to initialize Sage apps database")` form), `src-tauri/permissions/main-host.toml` (remove `reset_apps_database`)

- [ ] **Step 1: Restore**

```bash
BASE=$(git merge-base HEAD upstream/main)
git restore --source="$BASE" --worktree -- crates/sage-apps crates/sage/src/sage.rs \
  crates/sage-api/src/types/error_kind.rs src-tauri/src/app_state.rs
```

`git restore` also removes tracked files that don't exist at `$BASE`. If Step 3's diff still lists any path under `crates/sage-apps` (expected candidates: `src/agents.rs`, `src/agents/`, `src/db/agents.rs`, `src/db/migrations/0005_agents.sql`, `src/types/agent.rs`), delete that path from the working tree by hand.

- [ ] **Step 2: Edit** the four hand-modified files listed above.
- [ ] **Step 3: Verify the restore**

Run: `git diff --stat "$BASE" -- crates/sage-apps crates/sage/src/sage.rs crates/sage-api src-tauri/src/app_state.rs`
Expected: no output.

Run: `grep -rn "AppsDbFailure\|reset_apps_database\|AppsDatabase\|lock_data_dir\|BridgeCaller::Agent\|process_agent" crates src-tauri`
Expected: no output.

- [ ] **Step 4: Run** `cargo test -p sage-apps -p sage -p sage-rpc -p sage-tauri` — expected: all pass. `cargo clippy --workspace --all-features --all-targets` — no warnings. `cargo tree -p sage-auth | grep -c sage-apps` and `cargo tree -p sage-mcp | grep -c "sage-apps\|tauri "` — expected: `0` each.
- [ ] **Step 5: Checkpoint.** Do not commit.

---

### Task 7: Front end

**Files:**

- Regenerate: `src/bindings.ts` via `pnpm generate:bindings`
- Restore to merge base: `src/components/StartupError.tsx`, `src/contexts/ErrorContext.tsx`; delete `src/components/StartupError.test.tsx`
- Modify: `src/lib/agentPermissions.ts`, `src/lib/agentPermissions.test.ts`, `src/components/AgentPermissionsPanel.tsx`, `src/components/AgentPermissionsPanel.test.tsx`, `src/pages/settings/AgentSettings.tsx`

**Interfaces:**

- Consumes: generated `commands.agentsGetServerStatus`, `agentsSetServerSettings`, `agentsListAgents`, `agentsListCapabilities`, `agentsCreateAgent`, `agentsUpdateAgent`, `agentsRotateAgentToken`, `agentsRevokeAgent`; types `Agent`, `Capability`, `WalletScope`, `AgentCapabilityView`.

The shared `PermissionEditor` takes capability definitions shaped `{ key, label, description, flags }` and capability keys typed as `UserBridgeCapability`. `agentPermissions.ts` is the adapter: add `toEditorDefinitions(views: AgentCapabilityView[])` returning that shape with flags `{ externallyObservable: false, accessesSensitiveSecret: false, requestableByApp: true, userGrantable: true }`, and keep the casts between `Capability` and the editor's key type inside this file only.

- [ ] **Step 1: Regenerate bindings.** Run `pnpm generate:bindings` — expected: `src/bindings.ts` gains the `agents*` commands and `Agent`, `Capability`, `WalletScope`, `AgentCapabilityView`; loses `apps*Agent*`, `resetAppsDatabase`, `SageAgent`, and `AppsDatabase` in `ErrorKind`.
- [ ] **Step 2: Restore** `git restore --source="$(git merge-base HEAD upstream/main)" --worktree -- src/components/StartupError.tsx src/contexts/ErrorContext.tsx && rm src/components/StartupError.test.tsx`
- [ ] **Step 3: Update the tests** in `agentPermissions.test.ts` and `AgentPermissionsPanel.test.tsx` for the renamed types, and add:

```ts
it('adapts agent capabilities to editor definitions the user can grant', () => {
  const [d] = toEditorDefinitions([
    { key: 'wallet.get_coins', label: 'L', description: 'D' },
  ]);
  expect(d).toEqual({
    key: 'wallet.get_coins',
    label: 'L',
    description: 'D',
    flags: {
      externallyObservable: false,
      accessesSensitiveSecret: false,
      requestableByApp: true,
      userGrantable: true,
    },
  });
});
```

- [ ] **Step 4: Run** `pnpm test:unit` — expected: the new test fails, renamed imports fail to resolve.
- [ ] **Step 5: Implement**: the adapter, the type renames, and the eight command renames in `AgentSettings.tsx`. When `status.startError` is set and the server is not enabled, the settings section still shows it (today it only shows while enabled).
- [ ] **Step 6: Run** `pnpm test:unit`, `pnpm exec tsc -b`, `pnpm prettier:check`, `pnpm lint` — expected: all clean.
- [ ] **Step 7: Browser check.** Run `pnpm sandbox` in the background, open the URL from `.sandbox/instance.json`, go to Settings → Connected Apps, create an agent, confirm the twelve capabilities list and the token is shown once. If the sandbox cannot serve the agent commands (they are desktop-only Tauri commands), report that rather than launching Sage.
- [ ] **Step 8: Checkpoint.** Do not commit.

---

### Task 8: Docs and full validation

**Files:**

- Modify: `docs/agents.md` (twelve-tool list; agents are no longer described as using the Sage apps model; storage is `agents.sqlite`; keep the lazy-prune note), `docs/superpowers/specs/2026-11-14-sage-agents-mcp-design.md` (add one line at the top: superseded in part by the 2026-11-15 design)
- Create: a scratch file outside the repo (rewritten PR description for the user to paste; not in the repo)

- [ ] **Step 1: Update the docs.**
- [ ] **Step 2: Run the full validation** from `.claude/rules/validation.md`:

```bash
pnpm prettier:check && pnpm lint
cargo fmt --all -- --files-with-diff --check
cargo clippy --workspace --all-features --all-targets
cargo test --workspace
pnpm test:unit
```

Expected: all clean. Report any failure with its output, including ones unrelated to this change.

- [ ] **Step 3: Final diff check.** `git diff --stat $(git merge-base HEAD upstream/main)` — expected: no path under `crates/sage-apps`; `packages/sage-app-ui` shows only the two editor files.
- [ ] **Step 4: Report** what changed, what was verified, and that `pnpm extract` is owed at the next release. Do not commit.
