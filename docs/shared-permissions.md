# Shared RPC permissions

Sage Apps and agent transports use `sage-permissions` for caller identities,
wallet capabilities, wallet scopes, permission-policy validation and GUI approval bindings.
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

The dedicated credential listener supplies a host approval presenter for scoped HTTP
writes. A bare scoped router without a presenter rejects approval-required calls.
An in-process host adapter can also use `RpcSession` and the shared approval primitives
to present a GUI request, then execute its approved payload. `ApprovalBinding::new` and
`RpcSession::bind_approved_request` are trusted host APIs, not client RPC endpoints.

The desktop Permissions screen manages scoped service and agent credentials and
presents initial pairing requests. See [Scoped RPC](scoped-rpc.md) for creation,
rotation, revocation and the pairing protocol. The opt-in local HTTPS listener
authenticates bearer tokens against their current stored grants on every request.
This does not add an MCP server. RPC operation confirmation is a separate, one-time
approval bound to the exact parameters, caller, wallet and current network context.

## Apps extends the shared model

`sage-permissions::WalletCapability` contains shared RPC operations and their
security properties. `Caller` accepts only these grants. Apps owns its bridge and system
capabilities, runtime subscriptions, storage permissions, network whitelist, and the
App-specific flags controlling manifest requests and permission presentation.

The Apps capability catalog inherits wallet labels, descriptions and security properties
from the shared definitions, adding its own requestable/grantable/visible policy. The flat
`UserBridgeCapability` representation remains an adapter for existing manifests, saved
grants and generated SDK types; its keys and legacy aliases are unchanged. Before creating
an RPC session, Apps projects effective grants through `wallet_capability()`, excluding
all Apps-only extensions.

Apps validates the combined policy, including its network whitelist and external-access
capabilities, so an inherited secret-access grant cannot be combined with Apps-specific
external access. This rule remains enforced even though network permissions do not enter
the wallet caller's grant set.


## RPC-wide catalog

`RPC_POLICIES` classifies every HTTP endpoint with a grantable capability or an
owner-only restriction, its global/active-wallet/target-wallet/list scope, category,
and confirmation requirement. A coverage test compares the catalog against
`sage-api/endpoints.json`, so new endpoints cannot silently escape classification.
Services and agents use this full catalog. Apps preserves its bridge contract and
projects only the subset it exposes; shared RPC capabilities do not automatically
become callable through the Apps bridge. Administrative HTTP methods remain owner-only.
