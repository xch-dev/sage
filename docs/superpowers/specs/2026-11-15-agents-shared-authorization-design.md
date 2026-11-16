# Sage Agents on a shared authorization layer

Reworks PR #1005 in place. Supersedes the authorization and storage parts of
`2026-11-14-sage-agents-mcp-design.md`; the MCP server, token format, settings UI and
security model described there still apply unless this document says otherwise.

## Intent

PR #1005 lets AI agents read a Sage wallet over MCP. It authorized agents by adding a
`BridgeCaller::Agent` to the Sage Apps bridge and stored them in the Apps database.

Review feedback (Hadamcik, PR thread, 2026-11-15): the Apps permission model fits the
requirements, but it should not live in Apps. Agents depending on the Apps bridge forces
everyone who works on Apps to reason about agents. The agreed direction is a shared
authorization layer that RPC, agents and eventually Apps all use.

This rework moves agents onto that layer and takes them out of `sage-apps`.

## Goals

- Agents are authorized by a shared layer that does not depend on `sage-apps`.
- Agent tools call `sage_api` endpoints, the same ones RPC serves, not bridge methods.
- `sage-apps` ends up as it is on `main`: no agent code, no agent storage, no bridge changes.
- Existing RPC clients and the CLI see no change in URLs, request bodies, credentials or
  behaviour.
- The four requirements from the PR thread still hold: per-caller grants, wallet scope,
  per-caller credentials that can be rotated or revoked, and room for GUI approval later.

## Non-goals

- Moving Sage Apps onto the shared layer. That is the next PR; Apps keep their own checks
  until then. Stage 2 signing for agents follows it and reuses the approval flow Apps bring.
- Signing, sending or any write tool. Still stage 2.
- GUI approval. The layer leaves room for it (see Approval) but nothing is built.
- New RPC credentials for third-party RPC callers. Only the owner identity and agents exist.
- Mapping every `sage_api` endpoint to a capability. Only the endpoints agents use.

## Anti-goals

- Any change an existing RPC client could observe.
- An agent reading a wallet outside its scope, including by passing a fingerprint argument.
- A capability check that passes because an endpoint was never mapped. Unmapped means denied
  for every identity except the owner.

## Approaches considered

1. **Keep agents in the Apps bridge** (the PR as submitted). Least work, rejected in review.
2. **Shared types, agents still call bridge methods.** Moves the capability types out of
   Apps but keeps agents running bridge method implementations, so the dependency on Apps
   stays. Answers half the objection.
3. **Shared layer over `sage_api` endpoints, own store** (chosen). Capabilities are declared
   on `sage_api` endpoints, agents call those endpoints, agents have their own database, and
   the existing RPC cert maps to an owner identity. Removes the Apps dependency entirely and
   leaves RPC unchanged for current clients.
4. **HTTP RPC server enforces everything, per-caller credentials for all.** Needs every
   endpoint mapped and a credential migration for existing clients. Too large, and it breaks
   the "RPC unchanged" goal.

Don picked 3, with agents in their own store and the Apps database work reverted.

## Design

### Crates

Two new crates under `crates/`:

- **`sage-auth`**: the shared layer. No Tauri, no rmcp, no dependency on `sage-apps`.
  - `Capability`: what a caller may do (see Capabilities).
  - `WalletScope`: `AllWallets` or `SelectedWallets { fingerprints }`, with `allows`.
  - `Grant`: a set of capabilities plus a wallet scope.
  - `Identity`: `Owner`, or `Grantee(Grant)`. An agent is a grantee; an app will be one too.
  - `authorize(identity, capability, current_fingerprint) -> Result<Decision, AuthError>`:
    the core check. It takes a capability, not an endpoint, so callers whose actions have no
    RPC endpoint (an app signing a message, say) can use it later. `Decision` is a
    non-exhaustive enum with one variant today, `Allowed`. Denials are errors.
  - `endpoint_capability(endpoint_name) -> Option<Capability>` and
    `authorize_endpoint(identity, endpoint_name, current_fingerprint)`: the endpoint map and
    the helper RPC uses on top of the core check.
  - Agent records, token generation and hashing, and the agent store (see Storage).
- **`sage-mcp`**: the MCP server, moved from `crates/sage-apps/src/agents`. Depends on
  `sage-auth`, `sage-api` and `sage`. Desktop only, listed in `src-tauri/Cargo.toml` under
  the same non-mobile target block `sage-apps` uses.

`sage-rpc` gains a dependency on `sage-auth`.

### The check

`authorize` applies these rules in order:

1. `Owner` is always allowed.
2. A grantee must hold the capability. Otherwise `permission_denied`.
3. A wallet must be logged in. Otherwise `no_wallet`.
4. The logged-in wallet must be in the grantee's scope. Otherwise `wallet_not_in_scope`.

`authorize_endpoint` allows `Owner` for every endpoint, mapped or not. For a grantee the
endpoint must have a capability in the map, otherwise `permission_denied`; then `authorize`
runs with that capability.

Error codes keep the names the MCP server returns today.

### Capabilities

Sage Apps move onto this layer in the next PR, so the enum is shaped for them to adopt.

- One `Capability` per mapped endpoint, each with a label and description for the UI. This
  matches the Apps model, where read capabilities are already one per method.
- Variant names and serialized keys are the ones Apps already use: `WalletGetCoins` is
  stored as `"wallet.get_coins"`, that is, `"wallet."` plus the endpoint name. Grants that
  installed apps already hold will parse unchanged when Apps adopt the enum.
- `WalletGetToken` (`"wallet.get_token"`) is new; Apps have no equivalent today.

In this PR the Apps `UserBridgeCapability` enum is left untouched. It carries app-only
capabilities (lifecycle, storage, bridge messaging) and bridge-only reads that arrive in
`sage-auth` with the Apps migration.

### Tools

Each MCP tool names one `sage_api` endpoint. Its capability comes from the endpoint map and
its input schema from the endpoint's request type, using the existing `utoipa` derivation.

| Tool                       | Endpoint                   | Note                                                       |
| -------------------------- | -------------------------- | ---------------------------------------------------------- |
| `get_wallet`               | `get_wallet`               | Takes no arguments; always called with `fingerprint: None` |
| `get_sync_status`          | `get_sync_status`          |                                                            |
| `get_version`              | `get_version`              |                                                            |
| `get_token`                | `get_token`                | Replaces `get_asset_balance`                               |
| `get_coins`                | `get_coins`                | Also covers the removed `get_asset_coins`                  |
| `get_coins_by_ids`         | `get_coins_by_ids`         |                                                            |
| `get_spendable_coin_count` | `get_spendable_coin_count` |                                                            |
| `get_transactions`         | `get_transactions`         |                                                            |
| `get_transaction`          | `get_transaction`          |                                                            |
| `get_pending_transactions` | `get_pending_transactions` |                                                            |
| `get_xch_usd_price`        | `get_xch_usd_price`        |                                                            |
| `check_address`            | `check_address`            |                                                            |

Twelve tools instead of thirteen. `get_asset_balance` and `get_asset_coins` were bridge
methods with no RPC endpoint. `get_token` and `get_coins` are the closest endpoints, and
this is an accepted narrowing for stage 1:

- `get_coins` filters by asset id only, so agents can list XCH and CAT coins but no longer
  DID or NFT coins.
- Coin records no longer carry the puzzle, lineage proof or `locked` flag.
- `get_token` has no spendable coin count (`get_spendable_coin_count` covers it) and returns
  `token: null` for an unknown asset.

Mapping more read endpoints later (`get_nfts`, `get_dids` and so on) is one enum variant and
one table row each, and is left as a follow-up.

`get_wallet` accepts a fingerprint in RPC. The agent tool must not expose it, or an agent
could read a wallet outside its scope.

### Running a tool call

The MCP server's `AgentToolExecutor` trait stays. Its Tauri implementation changes to:

1. Lock `Sage` (the same `Arc<Mutex<Sage>>` RPC uses).
2. Read the current fingerprint from it.
3. Call `sage_auth::authorize` with the agent's identity and the tool's capability.
4. Call the endpoint method on `Sage`.
5. Release the lock.

The scope check and the endpoint run under one lock, so a wallet switch cannot land between
them. The server's earlier check in `call_tool` before the executor runs is kept, so its
tests still cover a real check.

### RPC

`sage-rpc`'s generated endpoint handler calls `authorize_endpoint(&Identity::Owner, endpoint, ..)`
before running the endpoint. Reaching the handler already means the client presented the
wallet cert, so the identity is always `Owner` and the check always passes.

This is a structural change only: it puts RPC on the shared layer so a later PR can add
other RPC identities without touching every handler. No test of existing RPC behaviour
should need to change.

### Storage

Agents and the agent server settings (enabled, port) move to their own SQLite database,
`agents.sqlite`, in the Sage data directory, owned by `sage-auth`. Schema as in
`0005_agents.sql` plus a small settings table.

- The Apps migration `0005_agents.sql` is deleted. It never shipped in a release.
- If the agents database can't be opened, Sage starts normally and the wallet is never
  blocked on it. The store is held as optional state. The status command returns
  `enabled: false`, the default port, and the reason in `start_error` regardless of
  `enabled`. Every other agent command returns an error with that reason. There is no retry
  until restart and no reset flow.
- Agents created on a build of the old PR are not migrated.

### Tauri commands

The agent commands move from `sage-apps` to `src-tauri` and are renamed from `apps_*` to
`agents_*` (for example `agents_create_agent`). `src-tauri/permissions/main-host.toml` is
updated to match, and `src/bindings.ts` is regenerated with `pnpm generate:bindings`.

Deleted wallets are still pruned from agent scopes the way the PR does it today: lazily,
when the agent server starts with Sage and whenever agents are listed. This also covers
wallets deleted over RPC or the CLI, which a hook on the Tauri delete command would miss.

### UI

`AgentSettings` and `AgentPermissionsPanel` stay, and keep using `PermissionEditor` and
`WalletScopeEditor` from `sage-app-ui`. The capability list now comes from `sage-auth`
through `agents_list_agent_capabilities`, in the shape the editor already takes. The
optional label and collapse props added to the editors stay; they are the only remaining
change under Apps, and the main app's dependency on `sage-app-ui` (with the mobile build
changes it needed) stays with them.

### Reverted

Everything the PR changed in `crates/sage-apps` returns to `main`, including:

- `BridgeCaller::Agent`, `process_agent`, and the `tools.sage(&ctx)` refactor across the
  bridge methods.
- `SageAppWalletScope::allows`, `types/agent.rs`, `db/agents.rs`, the settings helpers.
- The `rmcp`, `axum` and `utoipa` dependencies of `sage-apps`.

The Apps database failure work is reverted as well:

- `AppsDbFailure`, `reset_apps_database`, `AppsDb::delete_files`, `is_newer_version_error`.
- `ErrorKind::AppsDatabase` and its mapping in `sage-rpc`.
- `Sage::lock_data_dir` and its test.
- The reset flow in `StartupError.tsx`, its test, and the `ErrorContext` change.
- `AppsDb::initialize` goes back to `expect` on failure, as on `main`.

### Approval (stage 2, not built)

`authorize` returns `Ok(Decision::Allowed)` or an error today. Stage 2 adds a
`Decision::NeedsApproval` variant, decided per capability. `Decision` exists from the start
so adding that variant later does not change every caller's signature.

### Docs

`docs/agents.md` is updated for the new tool list and the fact that agents no longer use
the Apps model. The PR description is rewritten once the code is done.

## Testing

- `sage-auth`: each rule of the check, including owner on an unmapped endpoint, agent on an
  unmapped endpoint, no wallet, out-of-scope wallet; token hashing; store round trips;
  rotation and revocation; scope pruning that does not overwrite a newer edit.
- `sage-mcp`: the existing server tests, ported: auth, rotation and revocation mid-session,
  the token never echoed, Host header checks, schema derivation, a wallet switch outside
  scope refusing calls. Plus: `get_wallet` ignores a supplied fingerprint, and every tool's
  endpoint exists in the map.
- `sage-rpc`: existing tests pass unchanged.
- `sage-apps`: test count returns to what `main` has.
- Front end: existing agent tests updated for the renamed commands and tool list.
- Full validation per `.claude/rules/validation.md`.

## Left to the builder

- Module layout inside the two crates.
- Whether history is tidied with reverts or left as is; the final diff is what matters.
- Exact wording of capability labels and descriptions.
