# Sage Agents over MCP — Design

Status: draft for review · 2026-11-14

## Intent

Let AI agents act on a Sage wallet through the Model Context Protocol (MCP), so a user can run
things like trading bots: an agent watches a market (e.g. Dexie), and proposes or makes offers
using the user's wallet.

Every action that signs waits for the user to approve it in the Sage app. Later, a user can opt an
individual agent into acting autonomously within limits they set. Autonomy is always opt-in and
per agent.

## Goals

- An MCP server hosted by the running Sage app, which agents (Claude Code, Claude Desktop, Cursor,
  …) connect to over localhost.
- Per-agent credentials, created and revoked in the Sage app, each with its own capability grants
  and wallet scope.
- A small, curated tool set suited to trading: wallet state, balances, coins, transactions, and
  (stage 2) offers.
- Signing actions go through the same approval queue and approval UI that Sage apps use today.

## Non-goals

- Market data. Dexie, prices beyond what Sage already exposes, and chain exploration stay outside
  Sage. Agents use other tools for them (e.g. Dexie's API, Chia-Network's `chia-mcp`).
- Exposing the raw RPC API. Agents never get the ~270 RPC endpoints, and never get the mTLS
  admin credential.
- Headless support. `sage rpc start` gets no agent support in this design (see Constraints).
- Autonomous mode in stages 1–2. Stage 3 is sketched only.

## Anti-goals (failure even if it works)

- An agent can sign or spend without the user approving it, other than through an autonomy
  policy the user explicitly turned on for that agent.
- Sage gives an agent a credential stronger than its grants, such as the mTLS admin cert. Agents
  authenticate only with their own bearer token (see the threat model below).
- A second, separate permission model alongside Sage apps. Agents reuse the apps model.

**Threat model (decided).** Sage never hands an agent a credential stronger than its grants. Sage
does not try to defend against an agent that runs shell commands as the user's own OS account. Such
an agent can read `ssl/wallet.*` and edit the apps database, just like any other process the user
runs. To cover that:

- Settings → Agents shows a warning while the RPC server is also enabled, since the RPC cert on
  disk bypasses agent approvals.
- User docs say that agents with shell access can do anything the user can, and recommend running
  them sandboxed or as a different OS user.

**Delivery (decided).** Agents are not installable Sage apps. There is no zip, launchpad entry or
webview. They are a new kind of principal inside `sage-apps`, reusing its capabilities, grants,
wallet scope and approval pipeline and UI. They are managed in Settings → Agents.

- An agent acts on a wallet the user didn't grant it, including after the user switches wallets.

## Constraints and established patterns

- **`crates/sage-apps` is the pattern to follow.** It already has:
  - Capabilities (`UserBridgeCapability`, defined in `capabilities/definitions.rs` with labels,
    descriptions and flags such as `accesses_sensitive_secret`, `user_grantable`).
  - Per-app grants (`granted_permissions_json`) and wallet scope (`SageAppWalletScope`:
    `AllWallets` or `SelectedWallets { fingerprints }`), persisted in the apps database.
  - Bridge methods (`BridgeMethod` trait in `bridge/methods/shared.rs`): `capability()`,
    `prepare_approval()`, `binds_approval_to_wallet()`, `requires_signing()`,
    `handle_approved()`, `handle()`.
  - A request pipeline (`bridge/bridge_request.rs`): capability check → watch-only refusal for
    signing methods → `prepare_approval` → pending approval queue with expiry → approval UI (the
    bridge-approval system app, shown as a modal) → re-check capability and wallet binding →
    execute.
  - Approval summaries built from what the wallet actually built, not from request arguments
    (e.g. `wallet/sign_coin_spends.rs`). `sage wallet send` in `sage-cli` follows the same
    principle.
- **`sage-apps` depends on Tauri.** So the agent server runs only inside the GUI app. This was
  accepted: approval in the app is the point.
- **Tauri command ACL.** Any new Tauri command must be added to `main-host.toml`, or it's silently
  denied at runtime.
- **Translations.** Wrap new UI strings with Lingui. Do not run `pnpm extract` or edit `.po` files.
  The next release regenerates catalogs.
- **Imports and modules** follow `.claude/rules/imports.md`.
- **Sage state lock.** `Sage` sits behind one `Arc<Mutex<Sage>>`. Nothing may hold that lock
  while waiting for user approval.
- **Workspace axum is 0.7.** Check `rmcp`'s server transport against it. `rmcp` exposes its
  streamable-HTTP server as a tower service, which can be mounted or served on its own listener.

## Approaches considered

1. **Elicitation.** The MCP server asks the user to confirm through the agent's client (MCP
   `elicitation/create`), with a separate token and policy system in Sage. Works headless, but
   needs client support that varies, and duplicates a permission model Sage already has.
2. **Client tool permissions.** Mark spending tools destructive and let the client prompt. Nearly
   free, but approval belongs to the client (one "always allow" removes it), and prompts show raw
   JSON.
3. **Agents as Sage apps (chosen).** An agent is a new kind of app principal in `sage-apps`. Its
   MCP tools run as bridge methods, so grants, wallet scope, watch-only refusal, approval queue,
   approval UI, and expiry are all reused. Costs: GUI only, and the bridge pipeline must accept a
   caller that isn't a webview.

Chosen: **3**, because it follows the established pattern, gives agents and Sage apps one security
model and one approval UI, and turns stage 3 autonomy into an auto-approve decision inside the
existing approval path.

Also decided: **an MCP server hosted in Sage over streamable HTTP on localhost**, rather than a
stdio proxy over the mTLS RPC, because the proxy would need the admin certificate, which any agent
with shell access could read and use to bypass approvals.

## Design

### Agent identity and credentials

- An **agent** is a named principal with: an id, a display name, granted user capabilities, a
  wallet scope (`SageAppWalletScope`), created/last-used timestamps, and a token hash.
- **Bearer token.** 32 random bytes, encoded with a recognizable prefix (e.g. `sage_agent_…`).
  Shown once at creation, with a ready-to-paste client config snippet. Sage stores only its
  SHA-256 hash, which is enough for a high-entropy random token. Compare in constant time. Revoke
  deletes the agent (or its token). Rotating a token means issuing a new one and invalidating the
  old one.
- Clients send `Authorization: Bearer <token>`. Requests without a valid token get a 401, before
  any MCP handling.
- **Storage.** A new apps-db migration (`crates/sage-apps/src/db/migrations/0005_…`) adds an
  agents table. Whether agents get their own table or a `sage_apps` row with a new source kind is
  the builder's call. `sage_apps` rows currently require storage and origin rows that only make
  sense for webviews, which argues for a separate table that reuses the grant and wallet-scope JSON
  types.

### Bridge pipeline: callers that aren't webviews

Today the pipeline identifies the caller by webview (`assert_bridge_origin` → `BridgeOrigin` with
a `SharedSageApp`), and delivers approved results by emitting `sage-bridge:response` to that
webview.

Change it so the caller can be an app or an agent:

- Generalize the origin/context the pipeline works with so capability checks, wallet-scope checks,
  `ensure_app_is_enabled_for_scope`, watch-only refusal and approval binding work for both.
- For agents, approved results go back to the waiting MCP request (e.g. a oneshot channel keyed by
  approval id), not to a webview event.
- Webview apps must behave exactly as before. Existing sage-apps tests must keep passing.

How to generalize (a trait, an enum of principals, or giving agents a `SageApp` variant) is the
builder's call. Pick whatever touches the least of the existing pipeline.

### MCP server

- **Library:** `rmcp` (the official Rust MCP SDK), with its streamable-HTTP server transport.
- **Where:** a module inside `sage-apps` (e.g. `crates/sage-apps/src/mcp/`), since it needs the
  crate-private bridge internals and Tauri state.
- **Listener:** `127.0.0.1` only, on its own port (default proposed: 9258, next to RPC's 9257).
  Plain HTTP on loopback; the bearer token is the authentication.
- **Lifecycle:** follows the RPC server's pattern in `src-tauri/src/commands.rs` (`RpcTask`,
  `start_rpc_server`, `stop_rpc_server`, `is_rpc_running`, run-on-startup). The enabled flag and
  port are stored in the apps settings table (`sage_app_settings`), like `auto_update_enabled`.
  Off by default.
- **Tools** are a curated list. Each MCP tool maps to one bridge method, and is listed to an agent
  only if that agent has the method's capability. Calls run through the same pipeline as app
  bridge calls. Tool descriptions come from the capability definitions plus per-tool text.
- **Input schemas.** Bridge param types derive `specta::Type`, not JSON Schema. Add
  `schemars::JsonSchema` to the param types exposed as tools (or hand-write schemas). Builder's
  call.
- **Errors** map bridge error codes (`permission_denied`, `watch_only`, `wallet_changed`,
  `approval_timeout`, …) to MCP tool errors with the code in the message, so agents can react.
- **Concurrency:** don't hold the `Sage` lock across `await`s on approval.

### Settings → Agents (UI)

A new section on the Settings page:

- A toggle to enable the agent server, and its port, plus a running indicator. These follow the
  existing RPC section.
- A list of agents (name, wallet scope, granted capabilities, last used) with Revoke.
- **Add agent:** name → wallet scope (default: the current wallet only) → capabilities (grouped
  from the capability definitions; default: the read-only set) → shows the token once, with copy
  buttons for:
  - Claude Code: `claude mcp add --transport http sage http://127.0.0.1:<port>/mcp --header "Authorization: Bearer <token>"`
  - a generic JSON `mcpServers` entry.
- **Edit agent:** change grants and scope; rotate token.
- Backed by new `apps_*` Tauri commands, following `apps_get_auto_update_enabled` and similar.
  Add them to `main-host.toml`.

### Stage 1 tool set (read-only)

Expose these existing user bridge methods:

| Tool                                                                | Bridge method / capability                    |
| ------------------------------------------------------------------- | --------------------------------------------- |
| `get_wallet`                                                        | `wallet.get_wallet`                           |
| `get_sync_status`                                                   | `wallet.get_sync_status`                      |
| `get_version`                                                       | `wallet.get_version`                          |
| `get_asset_balance`                                                 | `wallet.get_asset_balance`                    |
| `get_asset_coins`                                                   | `wallet.get_asset_coins`                      |
| `get_coins` / `get_coins_by_ids`                                    | `wallet.get_coins`, `wallet.get_coins_by_ids` |
| `get_spendable_coin_count`                                          | `wallet.get_spendable_coin_count`             |
| `get_transactions` / `get_transaction` / `get_pending_transactions` | matching `wallet.*`                           |
| `get_xch_usd_price`                                                 | `wallet.get_xch_usd_price`                    |
| `check_address`                                                     | `wallet.check_address`                        |

Never expose to agents, even if a future UI could grant it: `wallet.get_secret_key`, and any
capability with `accesses_sensitive_secret`. Signing capabilities (`wallet.sign_coin_spends`,
`wallet.sign_message`, `wallet.send_xch*`, `wallet.send_transaction`) are not exposed in stage 1.

**Stage 1 is done when** a user can enable the server, add an agent with read-only grants, paste
the snippet into Claude Code, and have the agent read balances and coins for the scoped wallet. A
wrong or revoked token gets 401, an agent without a capability doesn't see that tool and gets
`permission_denied` if it calls it anyway, and switching to a wallet outside the agent's scope
causes reads to be refused.

### Stage 2 — offers with approval (sketch)

- Add offer bridge methods to `sage-apps`: `wallet.view_offer` (read), `wallet.get_offers`
  (read), `wallet.make_offer`, `wallet.take_offer`, `wallet.cancel_offer`. Each new capability
  gets a definition. The three that sign set `requires_signing`, `binds_approval_to_wallet`, and
  build their approval summary from the offer or spend Sage actually built: what you give, what
  you get, fee, expiry. Sage apps get offer support from the same work.
- `make_offer` must not sign before approval. Today the `make_offer` endpoint builds and signs in
  one call, so stage 2 needs a build-unsigned → approve → sign path.
- Approval UI: the bridge-approval modal is currently shown over app windows
  (`AppModalPresentation`). Agents have no window, so it must also appear over the main wallet UI.
- Tool call blocking vs. client timeouts: a tool call that waits for approval may outlast an MCP
  client's tool timeout. Decide in stage 2 between blocking with an expiry under typical client
  timeouts, or returning a pending approval id plus a `get_approval_status` tool.

### Stage 3 — opt-in autonomy (sketch)

A per-agent policy, edited in Settings → Agents: allowed assets, max value per trade, max per day,
max fee, expiry window. In the pipeline, a signing request that fits the agent's policy is approved
automatically. Everything else goes to the approval queue as usual. Auto-approvals are logged and
visible in the agent's detail view. Off by default, and turning it on is an explicit per-agent
action.

## Scope of the first plan

Plan and build **stage 1 only**. Stages 2 and 3 are sketched for direction, and their open
decisions (e.g. blocking vs. polling approvals) are made when stage 2 is planned. Stage 1 should
generalize the bridge origin without committing to either approach. _(Author's call.)_

## Wallet binding

Agents follow the established apps behavior: they act on the wallet currently open in Sage, and are
refused when it is outside their wallet scope. The Add-agent default scope is
`SelectedWallets { [current] }`, so a user who switches wallets gets refusals rather than an agent
silently reading another wallet. `AllWallets` is available but not the default. Binding an agent to
a wallet that isn't open would need changes to core Sage and is out of scope. _(Author's call.)_

## Notes from code review

- Bridge method names are camelCase (`wallet.getWallet`, …). The `wallet.get_*` names in the table
  above are the capability names.
- `sage-apps` builds only for desktop (`cfg(not(mobile))`), so agents are desktop-only.

## Left to the builder

- Separate agents table vs. `SageApp` variant, and how to generalize the bridge origin.
- `schemars` derives vs. hand-written tool schemas.
- How to run `rmcp`'s HTTP service next to axum 0.7 (own listener, or nested service).
- Exact tool names and descriptions, error-message format, UI layout details within Settings.
- Test layout, following `sage-apps` and `sage-rpc` test conventions.
