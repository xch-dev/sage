# Operation Protection Matrix

Analysis of biometric vs. password protection across all protected operations in Sage wallet.

## Architecture

Password enforcement happens at two distinct layers, depending on `auto_submit`:

1. **ConfirmationDialog (centralized)** — When `auto_submit: false` (the default for all UI operations), the Rust `transact()` function ignores the password entirely and returns unsigned coin spends. The user reviews the transaction in `ConfirmationDialog`, whose Submit button calls `requestPassword` → `signCoinSpends` → `submitTransaction`. This is the single enforcement point for all normal UI transaction flows.

2. **Direct `requestPassword` (per-call-site)** — WalletConnect handlers set `auto_submit: true`, which means signing happens inside the Rust command itself. These must call `requestPassword` and pass the password to the command directly.

Password and biometric are mutually exclusive. `requestPassword(hasPassword)` routes to password dialog OR biometric prompt, never both. A wallet with a password set never triggers biometric.

## Matrix — UI Operations

Legend: ✅ = protected, 🔄 = redundant double-prompt, ⚠️ = bug, ❌ = not protected

| Operation                                           | Protected | Mechanism                              | Status | Call site                                                                |
| --------------------------------------------------- | :-------: | -------------------------------------- | :----: | ------------------------------------------------------------------------ |
| **Transaction Operations (via ConfirmationDialog)** |           |                                        |        |                                                                          |
| Send XCH                                            |    ✅     | ConfirmationDialog                     |   OK   | `Send.tsx:188`                                                           |
| Send CAT                                            |    ✅     | ConfirmationDialog                     |   OK   | `Send.tsx:206`                                                           |
| Bulk send XCH                                       |    ✅     | ConfirmationDialog                     |   OK   | `Send.tsx:181`                                                           |
| Bulk send CAT                                       |    ✅     | ConfirmationDialog                     |   OK   | `Send.tsx:198`                                                           |
| Combine coins                                       |    ✅     | ConfirmationDialog (via Token.tsx)     |   OK   | `OwnedCoinsCard.tsx:261`                                                 |
| Split coins                                         |    ✅     | ConfirmationDialog (via Token.tsx)     |   OK   | `OwnedCoinsCard.tsx:310`                                                 |
| Auto-combine XCH/CAT                                |    ✅     | ConfirmationDialog (via Token.tsx)     |   OK   | `OwnedCoinsCard.tsx:363`                                                 |
| Issue CAT                                           |    ✅     | ConfirmationDialog                     |   OK   | `IssueToken.tsx:48`                                                      |
| Multi-send                                          |     —     | No UI call site                        |  N/A   | Dead code in bindings                                                    |
| Sign coin spends (Sign button)                      |    ✅     | Direct `requestPassword`               |   OK   | `ConfirmationDialog.tsx:520`                                             |
| Sign coin spends (Submit button)                    |    ✅     | Direct `requestPassword`               |   OK   | `ConfirmationDialog.tsx:627`                                             |
| **NFTs / DIDs**                                     |           |                                        |        |                                                                          |
| Bulk mint NFTs                                      |    ✅     | ConfirmationDialog                     |   OK   | `MintNft.tsx:140`                                                        |
| Transfer NFTs                                       |    ✅     | ConfirmationDialog                     |   OK   | `MultiSelectActions.tsx:135`                                             |
| Burn NFTs                                           |    ✅     | ConfirmationDialog                     |   OK   | `MultiSelectActions.tsx:168`                                             |
| Add NFT URI                                         |     —     | No UI call site                        |  N/A   | Dead code in bindings                                                    |
| Assign NFTs to DID                                  |    ✅     | ConfirmationDialog                     |   OK   | `MultiSelectActions.tsx:152`                                             |
| Create DID                                          |    ✅     | ConfirmationDialog                     |   OK   | `CreateProfile.tsx:46`                                                   |
| Transfer DIDs                                       |    ✅     | ConfirmationDialog                     |   OK   | `DidList.tsx:166`                                                        |
| Burn DIDs                                           |    ✅     | ConfirmationDialog                     |   OK   | `DidList.tsx:182`                                                        |
| Normalize DIDs                                      |    ✅     | ConfirmationDialog                     |   OK   | `DidList.tsx:198`                                                        |
| **Options**                                         |           |                                        |        |                                                                          |
| Mint option                                         |    ✅     | ConfirmationDialog                     |   OK   | `MintOption.tsx:91`                                                      |
| Transfer options                                    |    ✅     | ConfirmationDialog                     |   OK   | `useOptionActions.tsx:63`                                                |
| Exercise options                                    |    ✅     | ConfirmationDialog                     |   OK   | `useOptionActions.tsx:43`                                                |
| Burn options                                        |    ✅     | ConfirmationDialog                     |   OK   | `useOptionActions.tsx:83`                                                |
| **Clawback**                                        |           |                                        |        |                                                                          |
| Claw back coins                                     |    ✅     | ConfirmationDialog (via Token.tsx)     |   OK   | `ClawbackCoinsCard.tsx:215`                                              |
| Finalize clawback                                   |    ✅     | ConfirmationDialog (via Token.tsx)     |   OK   | `ClawbackCoinsCard.tsx:260`                                              |
| **Offers**                                          |           |                                        |        |                                                                          |
| Make offer (split-NFT path)                         |    ✅     | Direct `requestPassword`               |   OK   | `useOfferProcessor.ts:116` — password forwarded                          |
| Make offer (single/non-split)                       |    ✅     | Direct `requestPassword`               | ⚠️ BUG | `useOfferProcessor.ts:160` — password obtained at line 65 but not passed |
| Take offer                                          |    ✅     | `requestPassword` + ConfirmationDialog |   🔄   | `Offer.tsx:103` — double-prompted                                        |
| Cancel offer                                        |    ✅     | `requestPassword` + ConfirmationDialog |   🔄   | `OfferRowCard.tsx:67` — double-prompted                                  |
| Cancel all offers                                   |    ✅     | `requestPassword` + ConfirmationDialog |   🔄   | `Offers.tsx:193` — double-prompted                                       |
| **Secrets / Key Management**                        |           |                                        |        |                                                                          |
| View mnemonic / secret key                          |    ✅     | Direct `requestPassword`               |   OK   | `WalletCard.tsx:179`                                                     |
| Delete wallet key                                   |    ✅     | Direct `requestPassword` (gate)        |   OK   | `WalletCard.tsx:82`                                                      |
| Import key (secret/mnemonic)                        |    ✅     | Password set at import time            |   OK   | Encrypt-at-import only                                                   |
| Set / Change / Remove password                      |    ✅     | Inline form (not `requestPassword`)    |   OK   | `Settings.tsx:1238`                                                      |
| **Key Derivation**                                  |           |                                        |        |                                                                          |
| Increase derivation (hardened)                      |    ✅     | Direct `requestPassword`               |   OK   | `Settings.tsx:1274`                                                      |
| Increase derivation (unhardened)                    |    ❌     | None                                   |   OK   | No private key needed                                                    |
| **Other Privileged Actions**                        |           |                                        |        |                                                                          |
| Enable/disable biometric toggle                     |    ✅     | `requestPassword(false)`               |   OK   | `Settings.tsx:972/989`                                                   |
| Start/stop RPC server                               |    ✅     | `requestPassword(false)`               |   OK   | `Settings.tsx:975/993`                                                   |
| **Unprotected (by design)**                         |           |                                        |        |                                                                          |
| View balances / addresses / NFTs                    |    ❌     | None                                   |   OK   | Read-only                                                                |
| Submit pre-signed transaction                       |    ❌     | None                                   |   OK   | No key access needed                                                     |
| Login / logout wallet                               |    ❌     | None                                   |   OK   | No secret access                                                         |
| Rename / resync / emoji                             |    ❌     | None                                   |   OK   | Metadata only                                                            |

## Matrix — WalletConnect Operations

All WC handlers use `auto_submit: true` (except `signCoinSpends`), so password is required at the call site. All are correctly wired via `HandlerContext.requestPassword`.

| Operation                             | Protected | auto_submit | Status | Call site             |
| ------------------------------------- | :-------: | :---------: | :----: | --------------------- |
| `chia_send` (XCH/CAT)                 |    ✅     |   `true`    |   OK   | `high-level.ts:54/64` |
| `chia_bulkMintNfts`                   |    ✅     |   `true`    |   OK   | `high-level.ts:100`   |
| `chia_createOffer`                    |    ✅     |   not set   |   OK   | `offers.ts:12`        |
| `chia_takeOffer`                      |    ✅     |   `true`    |   OK   | `offers.ts:41`        |
| `chia_cancelOffer`                    |    ✅     |   `true`    |   OK   | `offers.ts:58`        |
| `chip0002_signCoinSpends`             |    ✅     |   `false`   |   OK   | `chip0002.ts:81`      |
| `chip0002_signMessage`                |    ✅     |     N/A     |   OK   | `chip0002.ts:105`     |
| `chia_signMessageByAddress`           |    ✅     |     N/A     |   OK   | `high-level.ts`       |
| WC read-only (connect, chainId, etc.) |    ❌     |     N/A     |   OK   | No signing            |

## Issues

### Bug: `makeOffer` non-split path drops password

- **File:** `src/hooks/useOfferProcessor.ts:160`
- `requestPassword` is correctly called at line 65 and stored in `password`.
- The split-NFT path at line 116 passes `password` to `makeOffer`.
- The single/non-split path at line 160 omits `password` from the call.
- **Impact:** Single-offer creation silently fails on password-protected wallets.

### Redundancy: Offer operations double-prompt

`takeOffer` (`Offer.tsx:103`), `cancelOffer` (`OfferRowCard.tsx:67`), and `cancelOffers` (`Offers.tsx:193`) all call `requestPassword` before the command AND pass the result through `ConfirmationDialog`. Since `auto_submit` is not set, the password passed to the command is ignored by `transact()`. The user gets prompted twice: once before the confirmation dialog opens, then again on Submit.

**Recommendation:** Remove `requestPassword` from these call sites and let `ConfirmationDialog` handle it, matching the pattern used by `Send.tsx`, `IssueToken.tsx`, `CreateProfile.tsx`, and all other transaction pages.

### Fixed: Stale `has_password` in WalletContext

**Root cause of the original "incorrect password" bug.** `WalletContext` fetched `wallet.has_password` once at login. After setting a password in Settings, the global context was never updated, so `ConfirmationDialog` always saw `has_password: false` and called `requestPassword(false)` — which returns `null` immediately on desktop.

**Fix applied:** `Settings.tsx` `WalletSettings` now calls `setGlobalWallet(data.key)` after a successful `changePassword`, keeping the global context in sync.

### Design considerations

1. **`deleteKey` is unspecced** — protected in the UI but not mentioned in the spec.
2. **Biometric toggle auth gap** — calls `requestPassword(false)` unconditionally, so on a password-protected wallet it skips all auth.
3. **Import path is password-only** — biometric is never an option at import time.
4. **Session caching asymmetry** — biometric caches for 5 minutes; password never caches.
5. **Dead code** — `multiSend` and `addNftUri` exist in bindings but have no UI call sites.
