# Connecting a local service or agent

Desktop Sage provides a separate, opt-in HTTPS listener for native clients on the
same computer. It defaults to `https://127.0.0.1:9258`. The existing owner RPC
listener and its client certificate remain separate and unchanged.

## Create a credential in Sage

1. Open **Settings → Permissions**, then All, External services, or Agents.
2. Choose **Create credential**. Name the connection, choose its type, select
   wallets and RPC permissions, and choose Create. Nothing is granted by default.
3. Save the displayed secret securely. It is shown once, and copied only when you
   choose Copy credential. Clear your clipboard after saving it.
4. Enable the local listener. It starts automatically on future launches while
   enabled. Stop it to block connections without revoking their credentials.
5. Configure the client to trust the **public server certificate** at the path
   shown under Connection instructions. Never share `server.key` or disable TLS
   verification.

The client sends `Authorization: Bearer <secret>` with each RPC request. Never
include the secret in a URL or logs. The token is `sage_rpc_v1_` followed by 64
hexadecimal characters. A caller only has its selected RPC capabilities, and
wallet operations enforce its allowed wallet scope. Most use Sage's active wallet;
`get_wallet` and `get_wallet_address` can explicitly select an allowed wallet.
`get_wallets` returns only allowed wallets. Global reads such as `get_version` do not
require a selected wallet. “All wallets” includes
wallets added later. Transaction and other write capabilities require confirmation for every request.
These credentials cannot administer Sage or export private keys. All 126 HTTP RPC
endpoints have explicit policies: 94 are available to scoped clients and 32 require
owner credentials. Some endpoints share one capability.

Expand a connection's card to see its grants. **Replace credential** keeps the
identity and grants, invalidates the old secret, and shows a new secret once.
**Revoke access** removes the credential. Both affect subsequent requests; a
request already authorized and in progress may finish.

Sage stores SHA-256 token hashes, grants, identities, and listener settings in
`rpc-credentials.json` under the Sage data root. Writes replace the file
atomically; on Unix it is owner-readable/writable only. The dedicated listener
certificate and key are under `scoped-rpc/` in that root.

## Request approval from a native client

This flow authorizes an external client; it does not install a Sage App bundle.
Installed Sage Apps continue to use their existing install confirmation and
bridge, with their own app identity and shared wallet permissions.

With the listener enabled:

1. Generate a cryptographically random secret with at least 32 bytes of entropy.
   Encode it as a string of 32–256 characters. Keep it private to this request.
2. Send `POST /pairing/request` with the SHA-256 hex digest of the UTF-8 secret
   and the desired grant. For example:

   ```json
   {
     "secretHash": "<64-character SHA-256 hex digest>",
     "grant": {
       "name": "My portfolio agent",
       "kind": "agent",
       "capabilities": ["wallet.get_coins", "wallet.get_version"],
       "walletScope": {"kind": "selectedWallets", "fingerprints": [123456789]}
     }
   }
   ```

   `kind` is `agent` or `service`. Wallet scope can instead be
   `{"kind":"allWallets"}`. Only capabilities from the shared RPC catalog are accepted; administrative methods
   cannot be requested or granted.

3. The 202 response contains `id`, `code`, `grant`, and `expiresAt` (Unix seconds).
   Display the code prominently in your client and ask the user to compare it
   with the Sage dialog. The request expires after five minutes.
4. Sage displays the request globally, even outside Settings. The name is
   explicitly unverified. The user must confirm matching codes and can reduce
   the permissions or wallet scope before approving, or reject it. Closing the
   dialog defers it; **Review request** in Permissions reopens it.
5. Poll `POST /pairing/collect` every few seconds with
   `{"id":"<request id>","secret":"<original private secret>"}`.
   Before approval, the response is `{"status":"pending","credential":null}`.
   After approval it is:

   ```json
   {
     "status": "approved",
     "credential": {
       "credential": {"id": "<identity>", "grant": {}, "createdAt": 0},
       "token": "<new bearer secret>"
     }
   }
   ```

   The actual `grant` contains the approved subset and `createdAt` is Unix time.
   Store the token securely and discard the pairing secret. Collection succeeds
   **only once**. Rejection, expiry, invalid secrets and repeat collection return
   an error. If the successful response is lost, the user can revoke that
   connection and pair again.

Pairing requests are memory-only and disappear on restart or when the listener
is stopped. Requests are limited to five per minute and eight outstanding.
Credentials survive restarts. Browser Origin headers and non-local Host headers
are rejected; a hosted website needs a native local connector for this flow.

There is currently no optional-permission declaration or permission-upgrade
endpoint for an existing credential. Pairing covers the initial grant only.
MCP transport is not implemented by this listener.

## Operation confirmation

Requests with a confirmation-required capability (transactions, offers and other
writes) remain pending on their original HTTP connection while Sage shows a separate
**Approve RPC request** dialog. Use a client response timeout longer than five minutes.
The dialog identifies the credential, method, network, wallet and exact parameters.
It currently uses an advanced JSON parameter review, not a decoded transaction preview;
encoded offers and coin spends require an informed review before approval.

The user can approve once, reject, or close the dialog and review it later in Permissions.
Closing does not approve. The request expires after five minutes. A capability grant
never substitutes for this approval. Sage releases the wallet lock while waiting,
then reauthenticates and checks current permissions, scope, network and the exact
approved payload before executing. Revocation, credential rotation, disconnect,
expiry, or stopping the listener cancel pending requests. Execution errors are returned
normally after authorization; an approval does not guarantee a transaction succeeds.

Administrative commands—including login/logout, key import/export/deletion, wallet
creation/deletion, database maintenance, peer changes, and network settings—remain
owner-only. The Permissions screen lists these separately. Adding a new RPC endpoint
without classifying it fails the shared catalog coverage test and is denied at runtime.
