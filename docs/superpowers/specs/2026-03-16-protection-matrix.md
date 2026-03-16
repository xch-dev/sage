# Operation Protection Matrix

Analysis of biometric vs. password protection across all protected operations in Sage wallet.

## Architecture

Password enforcement happens at two distinct layers, depending on `auto_submit`:

1. **ConfirmationDialog (centralized)** — When `auto_submit: false` (the default for all UI operations), the Rust `transact()` function ignores the password entirely and returns unsigned coin spends. The user reviews the transaction in `ConfirmationDialog`, whose Submit button calls `requestPassword` → `signCoinSpends` → `submitTransaction`. This is the single enforcement point for all normal UI transaction flows.

2. **Direct `requestPassword` (per-call-site)** — WalletConnect handlers set `auto_submit: true`, which means signing happens inside the Rust command itself. These call `requestPassword` and pass the password to the command directly.

Password and biometric are mutually exclusive. `requestPassword(hasPassword)` routes to password dialog OR biometric prompt, never both. A wallet with a password set never triggers biometric.

## Matrix — UI Operations

Legend: ✅ = protected, 🔄 = redundant double-prompt, ⚠️ = bug, ❌ = not protected

| Operation                                           | Protected | Mechanism                                 | Status | Call site                                                            |
| --------------------------------------------------- | :-------: | ----------------------------------------- | :----: | -------------------------------------------------------------------- |
| **Transaction Operations (via ConfirmationDialog)** |           |                                           |        |                                                                      |
| Send XCH                                            |    ✅     | ConfirmationDialog                        |   OK   | `Send.tsx:188`                                                       |
| Send CAT                                            |    ✅     | ConfirmationDialog                        |   OK   | `Send.tsx:206`                                                       |
| Bulk send XCH                                       |    ✅     | ConfirmationDialog                        |   OK   | `Send.tsx:181`                                                       |
| Bulk send CAT                                       |    ✅     | ConfirmationDialog                        |   OK   | `Send.tsx:198`                                                       |
| Combine coins                                       |    ✅     | ConfirmationDialog (via Token.tsx)        |   OK   | `OwnedCoinsCard.tsx:261`                                             |
| Split coins                                         |    ✅     | ConfirmationDialog (via Token.tsx)        |   OK   | `OwnedCoinsCard.tsx:310`                                             |
| Auto-combine XCH/CAT                                |    ✅     | ConfirmationDialog (via Token.tsx)        |   OK   | `OwnedCoinsCard.tsx:363`                                             |
| Issue CAT                                           |    ✅     | ConfirmationDialog                        |   OK   | `IssueToken.tsx:48`                                                  |
| Multi-send                                          |     —     | No frontend binding                       |  N/A   | Rust-only; no TypeScript binding or UI                               |
| Sign coin spends (Sign button)                      |    ✅     | Direct `requestPassword`                  |   OK   | `ConfirmationDialog.tsx:520`                                         |
| Sign coin spends (Submit button)                    |    ✅     | Direct `requestPassword`                  |   OK   | `ConfirmationDialog.tsx:627`                                         |
| **NFTs / DIDs**                                     |           |                                           |        |                                                                      |
| Bulk mint NFTs                                      |    ✅     | ConfirmationDialog                        |   OK   | `MintNft.tsx:140`                                                    |
| Transfer NFTs                                       |    ✅     | ConfirmationDialog                        |   OK   | `MultiSelectActions.tsx:135`                                         |
| Burn NFTs                                           |    ✅     | ConfirmationDialog                        |   OK   | `MultiSelectActions.tsx:168`                                         |
| Add NFT URI                                         |    ✅     | ConfirmationDialog                        |   OK   | `NftCard.tsx:236`                                                    |
| Assign NFTs to DID                                  |    ✅     | ConfirmationDialog                        |   OK   | `MultiSelectActions.tsx:152`                                         |
| Create DID                                          |    ✅     | ConfirmationDialog                        |   OK   | `CreateProfile.tsx:46`                                               |
| Transfer DIDs                                       |    ✅     | ConfirmationDialog                        |   OK   | `DidList.tsx:166`                                                    |
| Burn DIDs                                           |    ✅     | ConfirmationDialog                        |   OK   | `DidList.tsx:182`                                                    |
| Normalize DIDs                                      |    ✅     | ConfirmationDialog                        |   OK   | `DidList.tsx:198`                                                    |
| **Options**                                         |           |                                           |        |                                                                      |
| Mint option                                         |    ✅     | ConfirmationDialog                        |   OK   | `MintOption.tsx:91`                                                  |
| Transfer options                                    |    ✅     | ConfirmationDialog                        |   OK   | `useOptionActions.tsx:63`                                            |
| Exercise options                                    |    ✅     | ConfirmationDialog                        |   OK   | `useOptionActions.tsx:43`                                            |
| Burn options                                        |    ✅     | ConfirmationDialog                        |   OK   | `useOptionActions.tsx:83`                                            |
| **Clawback**                                        |           |                                           |        |                                                                      |
| Claw back coins                                     |    ✅     | ConfirmationDialog (via Token.tsx)        |   OK   | `ClawbackCoinsCard.tsx:215`                                          |
| Finalize clawback                                   |    ✅     | ConfirmationDialog (via Token.tsx)        |   OK   | `ClawbackCoinsCard.tsx:260`                                          |
| **Offers**                                          |           |                                           |        |                                                                      |
| Make offer (split-NFT path)                         |    ✅     | Direct `requestPassword`                  |   OK   | `useOfferProcessor.ts:116` — password forwarded                      |
| Make offer (single/non-split)                       |    ✅     | Direct `requestPassword`                  |   OK   | `useOfferProcessor.ts:160` — fixed: password now forwarded           |
| Take offer                                          |    ✅     | ConfirmationDialog                        |   OK   | `Offer.tsx` — fixed: removed redundant pre-prompt                    |
| Cancel offer                                        |    ✅     | ConfirmationDialog                        |   OK   | `OfferRowCard.tsx` — fixed: removed redundant pre-prompt             |
| Cancel all offers                                   |    ✅     | ConfirmationDialog                        |   OK   | `Offers.tsx` — fixed: removed redundant pre-prompt                   |
| **Secrets / Key Management**                        |           |                                           |        |                                                                      |
| View mnemonic / secret key                          |    ✅     | Direct `requestPassword`                  |   OK   | `WalletCard.tsx:179`                                                 |
| Delete wallet key                                   |    ✅     | `requestPassword` + `getSecretKey` verify |   OK   | `WalletCard.tsx:82` — password verified via decryption before delete |
| Import key (secret/mnemonic)                        |    ✅     | Password set at import time               |   OK   | Encrypt-at-import only                                               |
| Set / Change / Remove password                      |    ✅     | Inline form (not `requestPassword`)       |   OK   | `Settings.tsx:1238`                                                  |
| **Key Derivation**                                  |           |                                           |        |                                                                      |
| Increase derivation (hardened)                      |    ✅     | Direct `requestPassword`                  |   OK   | `Settings.tsx:1274`                                                  |
| Increase derivation (unhardened)                    |    ❌     | None                                      |   OK   | No private key needed                                                |
| **Other Privileged Actions**                        |           |                                           |        |                                                                      |
| Enable/disable biometric toggle                     |    ✅     | `requestPassword(false)`                  |   OK   | `Settings.tsx:972/989`                                               |
| Start/stop RPC server                               |    ✅     | `requestPassword(false)`                  |   OK   | `Settings.tsx:975/993`                                               |
| **Unprotected (by design)**                         |           |                                           |        |                                                                      |
| View balances / addresses / NFTs                    |    ❌     | None                                      |   OK   | Read-only                                                            |
| Submit pre-signed transaction                       |    ❌     | None                                      |   OK   | No key access needed                                                 |
| Login / logout wallet                               |    ❌     | None                                      |   OK   | No secret access                                                     |
| Rename / resync / emoji                             |    ❌     | None                                      |   OK   | Metadata only                                                        |

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

## Remaining Design Considerations

1. **Session caching asymmetry** — biometric caches for 5 minutes; password never caches.
