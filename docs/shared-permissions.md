# Shared RPC permissions

Sage Apps and agent transports use `sage-permissions` for caller identities,
capabilities, wallet scopes, permission-policy validation and GUI approval bindings.
The crate has no dependency on Sage Apps, Tauri or HTTP. Existing capability keys,
legacy aliases and wallet-scope serialization are preserved.

## Execution boundary

`sage-rpc::RpcSession` holds the Sage mutex from authorization through endpoint
execution. It does not expose or dereference to the underlying `Sage`. Scoped
sessions deny endpoints without an explicit policy, including wallet management,
settings changes and key enumeration. New RPC endpoints therefore do not become
available to scoped callers automatically.

Each caller has an `App` or `Agent` identity, effective capability grants and a wallet
scope. Adapters must resolve these from trusted host state on every invocation,
including after approval. Request parameters must never supply a caller's identity,
grants or approval. Scope checks happen under the same lock as execution, and explicit
fingerprints cannot select a different wallet.

The Apps adapter resolves current effective grants and creates a scoped session.
Its wallet methods call that session rather than Sage directly. Apps retain their
existing parameter/result shapes, manifest permission handling, installation storage,
webview-origin checks and GUI presentation. Host UI/runtime operations remain in Apps.

Balance and public-key summaries have dedicated RPC session methods: their grants do
not authorize access to the underlying coin or derivation records. Scoped handle and
contact responses remove handle payment history; contact IDs are masked in the RPC
response and omitted by the existing Apps result adapter. Refreshing handle data and
reading contacts require their respective additional capabilities.

## Approvals

The shared approval queue consumes a pending request before execution. A binding
records the caller, reviewed request, expiry and (for wallet operations) the wallet
fingerprint. Apps continue to display and resolve requests using the existing Apps
approval UI and send the result to the originating app.

After the GUI response, the Apps adapter validates the original binding and translates
the reviewed request into its final RPC payload, including any user-selected fee.
RPC checks the binding against that exact method and payload, rechecks current
capabilities and wallet scope, and consumes the session's approval before executing.
An approval does not replace a capability grant. Secret-key access and signing require
approval. The existing automatic-send grant still permits address-based sends;
XCHandles recipients always require review. Watch-only wallets cannot sign.

## HTTP and future agent transports

The existing certificate-authenticated owner listener explicitly uses an owner session
and retains its administrative API. Possession of the existing owner credentials is
not a scoped agent identity.

`make_scoped_router` accepts a trusted `RpcAuthenticator` which validates credentials
and returns current grants for each request. It never falls back to owner access when
authentication fails. The resolver runs under the Sage lock and must not acquire that
lock itself. A token store can implement it without depending on Apps. Request headers
are inputs to authentication, not declarations of identity or permissions.

The scoped HTTP router has no approval presenter and rejects approval-required calls.
An in-process host adapter can use `RpcSession` and the shared approval primitives to
present a GUI request, then execute its approved payload. `ApprovalBinding::new` and
`RpcSession::bind_approved_request` are trusted host APIs, not client RPC endpoints.

This change does not add an MCP server, agent credential management or an agent GUI
approval presenter. Those can be built on the shared layer in a subsequent stage.
