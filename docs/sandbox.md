# Sandbox

`pnpm sandbox` runs Sage's UI in a normal browser against a real Sage backend on a simulated chain.
It needs Rust and `pnpm install`, nothing else. Nothing reaches a real network, and each worktree
gets its own ports and data, so several agents or people can run one at the same time.

Use it to check a UI change by hand or with an agent's browser tools: send, receive, make and take
offers, mint, and so on. For automated coverage of backend behavior, use the tests in
`crates/sage-rpc/src/tests/`.

## Starting and stopping

```bash
pnpm sandbox
```

The first run builds the `sage-sandbox` binary, which takes a while. After that a start takes a few
seconds. When everything is ready it prints:

```
  alice  http://alice.localhost:<port>  (bridge http://127.0.0.1:<port>/alice)
  bob    http://bob.localhost:<port>  (bridge http://127.0.0.1:<port>/bob)
```

Ctrl+C stops everything and throws the data away. If a run was killed without a chance to clean up,
the next `pnpm sandbox` stops the leftover processes first.

The run writes `.sandbox/` in the worktree root. It is gitignored:

| File            | Contents                                                                        |
| --------------- | ------------------------------------------------------------------------------- |
| `instance.json` | Each wallet's UI URL, bridge URL, fingerprint, mnemonic, and address; log paths |
| `logs/*.log`    | The backend's and Vite's output                                                 |
| `ports.json`    | The ports from the last run, reused when they are still free                    |
| `run.json`      | Process IDs, so the next start can clean up after a crashed run                 |
| `data/`         | Each wallet's Sage data directory, wiped on every start                         |

Agents should read URLs from `instance.json` rather than parsing terminal output. It only exists
while the sandbox is running.

## What runs

- A `PeerSimulator` from `chia-wallet-sdk`, on testnet11's genesis. It confirms each transaction as
  soon as it's submitted, so there are no blocks to farm and nothing waits in a mempool.
- Two Sage backends, `alice` and `bob`, each logged in to its own wallet and connected only to the
  simulator. Each wallet starts with 100 TXCH in ten 10 TXCH coins. Their mnemonics are fixed, so
  fingerprints and addresses are the same on every run.
- A bridge (`crates/sage-sandbox`) that serves both backends over plain HTTP in place of Tauri's IPC,
  and stands in for the XCHandles API.
- Vite, with `src/sandbox/install.ts` injected ahead of the app. It sends every Tauri command to the
  bridge, forwards sync events to the app, and gives the plugins the UI uses browser stand-ins. Vite
  only injects it in sandbox mode, so it never ships in a build.

## Two wallets

The host name picks the wallet: `alice.localhost` is alice and `bob.localhost` is bob. Browsers
resolve every `*.localhost` name to 127.0.0.1, and the separate host names keep each wallet's local
storage apart.

Bob plays everyone else. To test receiving, an offer being taken, and so on, act as bob, either in
a second tab or by calling bob's bridge directly.

## Calling the bridge

Every command the UI can call is also an HTTP endpoint on the bridge:

```
POST <bridge>/<wallet>/invoke/<command>
```

The body is the arguments object that the UI passes to Tauri's `invoke`. For most commands this is
`{ "req": { ... } }`, with the same request as the matching function in `src/bindings.ts` and the
same snake_case command name as in `TAURI_INVOKE`. Errors come back as `400` with
`{ "kind", "reason" }`. Amounts are in mojos.

```bash
bridge=$(jq -r .bridge .sandbox/instance.json)
alice=$(jq -r .wallets.alice.address .sandbox/instance.json)

# Bob sends alice 5 TXCH. Alice's open UI updates within a second.
curl -s -X POST "$bridge/bob/invoke/send_xch" -H 'content-type: application/json' \
  -d "{\"req\": {\"address\": \"$alice\", \"amount\": \"5000000000000\", \"fee\": \"0\", \"auto_submit\": true}}"

# Bob issues a CAT.
curl -s -X POST "$bridge/bob/invoke/issue_cat" -H 'content-type: application/json' \
  -d '{"req": {"name": "Test", "ticker": "TST", "amount": "1000000", "fee": "0", "auto_submit": true}}'
```

`GET <bridge>/` lists the wallets, and `GET <bridge>/<wallet>/events` streams that wallet's
`sync-event` payloads as server-sent events.

## Inspecting the page

`window.__SAGE_SANDBOX__` has:

| Field         | Contents                                                                    |
| ------------- | --------------------------------------------------------------------------- |
| `wallet`      | The wallet this tab talks to                                                |
| `clipboard`   | The last text the app copied, since a browser clipboard may not be readable |
| `openedUrls`  | URLs the app asked to open externally                                       |
| `unsupported` | Commands that failed because the sandbox has no stand-in for them           |

## What doesn't work

- Mini-apps. They run in native webviews, so the Apps page lists nothing and can't start an app.
  The app logs "Failed to refresh sandbox state" to the console on each load because of this.
- Native features: file dialogs, biometrics, NFC, QR scanning, the share sheet, and window controls.
- The RPC server settings, since starting it would bind the real RPC port.
- CAT names from the token registry, NFT metadata and images from their URIs, and block timestamps.
  The backends run in Sage's test mode, which turns these background lookups off.
- Mempool states. The simulator confirms transactions immediately, so a pending transaction never
  shows as pending.
- Fiat prices still come from the real price APIs.

## Changing the sandbox

- Nothing in `crates/sage-sandbox` hot-reloads. Restart `pnpm sandbox` to pick up a change there.
  Frontend changes, including to `src/sandbox/install.ts`, hot-reload as usual.
- Tauri commands that are `sage-api` endpoints reach the bridge automatically. A command written by
  hand in `src-tauri/src/commands.rs` needs a case in `crates/sage-sandbox/src/bridge.rs`, and a new
  Tauri plugin call needs a stand-in in `src/sandbox/install.ts`.

## Troubleshooting

- The backend and Vite log to `.sandbox/logs/`. The terminal only shows startup progress and
  failures.
- If a port from `ports.json` is taken, the sandbox picks a free one. The URLs change, so read
  `instance.json` again after a restart.
- If a page shows "`<command>` isn't available in the sandbox", the UI called something without a
  stand-in. See [Changing the sandbox](#changing-the-sandbox).
