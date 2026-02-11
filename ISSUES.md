# Sage Wallet - Open Issues: Summary & Implementation Tasks

**Date:** February 10, 2026 | **Version:** 0.12.8 | **Open Issues:** 34

---

## #737 — Show spendable balance
**Opened by:** Rigidity (Feb 10, 2026) | **Type:** Enhancement | **Labels:** None

### Summary
Currently the wallet displays only the total balance. Users need to see their *spendable* balance separately — coins that are not locked in pending transactions, offers, or clawback timeouts.

### Context
The backend already distinguishes between total and spendable balances. `sage-database` has both `xch_balance()` and `selectable_xch_balance()` (and CAT equivalents) in `crates/sage-database/src/tables/coins.rs:130-144`. However, only the total balance is surfaced to the frontend via the `GetSyncStatus` response.

### Tasks
1. **Backend: Add spendable balance to API responses**
   - File: `crates/sage/src/endpoints/data.rs:82-121`
   - Add `spendable_balance` field to `GetSyncStatusResponse` alongside existing `balance`
   - Call `selectable_xch_balance()` and return it in the response
   - File: `crates/sage-api/src/records/` — add `spendable_balance` to relevant record types

2. **Backend: Add spendable balance to token records**
   - File: `crates/sage/src/endpoints/data.rs` (token listing)
   - For each CAT token, calculate `selectable_cat_balance()` and include in `TokenRecord`
   - File: `crates/sage-api/src/records/token.rs` — add `spendable_balance: Amount` field

3. **Frontend: Display spendable balance in wallet overview**
   - File: `src/state.ts` — update `useWalletState` to store spendable balance from sync status
   - File: `src/components/TokenCard.tsx:86-114` — show spendable vs total when they differ
   - File: `src/components/TokenGridView.tsx:134-140` — add spendable balance display

4. **Frontend: Use spendable balance for "Send Max" operations**
   - File: `src/components/selectors/AssetSelector.tsx:177` — use spendable balance for max amount
   - File: `src/pages/Send.tsx` — max button should reference spendable, not total

---

## #735 — App upgrade overwrites user theme selection
**Opened by:** dkackman (Feb 9, 2026) | **Type:** Bug | **Labels:** None

### Summary
Multiple users report that upgrading from v0.12.7 to v0.12.8 causes the theme to revert to light mode. Reproducible on Linux and Android. Windows users who skip the "uninstall" checkbox during upgrade retain their theme. Suggests the issue is tied to clean install behavior rather than in-place upgrade.

### Context
Theme selection is managed by the `theme-o-rama` library via `ThemeProvider` in `App.tsx:155-157`. The `resolveThemeImage()` function in `src/lib/themes.ts:43` uses localStorage for background images, but the **actual theme choice** persistence depends on how `theme-o-rama` is configured. When the app data directory is cleared during upgrade, the theme preference is lost.

### Tasks
1. **Investigate theme persistence mechanism**
   - File: `src/App.tsx:147-169` — check how `ThemeProvider` is initialized and what `defaultTheme`/`storageKey` props are passed
   - File: `node_modules/theme-o-rama/` — check if the library persists to localStorage, config file, or relies on app data
   - Determine: does upgrade on Linux/Android wipe localStorage? Does it wipe the Tauri app data directory?

2. **Add redundant persistence to Rust backend config**
   - File: `crates/sage/src/endpoints/themes.rs` — save selected theme name to `config.toml` when user changes theme
   - File: `crates/sage-config/src/config.rs` — add `theme: Option<String>` to global config
   - This ensures theme survives even if localStorage is cleared

3. **Restore theme on startup from config**
   - File: `src/App.tsx` — on initialization, read saved theme from backend config and pass to `ThemeProvider`
   - File: `src/hooks/useInitialization.ts` — fetch theme preference from backend before rendering

4. **Test across platforms**
   - Verify theme persists through upgrade on Linux, Android, Windows, and macOS
   - Verify both built-in and user/NFT themes are restored correctly

---

## #729 — Reflections about synchronization
**Opened by:** Ganbin (Jan 16, 2026) | **Type:** Discussion/Enhancement | **Labels:** None

### Summary
Proposes two sync optimizations for large wallets:
1. **Resync from a specific block height** — rather than full resync, sync only from block N or the last X blocks
2. **Artificial genesis block** — set a custom starting height to ignore historical transactions (useful after coin consolidation)

Rigidity responded that sync time is correlated with coin count, not block height, and suggested alternatives: disable spent coin sync, resync specific asset IDs, or sync particular coin IDs.

### Tasks
1. **Add selective resync options to sync manager**
   - File: `crates/sage-wallet/src/sync_manager.rs` — add new `SyncCommand` variants:
     - `ResyncAsset(AssetId)` — resync a single asset type
     - `ResyncFromHeight(u32)` — resync only coins created after a given height
   - File: `crates/sage/src/endpoints/keys.rs` (resync endpoint) — add parameters for selective resync

2. **Add option to skip spent coin sync**
   - File: `crates/sage-wallet/src/sync_manager.rs` — add config flag to skip syncing already-spent coins
   - File: `crates/sage-config/src/wallet.rs` — add `skip_spent_coin_sync: bool` config option

3. **Frontend: Expose selective resync in UI**
   - File: `src/components/dialogs/ResyncDialog.tsx` — add options for:
     - Full resync (current behavior)
     - Resync from specific height
     - Resync specific asset
   - File: `src/pages/Settings.tsx` — add option to configure skip-spent-coins behavior

4. **Document sync optimization strategies**
   - Add a section to README or docs explaining sync behavior and optimization options for large wallets

---

## #727 — Deep links as alternative to WalletConnect
**Opened by:** AbandonedLand (Jan 15, 2026) | **Type:** Feature Request | **Labels:** None

### Summary
Proposes implementing deep links (`sage://`) as a simpler alternative to WalletConnect for certain operations. Key proposals:
- `sage://add_offer/<offer_bech32>` — add offer to a cart in the wallet
- `sage://create_offer?offered_asset_id=...&submit_to=<url>` — create an offer with pre-filled fields
- `sage://review_sign_spend_bundle` — review and sign a spend bundle
- A "cart" section in the wallet for batching offer acceptances

### Context
PR #728 (open since Jan 16, 2026) already implements a scheme handler, which is the foundation needed for deep links. The Tauri framework supports custom URI schemes via `tauri-plugin-deep-link`.

### Tasks
1. **Complete and merge PR #728 (scheme handler)**
   - Review and finalize the scheme handler implementation
   - File: `src-tauri/` — Tauri deep link plugin configuration
   - Define the URI scheme prefix: `sage://`

2. **Define deep link protocol specification**
   - Document supported URI patterns and parameters
   - Security considerations: validate all deep link parameters, prevent injection
   - Add CORS-style origin validation for `submit_to` URLs

3. **Implement offer deep link handler**
   - File: `src/App.tsx` — register deep link event listener
   - Parse `sage://add_offer/<bech32>` and route to offer review page
   - Parse `sage://create_offer?...` and pre-fill the MakeOffer form

4. **Implement offer cart (optional, larger scope)**
   - New page: `src/pages/OfferCart.tsx` — batch offer review/acceptance
   - New Zustand store slice for cart state
   - File: `src/state.ts` — add `useOfferCartState`

5. **Security review**
   - All deep link inputs must be validated before processing
   - `submit_to` URLs must be allowlisted or require user confirmation
   - Prevent malicious deep links from auto-submitting transactions

---

## #726 — Import error messages need more specificity ✅ Fixed in PR #740
**Opened by:** AmethystWizard (Jan 14, 2026) | **Type:** UX Bug | **Labels:** None

### Summary
When importing a wallet, the mnemonic input accepts commas but then fails with the unhelpful error: "BIP39 error: mnemonic contains an unknown word (word 0)". User requests:
1. More specific error messages (e.g., "avoid punctuation, capitalization, special characters")
2. Access to the BIP39 word list for reference

### Context
The import flow in `crates/sage/src/endpoints/keys.rs:124-161` calls `Mnemonic::from_str(&req.key)` which returns generic BIP39 errors. The `?` operator converts these to `Error::InvalidKey`, losing the specific BIP39 error context. The frontend in `src/pages/ImportWallet.tsx:82-103` displays whatever error message it receives.

### Tasks
1. **Backend: Improve mnemonic parsing error messages**
   - File: `crates/sage/src/endpoints/keys.rs:151` — replace generic `?` with explicit error mapping:
     ```rust
     let mnemonic = Mnemonic::from_str(&trimmed_key)
         .map_err(|e| Error::InvalidMnemonic(format!("{e}")))?;
     ```
   - File: `crates/sage/src/error.rs` — add `InvalidMnemonic(String)` variant with user-friendly message

2. **Backend: Pre-validate and normalize mnemonic input**
   - File: `crates/sage/src/endpoints/keys.rs` — before parsing, strip punctuation, normalize whitespace, lowercase:
     ```rust
     let normalized = req.key
         .chars()
         .filter(|c| c.is_alphanumeric() || c.is_whitespace())
         .collect::<String>()
         .to_lowercase();
     let words: Vec<&str> = normalized.split_whitespace().collect();
     ```
   - Validate word count (must be 12 or 24) and provide a specific error if wrong
   - Check each word against the BIP39 English wordlist and report which word is invalid

3. **Frontend: Display detailed error and add helper text**
   - File: `src/pages/ImportWallet.tsx` — add input helper text: "Enter your 12 or 24 word seed phrase, separated by spaces"
   - Add a note below the input: "Words are case-insensitive. Do not include commas or other punctuation."
   - Display the specific invalid word when the error occurs

4. **Frontend: Optional BIP39 word list reference** (nice-to-have)
   - Add a collapsible "View valid BIP39 words" section or link to the BIP39 wordlist
   - Or implement autocomplete/suggestion for mnemonic words as user types

---

## #723 — Wallet error on single-sided request-only offer ✅ Fixed in PR #740
**Opened by:** judeallred (Jan 5, 2026) | **Type:** Bug | **Labels:** None

### Summary
Creating an offer with only "request" assets (no offered assets) and zero fee throws a wallet error. Setting a non-zero fee prevents the bug. The error occurs during offer creation in the backend.

### Context
In `crates/sage/src/endpoints/offers.rs:40-183`, the `make_offer()` function processes offered/requested assets but doesn't properly handle the case where offered assets are empty AND fee is zero. A request-only offer with zero fee likely fails because there are no coins to spend (no fee coin, no offered coins), making it impossible to create a valid coin spend.

### Tasks
1. **Backend: Add validation for empty-offer case**
   - File: `crates/sage/src/endpoints/offers.rs` (around line 70-90) — add check:
     ```rust
     if offered_assets.is_empty() && fee_amount == 0 {
         return Err(Error::Api("A fee is required when creating a request-only offer".into()));
     }
     ```
   - This should return a clear user-facing error rather than an internal wallet error

2. **Frontend: Add validation before submission**
   - File: `src/pages/MakeOffer.tsx:73-88` — add a check before calling `commands.createOffer()`:
     - If no offered assets AND fee is 0, show an error message explaining a fee is required
   - Or: auto-set a minimum fee when creating request-only offers

3. **Frontend: Improve error display**
   - File: `src/pages/MakeOffer.tsx` — ensure the error from the backend is displayed clearly rather than as a generic "wallet error"

---

## #704 — Android keyboard closes automatically during token search
**Opened by:** Sunkiller-qc (Oct 30, 2025) | **Type:** Bug | **Labels:** None | **Device:** Pixel 8 Pro, Android 16

### Summary
On Android, when tapping the token search field during offer creation, the keyboard opens briefly then immediately closes, along with the token list. This makes token search impossible on Android.

### Context
The token selector components use `cmdk` (Command palette) wrapped in a Popover. On Android, Popover focus management can conflict with the virtual keyboard — the popover's `onInteractOutside` or `onFocusOutside` handlers may interpret the keyboard appearing as a loss of focus, triggering close. The recent rewrite to shadcn Command component (commit 75ae56f1, PR #733) may have affected this.

### Tasks
1. **Investigate focus behavior on Android**
   - File: `src/components/selectors/AssetSelector.tsx` — check if `platform()` detection is used and whether mobile-specific focus handling exists
   - File: `src/components/ui/command.tsx` — check if the shadcn Command component has mobile focus issues
   - Test: Does the issue occur with the old selector or only the new shadcn Command version?

2. **Fix popover/command focus management for Android**
   - File: `src/components/selectors/AssetSelector.tsx` — add Android-specific focus handling:
     ```tsx
     const isMobile = platform() === 'ios' || platform() === 'android';
     ```
   - Prevent `onInteractOutside` from closing the popover when keyboard appears
   - Add `onOpenAutoFocus` handler to explicitly focus the input after popover animation completes
   - Consider using a Sheet (bottom drawer) instead of Popover on mobile for better keyboard interaction

3. **Add delayed focus for Android**
   - File: `src/components/selectors/AssetSelector.tsx` — add a short delay before focusing the input on Android to let the popover animation complete:
     ```tsx
     useEffect(() => {
       if (open && isMobile) {
         setTimeout(() => inputRef.current?.focus(), 300);
       }
     }, [open]);
     ```

4. **Test on Android emulator and physical device**
   - Verify fix works on Pixel 8 Pro / Android 16 specifically
   - Test on both offer creation and send flows where token search is used

---

## #691 — NFT edition total = 0 should display as infinity ✅ Fixed in PR #740
**Opened by:** DrakoPensulo (Oct 1, 2025) | **Type:** Enhancement | **Labels:** None

### Summary
When an NFT has `edition_total = 0`, neither the edition number nor total displays. The convention (used by MintGarden) is that `edition_total = 0` means infinity (unlimited editions). The wallet should display this appropriately (e.g., "3 of ∞").

### Context
The display logic in both `NftCard.tsx:320` and `Nft.tsx:275` uses:
```tsx
{nft.edition_total != null && nft.edition_total > 1 && (
```
This hides edition info when `edition_total` is 0 or 1. The backend in `data.rs:850-851` passes through the raw `edition_total` value from metadata without interpreting 0 as infinity.

### Tasks
1. **Frontend: Handle edition_total = 0 as infinity**
   - File: `src/components/NftCard.tsx:320-328` — change display logic:
     ```tsx
     {nft.edition_total != null && (nft.edition_total > 1 || nft.edition_total === 0) && (
       <Badge>
         {nft.edition_number} of {nft.edition_total === 0 ? '∞' : nft.edition_total}
       </Badge>
     )}
     ```
   - File: `src/pages/Nft.tsx:275-278` — apply the same change:
     ```tsx
     {nft?.edition_total != null && (nft?.edition_total > 1 || nft?.edition_total === 0) && (
       <LabeledItem
         content={`${nft.edition_number} of ${nft.edition_total === 0 ? '∞' : nft.edition_total}`}
       />
     )}
     ```

2. **Verify i18n compatibility**
   - Ensure the infinity symbol (∞) renders correctly across all platforms (desktop, iOS, Android)
   - Wrap in `<Trans>` if the string needs localization

---

## #642 — Upload offer short codes and support QRs better
**Opened by:** Rigidity (Sep 1, 2025) | **Type:** Bug/Enhancement | **Labels:** bug, ui

### Summary
Improve offer sharing by supporting short codes and better QR code handling. No detailed description provided but the issue implies:
1. Ability to upload offers as short codes (compressed/shortened offer representations)
2. Better QR code scanning and generation for offers

### Context
PR #714 (merged Feb 8, 2026) added QR code image sharing, which partially addresses this. The offer system currently uses full bech32-encoded offer strings which are very long and not QR-friendly.

### Tasks
1. **Implement offer short code generation**
   - File: `crates/sage/src/endpoints/offers.rs` — add endpoint to upload offer to a shortening service and return a short code
   - File: `src/lib/offerUpload.ts` — integrate with Dexie or another offer hosting service to get short URLs

2. **Improve QR code for offers**
   - File: `src/components/` — ensure QR codes use short codes rather than full offer bech32 (which may exceed QR capacity)
   - Add QR scanning that can handle both full offers and short code URLs

3. **Add offer import from short code**
   - File: `src/pages/Offers.tsx` — add input field for pasting short codes
   - Resolve short codes to full offer data via API call

---

## #628 — Webhook for events and database table for transaction results
**Opened by:** Rigidity (Aug 26, 2025) | **Type:** Enhancement | **Labels:** enhancement, rpc

### Summary
Add a webhook system that allows external services to subscribe to wallet events (transaction confirmations, failures, coin state changes, sync events) via configurable HTTP callback URLs. Additionally, introduce a new database table to persist transaction results so RPC consumers can query the outcome of previously submitted transactions.

### Context
Sage emits `SyncEvent` variants (defined in `crates/sage-wallet/src/sync_manager/sync_event.rs`) such as `TransactionUpdated`, `TransactionFailed`, `CoinsUpdated`, etc. These events are consumed internally but there is no mechanism to push them to external consumers over HTTP. The RPC server exposes an axum-based HTTPS API but offers no webhook registration. Transaction results are not stored persistently — the `mempool_items` table only tracks pending transactions and removes them on failure, losing error information. PR #694 ("Webhooks") is already linked to this issue.

### Tasks
1. **Add webhook configuration to `Config`**
   - File: `crates/sage-config/src/config.rs` — Add a `WebhookConfig` struct with fields for `enabled`, `urls`, and `event_filter`

2. **Create a `transaction_results` database table**
   - File: `migrations/` — Add a new migration creating `transaction_results` with columns: `transaction_id`, `status` (pending/confirmed/failed), `error`, `submitted_timestamp`, `confirmed_height`, `confirmed_timestamp`

3. **Add database accessor functions for transaction results**
   - File: `crates/sage-database/src/tables/` — Create `transaction_results.rs` with insert/update/get functions

4. **Persist transaction outcomes in the TransactionQueue**
   - File: `crates/sage-wallet/src/queues/transaction_queue.rs` — Write rows to the new table on `Status::Pending` and `Status::Failed`

5. **Implement the webhook dispatcher**
   - File: `crates/sage-rpc/src/` — Create `webhook.rs` that listens to the `SyncEvent` receiver and POSTs JSON payloads to configured webhook URLs

6. **Add RPC endpoints for webhook management and transaction result queries**
   - File: `crates/sage-api/src/requests/` — Add types for `RegisterWebhook`, `UnregisterWebhook`, `GetTransactionResult`

---

## #626 — Allow custom offer upload endpoints
**Opened by:** Rigidity (Aug 25, 2025) | **Type:** Enhancement | **Labels:** enhancement, offers

### Summary
Allow users to configure custom marketplace endpoints for uploading offers, beyond the currently hardcoded Dexie and MintGarden integrations. Deferred until after UI refactors are completed.

### Context
Offer marketplace upload is hardcoded in `src/lib/marketplaces.ts` (fixed array of Dexie and MintGarden) and `src/lib/offerUpload.ts` (POST to hardcoded API URLs). The `MakeOfferConfirmationDialog` renders marketplace toggle switches from this fixed array. There is no settings UI or persistent configuration for user-defined endpoints.

### Tasks
1. **Define a custom marketplace configuration schema**
   - File: `src/lib/marketplaces.ts` — Extend `MarketplaceConfig` to support a `custom` flag and `uploadUrl` field

2. **Add a settings UI for managing custom endpoints**
   - File: `src/pages/Settings.tsx` — Add a "Custom Offer Endpoints" section for add/edit/remove

3. **Persist custom endpoints in wallet configuration**
   - File: `crates/sage-config/src/wallet.rs` — Add `offer_endpoints: Vec<CustomOfferEndpoint>` field
   - File: `crates/sage-api/src/requests/settings.rs` — Add CRUD request/response types

4. **Create a generic upload function for custom endpoints**
   - File: `src/lib/offerUpload.ts` — Add `uploadToCustomEndpoint(offer, url)` function

5. **Merge custom endpoints into the marketplace list at runtime**
   - File: `src/lib/marketplaces.ts` — Change to a function/hook that merges hardcoded + user-configured endpoints

---

## #619 — Make WalletConnect fees adjustable
**Opened by:** Rigidity (Aug 22, 2025) | **Type:** Enhancement | **Labels:** enhancement, ui, walletconnect

### Summary
WalletConnect commands that create transactions currently use the fee provided by the dApp or default to zero. Users have no UI to review or override the fee before signing. This enhancement adds a fee adjustment dialog to the WalletConnect approval flow.

### Context
In `src/walletconnect/commands/high-level.ts`, `handleSend` passes `params.fee ?? 0` directly to commands with `auto_submit: true`. Similarly for `handleCreateOffer`, `handleTakeOffer`, etc. The `HandlerContext` interface only provides `promptIfEnabled()` for basic auth — no fee adjustment mechanism exists.

### Tasks
1. **Extend HandlerContext with a fee prompt method**
   - File: `src/walletconnect/handler.ts` — Add `promptForFee(suggestedFee): Promise<number>` to `HandlerContext`

2. **Create a fee adjustment dialog component**
   - File: `src/components/WalletConnectFeeDialog.tsx` (new) — Dialog showing dApp-suggested fee with override input

3. **Implement the fee prompt in WalletConnect context**
   - File: `src/contexts/WalletConnectContext.tsx` — Wire dialog to `HandlerContext.promptForFee`

4. **Update transactional command handlers to use adjustable fees**
   - Files: `src/walletconnect/commands/high-level.ts`, `src/walletconnect/commands/offers.ts` — Call `context.promptForFee()` before all transactional commands

5. **Add a default WalletConnect fee setting**
   - File: `src/pages/Settings.tsx` — Add "Default WalletConnect Fee" input in WC settings

---

## #618 — Support NFTs as underlying assets in options
**Opened by:** Rigidity (Aug 22, 2025) | **Type:** Enhancement | **Labels:** enhancement, options

### Summary
The options protocol currently only supports XCH and CAT tokens as underlying/strike assets. This extends it to support NFTs as underlying assets, enabling option contracts where the locked-up asset is an NFT.

### Context
The `OptionType` enum in `chia_wallet_sdk` defines `Xch`, `Cat`, and `RevocableCat` — no `Nft` variant. The `parse_option_asset` in `crates/sage/src/endpoints/transactions.rs` only handles XCH/CAT. The UI `MintOption` page uses `TokenSelector` which only shows fungible tokens.

### Tasks
1. **Extend `OptionType` in chia_wallet_sdk (upstream)**
   - Add an `Nft { launcher_id: Bytes32 }` variant

2. **Update `parse_option_asset` to handle NFT asset IDs**
   - File: `crates/sage/src/endpoints/transactions.rs` — Add NFT branch

3. **Update the MintOption UI to support NFT selection**
   - File: `src/pages/MintOption.tsx` — Add NFT picker alongside token selector

4. **Update the wallet option mint logic for NFT coin selection**
   - File: `crates/sage-wallet/src/wallet/options.rs` — Handle NFT `OptionType` variant

---

## #617 — Support on-demand option mint offers
**Opened by:** Rigidity (Aug 22, 2025) | **Type:** Enhancement | **Labels:** enhancement, offers, options

### Summary
Enable users to create offers that mint an option contract on-demand as part of the offer acceptance flow, rather than requiring the option to be minted first and then offered separately.

### Context
Currently, minting an option and creating an offer are separate operations. The `MakeOffer` flow only lists already-minted options via `OptionSelector`. The `Offered` struct has `options: Vec<Bytes32>` for existing option launcher IDs. There is no mechanism to combine option minting and offer creation into a single atomic spend bundle.

### Tasks
1. **Design the on-demand mint offer data model**
   - File: `crates/sage-api/src/requests/offers.rs` — Add `OptionMintSpec` struct and `offered_option_mints` field to `MakeOffer`

2. **Implement on-demand option minting in offer construction**
   - File: `crates/sage-wallet/src/wallet/offer/make_offer.rs` — Mint options in the same `SpendContext` and include in offered assets

3. **Add "Mint New Option" button in the offer asset selector**
   - File: `src/components/selectors/AssetSelector.tsx` — Inline option parameter entry

4. **Update MakeOffer page to support inline option minting**
   - File: `src/pages/MakeOffer.tsx` — Extend offer state with `optionMints` array

---

## #612 — Initial support for Action System in RPC and WalletConnect
**Opened by:** Rigidity (Aug 20, 2025) | **Type:** Enhancement | **Labels:** enhancement, rpc, walletconnect

### Summary
Sage already has a "create_transaction" action system endpoint that composes multiple actions (send, mint NFT, update NFT, fee) into a single transaction. This issue tracks exposing that action system through WalletConnect so dApps can build arbitrary multi-action transactions.

### Context
The `CreateTransaction` request type in `crates/sage-api/src/requests/action_system.rs` supports an `Action` enum with `Send`, `MintNft`, `UpdateNft`, and `Fee` variants. The RPC server already exposes `create_transaction` as a POST endpoint. However, the WalletConnect layer has no corresponding command.

### Tasks
1. **Define a new WalletConnect command for the action system**
   - File: `src/walletconnect/commands.ts` — Add `chia_createTransaction` command with Zod schema

2. **Implement the WalletConnect handler**
   - File: `src/walletconnect/commands/high-level.ts` — Add `handleCreateTransaction` calling `commands.createTransaction()`
   - File: `src/walletconnect/handler.ts` — Add dispatch case

3. **Consider extending the Action enum**
   - File: `crates/sage-api/src/requests/action_system.rs` — Evaluate adding `CreateDid`, `TransferDid`, `MakeOffer` action types

---

## #587 — Look into support for pool reward claims
**Opened by:** Rigidity (Aug 15, 2025) | **Type:** Enhancement | **Labels:** enhancement

### Summary
Sage currently does not support claiming farming rewards from pooling protocols. Farmers who participate in pools via a "plot NFT" (pool singleton) accumulate rewards that must be claimed through a specific on-chain transaction.

### Context
Sage has no existing code related to pool singletons, plot NFTs, or pool reward claims. The wallet's coin recognition system (`child_kind.rs`, `coin_kind.rs`) has no recognition for pool singleton coins. The sync manager has no awareness of pool-related coin states.

### Tasks
1. **Research the pool protocol and claim mechanism**
   - Investigate CHIP-0007, pool singleton state transitions, and claim mechanics

2. **Evaluate upstream SDK support**
   - Check whether `chia-wallet-sdk` 0.33 provides pool singleton parsing

3. **Add pool singleton coin recognition**
   - File: `crates/sage-wallet/src/child_kind.rs` — Add `PoolSingleton` variant
   - File: `crates/sage-wallet/src/coin_kind.rs` — Add parsing logic

4. **Add database storage for pool state**
   - File: `crates/sage-database/` — Add migration and query support for pool singleton state

5. **Implement pool reward claim transaction construction**
   - File: `crates/sage-wallet/src/wallet/` — Add `pool.rs` module

6. **Expose pool reward claims via API**
   - File: `crates/sage-api/src/requests/` — Add `ClaimPoolRewards` types

---

## #575 — Try from int error
**Opened by:** Rigidity (Aug 11, 2025) | **Type:** Bug | **Labels:** bug

### Summary
A "try from int" conversion error occurs in certain scenarios. The issue contains only a screenshot and no reproduction steps. dkackman asked for a specific offer file or repro steps but received no response.

### Context
This is likely a Rust `TryFrom<i64>` or similar integer conversion error that surfaces when processing certain blockchain data (possibly offer files with unusual amounts). The error could originate in amount parsing, coin value conversion, or database integer handling.

### Tasks
1. **Identify the error location**
   - Search for `try_from` and `TryFrom` usage in the codebase, especially around amount/value conversion
   - File: `crates/sage/src/utils/parse.rs` — check amount parsing for overflow cases
   - File: `crates/sage-database/src/` — check integer casts in database queries

2. **Add proper error handling for integer conversions**
   - Replace any panicking `.try_into().unwrap()` with proper error handling
   - Add bounds checking for amounts before integer conversion
   - Return user-friendly error messages for out-of-range values

3. **Request reproduction steps**
   - Comment on the issue asking for the specific offer file or scenario that triggers this
   - Add logging around integer conversions to capture the failing value when it occurs

---

## #565 — Update Android SDK version for Play Store compliance
**Opened by:** Rigidity (Aug 3, 2025) | **Type:** Build | **Labels:** build, p-high

### Summary
Google Play Store periodically raises the minimum `targetSdkVersion` required for app submissions. This issue tracks updating the Android SDK target version in Sage's build configuration to meet current Play Store compliance requirements. Flagged as high priority because non-compliance blocks publishing updates.

### Context
The Android build configuration is spread across multiple Gradle files. The main app build file at `src-tauri/gen/android/app/build.gradle.kts` currently sets `compileSdk = 35`, `minSdk = 24`, and `targetSdk = 35`. The Tauri plugin has its own build file at `tauri-plugin-sage/android/build.gradle.kts` with `compileSdk = 35` and `minSdk = 24`. The `buildSrc` Gradle plugin at `src-tauri/gen/android/buildSrc/build.gradle.kts` pins Android Gradle Plugin 8.6.0.

### Tasks
1. **Determine the required target SDK level**
   - Check Google Play Store's current requirements. The current `targetSdk = 35` may already satisfy the requirement, in which case AGP and dependencies need updating instead.

2. **Update the main app build configuration**
   - File: `src-tauri/gen/android/app/build.gradle.kts` — update `compileSdk`, `targetSdk`, and potentially `minSdk`. Update dependency versions for compatibility.

3. **Update the root build configuration and AGP**
   - File: `src-tauri/gen/android/build.gradle.kts` — update `com.android.tools.build:gradle` from `8.6.0` to latest stable. Update Kotlin Gradle plugin version from `1.9.25`.
   - File: `src-tauri/gen/android/buildSrc/build.gradle.kts` — update the AGP dependency to match.

4. **Update the Tauri plugin build configuration**
   - File: `tauri-plugin-sage/android/build.gradle.kts` — update `compileSdk` and `minSdk` to match the main app.

5. **Verify build and runtime compatibility**
   - Ensure CI pipeline (`.github/workflows/build.yml`) still builds successfully. Test on devices running the target API level.

---

## #397 — User friendly CLI RPC wrapper
**Opened by:** Rigidity (Apr 7, 2025) | **Type:** Enhancement | **Labels:** enhancement, rpc

### Summary
The current Sage CLI (`sage-cli`) exposes all RPC endpoints but requires users to pass raw JSON strings as arguments. This issue requests a user-friendly wrapper that converts human-readable command-line arguments into the required API types, potentially calls multiple RPC endpoints per command, and formats output in a more readable way than raw JSON.

### Context
The CLI is defined in `crates/sage-cli/src/main.rs`, which declares a `Command` enum with a single `Rpc` variant. The `RpcCommand` is auto-generated via the `impl_endpoints!` macro in `crates/sage-cli/src/rpc.rs`, reading from `endpoints.json` and generating a subcommand for each endpoint where the body is parsed from a raw JSON string. The `sage-client` library (`crates/sage-client/src/lib.rs`) provides a typed HTTP client for the RPC server.

### Tasks
1. **Design the CLI command structure**
   - Determine high-level user-facing commands (e.g., `sage send`, `sage balance`, `sage nfts`, `sage offers`, `sage keys`) with human-readable flags (e.g., `sage send --to xch1... --amount 1.5 --fee 0.001`)

2. **Add new top-level command variants for friendly commands**
   - File: `crates/sage-cli/src/main.rs` — add new variants to `Command` enum alongside `Rpc` (e.g., `Send`, `Balance`, `Wallets`) with clap-derived argument structs

3. **Implement argument-to-API-type conversion**
   - File: `crates/sage-cli/src/` — add modules for each command group converting CLI arguments (decimal XCH, addresses) into API types (mojos, puzzle hashes)

4. **Implement multi-call orchestration**
   - Some commands need multiple RPC calls (e.g., `sage send` might check balance first, then send). Implement using the `sage-client` `Client` struct.

5. **Implement human-readable output formatting**
   - Present balances in XCH rather than mojos, transaction summaries in tabular format. Support `--json` flag for machine-readable output.

6. **Preserve the existing raw RPC subcommand**
   - File: `crates/sage-cli/src/rpc.rs` — keep the existing `sage rpc <endpoint> <json>` interface for advanced users

---

## #390 — iPad in landscape obscures the network fee dialog ✅ Fixed in PR #740
**Opened by:** hoffmang9 (Mar 29, 2025) | **Type:** Bug | **Labels:** bug, ui

### Summary
On iPad in landscape orientation, the on-screen keyboard consumes so much vertical space that it obscures the network fee dialog, making the fee input and submit button inaccessible. A workaround exists (dismiss and re-show the keyboard), but the dialog is not scroll-aware or properly repositioned when the virtual keyboard appears.

### Context
The `FeeOnlyDialog` component in `src/components/FeeOnlyDialog.tsx` uses the `DialogContent` component from `src/components/ui/dialog.tsx`. The `DialogContent` positions itself with `fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%]`, which centers it in the viewport without accounting for the virtual keyboard reducing available height. There is no `max-h-[...]` or `overflow-y-auto` on the dialog content, and no `ScrollArea` wrapping the form.

### Tasks
1. **Add scroll support to DialogContent**
   - File: `src/components/ui/dialog.tsx` — add `max-h-[85vh]` and `overflow-y-auto` to the `DialogContent` class list so the dialog scrolls when viewport is constrained

2. **Adjust dialog vertical positioning for keyboard**
   - File: `src/components/ui/dialog.tsx` — consider changing from `top-[50%] translate-y-[-50%]` to a Flexbox centering strategy that respects the visual viewport, or use `max-height: calc(100vh - env(safe-area-inset-top) - env(safe-area-inset-bottom))`

3. **Ensure FeeOnlyDialog form is scrollable**
   - File: `src/components/FeeOnlyDialog.tsx` — wrap the form in a scrollable container if the generic DialogContent fix is insufficient

4. **Test on iPad landscape and other form-bearing dialogs**
   - Verify fix on iPad in landscape with keyboard active. Also test `TransferDialog`, `AssignNftDialog`, and other form dialogs that may exhibit the same issue.

5. **Consider using the VisualViewport API**
   - Evaluate `window.visualViewport` to detect keyboard presence and dynamically adjust dialog positioning or max-height

---

## #381 — Save coin memos while syncing
**Opened by:** Rigidity (Mar 25, 2025) | **Type:** Enhancement | **Labels:** enhancement, syncing

### Summary
When the wallet syncs coins from the blockchain, it currently discards memo data attached to `CREATE_COIN` conditions. Memos carry user-defined context (payment references, invoice IDs) and are the primary mechanism for communicating metadata between senders and receivers. This feature would persist memos in the database for display in transaction history.

### Context
During sync, `ChildKind::from_parent()` in `crates/sage-wallet/src/child_kind.rs` parses `CREATE_COIN` conditions, and `parse_clawback_unchecked()` already reads `create_coin.memos` for clawback detection. However, general-purpose memo bytes are never stored. The database has no `coin_memos` table. On the sending side, `calculate_memos()` in `crates/sage-wallet/src/wallet/memos.rs` constructs memos for outbound transactions but these aren't round-tripped into storage.

### Tasks
1. **Add a `coin_memos` table to the database schema**
   - File: `crates/sage-database/src/tables/` — create `memos.rs` with `insert_coin_memo(coin_id, memo_bytes)` and `coin_memos(coin_id)`. Add migration with `coin_memos` table (columns: `id`, `coin_id`, `memo BLOB`, `position INTEGER`)

2. **Extract memos from `CREATE_COIN` conditions during sync**
   - File: `crates/sage-wallet/src/child_kind.rs` — propagate raw memo list from each `CreateCoin<NodePtr>`
   - File: `crates/sage-wallet/src/queues/puzzle_queue.rs` — after `insert_puzzle()`, call `insert_coin_memo()` for each extracted memo

3. **Persist memos when inserting locally-created transactions**
   - File: `crates/sage-wallet/src/database.rs` — in `insert_transaction()`, extract memo bytes from the spend context and write them

4. **Expose memos in transaction records**
   - File: `crates/sage-database/src/tables/transactions.rs` — extend `TransactionCoin` to include `memos: Vec<Vec<u8>>` and join on `coin_memos`
   - File: `crates/sage-api/src/records/transaction.rs` — surface memos as hex strings in the API response

5. **Display memos in the UI transaction detail page**
   - File: `src/pages/Transaction.tsx` — render memos (decoded as UTF-8 when valid, hex otherwise)

---

## #327 — New Contacts feature request
**Opened by:** geraldneale (Feb 11, 2025) | **Type:** Enhancement | **Labels:** enhancement

### Summary
Users have no way to store and label frequently-used addresses inside Sage. This requests a Contacts feature where users can save addresses with human-readable names, similar to the Chia reference wallet. Includes shipping a default Sage donation address.

### Context
The Addresses page (`src/pages/Addresses.tsx`) only shows wallet-derived addresses. The Send page (`src/pages/Send.tsx`) requires manually typing or pasting. The config layer (`crates/sage-config/src/config.rs`) has no contacts storage. Since contacts are not wallet-specific and don't need on-chain sync, they are best stored in global config or a small SQLite table.

### Tasks
1. **Add contacts storage to the config or database layer**
   - File: `crates/sage-config/src/config.rs` — add `contacts: Vec<Contact>` to `GlobalConfig` (where `Contact` has `name: String`, `address: String`)

2. **Create API endpoints for CRUD operations on contacts**
   - File: `crates/sage-api/src/requests/` — add `contacts.rs` with `GetContacts`, `AddContact`, `UpdateContact`, `DeleteContact` types
   - File: `crates/sage/src/endpoints/` — implement handlers

3. **Build a Contacts page in the UI**
   - File: `src/pages/Contacts.tsx` (new) — list saved contacts with name, address, copy/edit/delete actions
   - File: `src/components/Nav.tsx` — add "Contacts" NavLink
   - File: `src/App.tsx` — register `/contacts` route

4. **Integrate contacts into the Send flow**
   - File: `src/pages/Send.tsx` — add contact picker or autocomplete to the recipient address field

5. **Ship a default Sage donation contact**
   - Include a pre-populated contact for "Sage Wallet" with address `xch1cfgwg47t6hy8fkxrf2ns759uw7ftcyjlhz3tf8g7d8zslent8pnqzfxaud`

---

## #296 — Address wallet import
**Opened by:** Rigidity (Jan 22, 2025) | **Type:** Enhancement | **Labels:** enhancement

### Summary
Add the ability to import a "cold wallet" using one or more raw XCH/NFT/DID addresses, without requiring any key material. This enables users to monitor balances and transactions for addresses they control via hardware wallets or offline signing setups.

### Context
The `ImportKey` request in `crates/sage-api/src/requests/keys.rs` accepts mnemonic or private key only. The sync system in `puzzle_queue.rs` discovers coins by watching for known `p2_puzzle_hash` values. For address-only import, the wallet needs to register decoded puzzle hashes from provided addresses and subscribe to those puzzle hashes for coin state updates. The keychain already supports `KeyData::Public` for watch-only wallets.

### Tasks
1. **Add an address-based import API endpoint**
   - File: `crates/sage-api/src/requests/keys.rs` — add `ImportAddressWallet` struct with `name`, `addresses: Vec<String>`, `emoji`
   - File: `crates/sage/src/endpoints/` — decode each bech32m address to puzzle hash, insert into `p2_puzzles`, create wallet config entry

2. **Support keyless wallet registration in config and database**
   - File: `crates/sage-config/src/wallet.rs` — extend `WalletConfig` for address-only wallets with no fingerprint
   - File: `crates/sage-database/src/tables/p2_puzzles.rs` — ensure puzzle hashes can be inserted without a public key derivation

3. **Enable coin sync for address-only wallets**
   - File: `crates/sage-wallet/src/sync_manager/wallet_sync.rs` — subscribe to registered puzzle hashes directly instead of deriving from master public key
   - File: `crates/sage-wallet/src/queues/puzzle_queue.rs` — ensure `is_custody_p2_puzzle_hash()` returns true for address-imported hashes

4. **Add UI for address-only import**
   - File: `src/pages/ImportWallet.tsx` — add "Import by Address" mode with multi-address input
   - File: `src/components/WalletCard.tsx` — display "Watch Only" badge for address-only wallets

5. **Restrict signing for address-only wallets**
   - File: `crates/sage-wallet/src/wallet/signing.rs` — return clear error when signing with no key material
   - File: `src/pages/Send.tsx` — disable send button for address-only wallets

---

## #281 — Allow arbitrage in offers
**Opened by:** Rigidity (Jan 17, 2025) | **Type:** Enhancement | **Labels:** enhancement, offers

### Summary
When making offers, the wallet currently selects coins for offered and requested amounts independently. This prevents creating offers where incoming requested assets help fund the offered amounts — blocking arbitrage workflows. The fix is to compute net positions before coin selection.

### Context
The `make_offer()` method in `crates/sage-wallet/src/wallet/offer/make_offer.rs` calls `self.select_spends()` which computes `required_amount = delta.output.saturating_sub(delta.input)` per asset. The `take_offer()` method already calls `offer.arbitrage()` to compute net amounts, suggesting the SDK has arbitrage calculation logic. The issue is applying this netting at offer-creation time.

### Tasks
1. **Refactor coin selection in `make_offer()` for net positions**
   - File: `crates/sage-wallet/src/wallet/offer/make_offer.rs` — before `self.select_spends()`, subtract offered amounts from requested amounts per asset and only select coins for the positive net difference

2. **Add an option to enable/disable arbitrage mode**
   - File: `crates/sage-wallet/src/wallet/offer/make_offer.rs` — add `allow_arbitrage: bool` parameter
   - File: `crates/sage-api/src/requests/offers.rs` — expose `allow_arbitrage` in `MakeOffer`

3. **Validate that arbitrage offers are self-consistent**
   - Ensure net offered amounts are non-negative and the wallet can cover the net difference

4. **Update the Make Offer UI**
   - File: `src/pages/MakeOffer.tsx` — add "Allow arbitrage" toggle for same-asset offers on both sides

5. **Add tests for arbitrage offer creation**
   - Verify coin selection only requires the net difference when assets overlap on both sides

---

## #279 — Sign message in UI and with WalletConnect
**Opened by:** Rigidity (Jan 17, 2025) | **Type:** Enhancement | **Labels:** enhancement, ui, walletconnect | **Milestone:** 0.12.2

### Summary
Sage has backend implementations for signing messages by public key and by address, exposed via WalletConnect. However, there is no in-app UI for users to sign arbitrary messages — useful for proving address ownership. This adds a dedicated "Sign Message" page.

### Context
The backend is fully implemented: `sign_message_with_public_key()` and `sign_message_by_address()` in `crates/sage/src/endpoints/wallet_connect.rs` derive the secret key, hash the message with the `"Chia Signed Message"` domain separator, and return the BLS signature. TypeScript bindings exist (`commands.signMessageWithPublicKey()` and `commands.signMessageByAddress()`). What's missing is a user-facing UI screen.

### Tasks
1. **Create a Sign Message page**
   - File: `src/pages/SignMessage.tsx` (new) — address picker dropdown, message textarea, "Sign" button calling `commands.signMessageByAddress()`, signature display with copy button

2. **Add navigation entry**
   - File: `src/components/Nav.tsx` — add "Sign Message" NavLink near "Addresses"
   - File: `src/App.tsx` — register `/sign-message` route

3. **Support message verification (optional stretch goal)**
   - File: `src/pages/SignMessage.tsx` — add "Verify" tab for pasting address + message + signature
   - File: `crates/sage/src/endpoints/wallet_connect.rs` — add `verify_message()` handler
   - File: `crates/sage-api/src/requests/wallet_connect.rs` — add `VerifyMessage` types

4. **Ensure WalletConnect `signMessageByAddress` response format is correct**
   - File: `src/walletconnect/commands/high-level.ts` — verify `publicKey` and `signature` fields are properly formatted hex

5. **Add sign/verify round-trip test**
   - File: `crates/sage-wallet/src/wallet/signing.rs` — test that signs a message then verifies the signature

---

## #278 — Manual spend bundle signing and submission
**Opened by:** Rigidity (Jan 17, 2025) | **Type:** Enhancement | **Labels:** enhancement, ui

### Summary
Add a dedicated interface for importing, viewing, signing, and submitting unsigned (or partially signed) spend bundles. Currently, signing and submission only happen as part of the transaction confirmation flow within `ConfirmationDialog`. The request is for a standalone tool accessible from the Transactions page that allows users to paste in a raw spend bundle JSON, inspect it, apply their signature, and broadcast it to the mempool.

### Context
The existing signing infrastructure is already well-developed. The backend endpoint `sign_coin_spends` in `crates/sage/src/endpoints/transactions.rs` (line 491) accepts `SignCoinSpends { coin_spends, auto_submit, partial }` and returns a `SignCoinSpendsResponse` with the full `SpendBundleJson`. The `submit_transaction` endpoint (line 523) accepts a `SubmitTransaction { spend_bundle }` and broadcasts it. On the frontend, `src/components/ConfirmationDialog.tsx` already has a "JSON" tab (line 225) with sign and copy-JSON buttons, but it is only accessible from an active transaction flow. The `ViewCoinSpends` endpoint (line 509) can produce a `TransactionSummary` from raw coin spends for display purposes. The `commands.signCoinSpends()` and `commands.submitTransaction()` bindings exist in `src/bindings.ts` (lines 110-117).

### Tasks
1. **Add "Import Spend Bundle" button to Transactions page**
   - File: `src/pages/Transactions.tsx` — Add an "Import Bundle" button (perhaps behind a dropdown or in a secondary toolbar area near the existing transaction options) that opens a new dialog for pasting raw spend bundle JSON
2. **Create SpendBundleDialog component**
   - File: `src/components/dialogs/SpendBundleDialog.tsx` (new) — Build a dialog with: (a) a textarea for pasting JSON, (b) a parsed summary view using `commands.viewCoinSpends()` to render the transaction summary, (c) a "Sign" button calling `commands.signCoinSpends()` with optional `partial: true` support for multi-sig, (d) a "Submit" button calling `commands.submitTransaction()`, (e) a "Copy Signed JSON" button for exporting the signed bundle
3. **Add JSON validation and parsing**
   - File: `src/components/dialogs/SpendBundleDialog.tsx` (new) — Parse the pasted JSON and validate it matches the `SpendBundleJson` schema (with `coin_spends` and `aggregated_signature`) or a bare `CoinSpendJson[]` array. Display clear errors for malformed input. When a full bundle is pasted, pre-populate the signature; when bare coin spends are pasted, leave signature empty until signing
4. **Wire up partial signing support**
   - File: `src/components/dialogs/SpendBundleDialog.tsx` (new) — Add a toggle or checkbox for partial signing (`partial: true`) so multi-signature workflows can each add their signature without failing on unknown public keys. The backend already supports this via `SignCoinSpends.partial` (line 742 of `crates/sage-api/src/requests/transactions.rs`)
5. **Add file import option (optional enhancement)**
   - File: `src/components/dialogs/SpendBundleDialog.tsx` (new) — Add a "Load from file" button that reads a `.json` file from disk using Tauri's file dialog, as an alternative to pasting

---

## #270 — Tangem signer support
**Opened by:** Rigidity (Jan 16, 2025) | **Type:** Enhancement | **Labels:** enhancement

### Summary
Integrate Tangem hardware wallet cards as a signing backend for Sage. Tangem cards are NFC-based hardware wallets that store private keys on a secure element. This would allow users to import a Tangem-backed public key into Sage, sync the wallet normally, and then tap their Tangem card to sign transactions instead of relying on software-stored keys.

### Context
Sage already has NFC NDEF reading infrastructure via the `tauri-plugin-sage` native plugin. The iOS implementation (`tauri-plugin-sage/ios/Sources/SagePlugin.swift`) implements `NFCNDEFReaderSessionDelegate` for reading NFC tags, and the Android implementation (`tauri-plugin-sage/android/src/main/java/SagePlugin.kt`) implements `NfcAdapter.ReaderCallback`. However, these currently only support reading NDEF payloads (used for scanning offer URLs from NFC tags via the `NfcScanDialog` in `src/components/dialogs/NfcScanDialog.tsx`). Tangem requires the Tangem SDK (available for iOS and Android) which communicates with the card via ISO 7816 APDU commands over NFC, not simple NDEF reads. The signing pipeline in `crates/sage/src/utils/spends.rs` (line 14) extracts secrets from the keychain via `self.keychain.extract_secrets(wallet.fingerprint, b"")` and then calls `wallet.sign_transaction()` in `crates/sage-wallet/src/wallet/signing.rs`. For Tangem, the signing would need to be delegated to the card rather than using a local secret key. The keychain (`crates/sage-keychain/src/keychain.rs`) already supports `KeyData::Public` (public-key-only, no secret) which is used for watch-only wallets and would be the appropriate storage type for Tangem-backed keys. Note: PR #720 (open) implements secure-element integration which may share architectural concerns.

### Tasks
1. **Add Tangem SDK native dependencies**
   - File: `tauri-plugin-sage/ios/Sources/SagePlugin.swift` — Add TangemSdk Swift package dependency and implement card session management (scan card, sign hash, read public key)
   - File: `tauri-plugin-sage/android/src/main/java/SagePlugin.kt` — Add TangemSdk Android dependency and implement corresponding card interaction commands
   - File: `tauri-plugin-sage/build.rs` or Cargo/Gradle/SPM configs — Add SDK build dependencies for both platforms
2. **Create Tauri plugin commands for Tangem operations**
   - File: `tauri-plugin-sage/` (plugin commands) — Add new plugin commands: `tangemScanCard` (reads public key and card ID from card), `tangemSign` (sends a list of hashes to the card, returns BLS signatures), and `tangemGetStatus` (checks if Tangem SDK is available on the current platform)
3. **Refactor signing pipeline to support external signers**
   - File: `crates/sage/src/utils/spends.rs` — Refactor the `sign()` method so that when `keychain.has_secret_key(fingerprint)` returns false, it returns the required signature requests (public key + message pairs) instead of attempting local signing. Add a new method like `signing_requests()` that returns the data needed for external signing
   - File: `crates/sage-api/src/requests/transactions.rs` — Add a `GetSigningRequests` request type that returns the required signatures for a set of coin spends without actually signing, and an `ApplySignatures` request type that takes externally produced signatures and aggregates them into the spend bundle
4. **Add Tangem import flow to the frontend**
   - File: `src/pages/ImportWallet.tsx` — Add a "Tangem Card" import option (mobile only, guarded by `platform() === 'ios' || platform() === 'android'`) that triggers NFC card scan, reads the master public key, and imports it as a public-key-only entry via `commands.importKey()` with `save_secrets: false`
5. **Add Tangem signing dialog to the confirmation flow**
   - File: `src/components/ConfirmationDialog.tsx` — Detect when the active wallet has no secret key (Tangem-backed). Instead of the normal software signing flow, show an NFC tap prompt dialog. Call the Tangem plugin sign command with the required hashes, collect the returned signatures, and aggregate them into the spend bundle before submission
6. **Add signer type metadata to wallet config**
   - File: `crates/sage-config/src/wallet.rs` — Add an optional `signer_type: Option<String>` field (values like `"software"`, `"tangem"`) to the `Wallet` struct so the frontend knows how to route signing for each wallet
   - File: `crates/sage-api/src/types/key_info.rs` — Surface signer type in the `KeyInfo` response so the frontend can conditionally render the appropriate signing UI

---

## #252 — Feature Request: Generate multiple offers of a fixed amount
**Opened by:** rkroelin (Jan 10, 2025) | **Type:** Enhancement | **Labels:** enhancement, offers

### Summary
Add the ability to batch-create multiple identical (or incrementally priced) offers at once. For example, a user with 100 XCH who wants to create 10 separate offers of 10 XCH each at a fixed price should be able to do so in a single workflow rather than manually creating each offer one at a time. An optional advanced feature would allow specifying a price delta between successive offers (e.g., starting at a base price and escalating by a fixed increment per offer).

### Context
The current offer creation flow is in `src/pages/MakeOffer.tsx`, which manages a single offer state via `useOfferStateWithDefault()` from `src/hooks/useOfferStateWithDefault.ts`. The confirmation and creation process goes through `MakeOfferConfirmationDialog` (`src/components/dialogs/MakeOfferConfirmationDialog.tsx`) and then `OfferCreationProgressDialog` (`src/components/dialogs/OfferCreationProgressDialog.tsx`), which uses `useOfferProcessor` (`src/hooks/useOfferProcessor.ts`) to execute the actual `commands.makeOffer()` call. On the backend, `MakeOffer` in `crates/sage-api/src/requests/offers.rs` (line 20) creates a single offer with `requested_assets`, `offered_assets`, `fee`, and optional `expires_at_second`. The backend endpoint in `crates/sage/src/endpoints/offers.rs` calls `wallet.make_offer()`. Importantly, the `splitNftOffers` flag in `MakeOffer.tsx` (line 28) already demonstrates a pattern of splitting offers for NFTs, creating multiple offers from one form submission. Each offer locks specific coins, so batch creation requires either pre-splitting coins or sequential offer creation that selects different coins each time. Coin splitting is already supported via `commands.split()` in `src/components/OwnedCoinsCard.tsx`.

### Tasks
1. **Add batch mode toggle to MakeOffer page**
   - File: `src/pages/MakeOffer.tsx` — Add a "Batch Create" switch and a numeric "Number of Offers" input field. When enabled, show the count field. Also add an optional "Price Increment" field for the advanced spread feature. Place these controls near the fee input area (around line 159)
2. **Extend OfferCreationProgressDialog for batch progress**
   - File: `src/components/dialogs/OfferCreationProgressDialog.tsx` — The dialog already handles sequential offer creation for split NFT offers. Extend it to accept a `batchCount` prop and iterate `batchCount` times, calling `commands.makeOffer()` each iteration. Show progress (e.g., "Creating offer 3 of 10"), allow cancellation mid-batch, and display a summary of all created offer IDs at the end
3. **Implement batch offer creation logic in useOfferProcessor**
   - File: `src/hooks/useOfferProcessor.ts` — Extend the processor to support batch mode. For each offer in the batch, use the same offered/requested asset configuration (or with delta applied to the requested amount). After each offer is created, the locked coins are no longer available, so subsequent offers will naturally pick different coins via the wallet's coin selection
4. **Add optional price spread/delta**
   - File: `src/pages/MakeOffer.tsx` — Add an "Escalation per offer" input that modifies the requested token amount for each successive offer. Compute each offer's requested amount as `base_amount + (i * delta)` for offer index `i`. Pass this configuration through to the offer processor
5. **Pre-split coins if needed**
   - File: `src/hooks/useOfferProcessor.ts` — Before batch creation, check if the user has enough discrete coins of sufficient size for N offers. If a single large coin needs to be divided, automatically call `commands.split()` to create N equal parts first, wait for the split transaction to confirm, then proceed with sequential offer creation. Show this as a preliminary step in the progress dialog

---

## #251 — Minor Feature Request: Split coins into nearest fixed size + change
**Opened by:** rkroelin (Jan 10, 2025) | **Type:** Enhancement | **Labels:** enhancement, ui

### Summary
Improve the coin split UX with two changes: (1) add a back button or consolidate the split input and confirmation screens so users can adjust parameters without losing their place and starting over, and (2) add a "target amount" split mode where the user specifies a desired coin denomination (e.g., 10 XCH) and the system calculates how many coins of that size can be created, with any remainder automatically handled as change.

### Context
The current split functionality lives in `src/components/OwnedCoinsCard.tsx`. The split dialog (line 532) uses a form with two fields: `outputCount` (number of output coins, default 2) and `splitFee` (network fee). When submitted, `onSplitSubmit` (line 299) calls `commands.split({ coin_ids, output_count, fee })` and feeds the result into the `ConfirmationDialog`. The backend `Split` struct in `crates/sage-api/src/requests/transactions.rs` (line 101) only accepts `output_count: u32` and `coin_ids`, not a target amount. The backend implementation in `crates/sage/src/endpoints/transactions.rs` (line 107) calls `wallet.split(coin_ids, output_count, fee)`. The user's pain point is the UX flow: after entering the output count and hitting Split, the ConfirmationDialog opens; if the resulting coin sizes are not what was intended, the user must cancel, which closes the dialog entirely, and then re-enter values from scratch. There is no preview of resulting coin sizes before confirmation, and no way to navigate back to adjust parameters.

### Tasks
1. **Add live preview of resulting coin sizes to the split dialog**
   - File: `src/components/OwnedCoinsCard.tsx` — Below the "Output Count" input in the split dialog (around line 555), compute and display a real-time preview of the resulting coin amounts. For each selected coin: `coinAmount / outputCount` with remainder shown separately. Example: "Result: 10 coins of 1.000 XCH + 1 coin of 0.500 XCH (change)". Use the `selectedCoinRecords` state (line 68) and `asset.precision` for formatting
2. **Add "Target Amount" split mode**
   - File: `src/components/OwnedCoinsCard.tsx` — Add a toggle (radio buttons or tabs) between "By Count" (current behavior) and "By Target Amount" modes in the split dialog. In "By Target Amount" mode, replace the output count field with a target coin size input (e.g., "10 XCH"). Auto-calculate `output_count = floor(totalSelectedAmount / targetAmount)` and show the resulting count and change amount in the preview. Update the form schema (`splitFormSchema` at line 284) to include an optional `targetAmount` field
3. **Add back navigation from confirmation to split dialog**
   - File: `src/components/ConfirmationDialog.tsx` — Add an optional `onBack?: () => void` callback prop to the `ConfirmationDialogProps` interface (line 45). When `onBack` is provided, render a "Back" button next to "Cancel" in the dialog footer (around line 605) that calls `onBack()` instead of fully resetting
   - File: `src/components/OwnedCoinsCard.tsx` — In `onSplitSubmit`, instead of directly closing the split dialog and opening the confirmation, keep the split dialog state. Pass an `onBack` prop to `ConfirmationDialog` that sets `response` back to null (hiding confirmation) and re-opens the split dialog with the previous values preserved
4. **Add backend support for target-amount splitting (optional)**
   - File: `crates/sage-api/src/requests/transactions.rs` — Add an optional `target_amount: Option<Amount>` field to the `Split` struct as an alternative to `output_count`
   - File: `crates/sage/src/endpoints/transactions.rs` — In the `split()` method (line 107), when `target_amount` is provided and `output_count` is not (or is 0), calculate the output count server-side from `total_selected_amount / target_amount`. This keeps the frontend calculation honest with a backend validation path

---

## #206 — Password protection
**Opened by:** Rigidity (Dec 29, 2024) | **Type:** Enhancement | **Labels:** enhancement, ui

### Summary
Add optional password protection for sensitive wallet operations. Per the maintainer's clarification in the issue comments, a user-set password would be required for three operations: (1) displaying the mnemonic or secret key, (2) signing transactions or offers, and (3) generating hardened keys. The password would be used to encrypt the stored keys, replacing the current hardcoded empty-password encryption.

### Context
The keychain encryption infrastructure already fully supports user passwords. In `crates/sage-keychain/src/encrypt.rs`, keys are encrypted with AES-256-GCM using a key derived from password + random salt via Argon2 (lines 20-48). The `keychain.rs` methods `add_mnemonic(mnemonic, password)`, `add_secret_key(master_sk, password)`, and `extract_secrets(fingerprint, password)` all accept a `password: &[u8]` parameter. However, every callsite in the application currently passes `b""` (empty bytes) as the password. There are 9 such callsites: `crates/sage/src/utils/spends.rs:15`, `crates/sage/src/endpoints/keys.rs:141,155,330`, `crates/sage/src/endpoints/offers.rs:156,194`, `crates/sage/src/endpoints/wallet_connect.rs:181,220`, and `crates/sage/src/endpoints/actions.rs:201`. On the frontend, the `Login` page (`src/pages/Login.tsx`) does not prompt for a password at all. Mobile platforms already have a biometric prompt via the `useBiometric()` hook (used in `ConfirmationDialog.tsx` line 67 before signing), but desktop has no equivalent protection gate. The `WalletConfig` in `crates/sage-config/src/wallet.rs` does not currently store any password-related configuration. The `CLAUDE.md` project reference (line 94) explicitly notes: "Keychain encryption uses empty password `b""` -- by design, developer plans to add optional user password during wallet setup (infrastructure already supports it)."

### Tasks
1. **Add password state to wallet config**
   - File: `crates/sage-config/src/wallet.rs` — Add a `password_protected: bool` field (default `false`) to the `Wallet` struct (line 24). This tells the app whether a given wallet requires password entry for sensitive operations. No need to store the password itself; the Argon2-encrypted keychain data serves as the verification mechanism (decryption fails with wrong password)
2. **Add set/change/remove password API endpoints**
   - File: `crates/sage-api/src/requests/keys.rs` — Add request types: `SetPassword { fingerprint: u32, password: String }` (for first-time setup), `ChangePassword { fingerprint: u32, old_password: String, new_password: String }`, and `RemovePassword { fingerprint: u32, password: String }`
   - File: `crates/sage/src/endpoints/keys.rs` — Implement password change by: (1) calling `keychain.extract_secrets(fingerprint, old_password)` to decrypt, (2) removing the key, (3) re-adding with `keychain.add_mnemonic(mnemonic, new_password)` or `add_secret_key(sk, new_password)`, (4) updating `wallet_config.password_protected`
3. **Thread password through all signing and secret-access callsites**
   - File: `crates/sage/src/utils/spends.rs` — Change `sign()` to accept a `password: &[u8]` parameter instead of hardcoded `b""` on line 15
   - File: `crates/sage-api/src/requests/transactions.rs` — Add an optional `password: Option<String>` field to `SignCoinSpends`. Consider whether to add it to every transaction request type or use a session-based approach
   - Files: `crates/sage/src/endpoints/offers.rs` (lines 156, 194), `crates/sage/src/endpoints/wallet_connect.rs` (lines 181, 220), `crates/sage/src/endpoints/actions.rs` (line 201), `crates/sage/src/endpoints/keys.rs` (line 330) — Update all 9 callsites that currently use `b""` to accept and pass through the user-provided password (or empty bytes if no password is set)
4. **Create password prompt dialog on the frontend**
   - File: `src/components/dialogs/PasswordDialog.tsx` (new) — Build a reusable modal dialog with a password input field, "Confirm" and "Cancel" buttons. Returns the entered password via a Promise or callback. Should support both single-entry (for signing) and double-entry (for setting new password) modes
5. **Integrate password prompt into the confirmation and signing flow**
   - File: `src/components/ConfirmationDialog.tsx` — Before calling `commands.signCoinSpends()` (around line 519) or `commands.submitTransaction()` (around line 636), check if the wallet has `password_protected: true`. If so, show the `PasswordDialog` and pass the entered password to the API call. This integrates alongside the existing biometric prompt on mobile (line 67)
6. **Add password setup UI in Settings**
   - File: `src/pages/Settings.tsx` — In the `WalletSettings` component (line 1021), add a "Password Protection" section with: a toggle to enable/disable password, a "Set Password" form (password + confirm), and a "Change Password" form (old + new + confirm). Call the new `SetPassword`/`ChangePassword`/`RemovePassword` commands
7. **Gate mnemonic/secret key viewing with password**
   - File: `src/components/WalletCard.tsx` (or wherever `commands.getSecretKey()` is called) — Before displaying the mnemonic or secret key, require the password if password protection is enabled. Pass the password through the `GetSecretKey` request, or validate it separately before allowing the reveal
8. **Implement optional session-based password caching**
   - File: `crates/sage/src/sage.rs` — Add an in-memory password cache with a configurable timeout (e.g., 5 minutes). After the user enters their password once, cache it so subsequent operations within the timeout window do not re-prompt. Clear the cache on logout, app background, or timeout. Store as `Option<(Vec<u8>, Instant)>` in the `Sage` struct, protected by a mutex

---

## #198 — Add option to change how many coins to show
**Opened by:** Rigidity (Dec 26, 2024) | **Type:** Enhancement | **Labels:** enhancement, ui

### Summary
The coin list view currently hard-codes its page size to 10 rows. Users with many coins (common after airdrops or dust attacks) need the ability to choose how many coins are displayed per page, similar to the page-size selector already available in the NFT list.

### Context
`OwnedCoinsCard` (`src/components/OwnedCoinsCard.tsx`, line 77) and `ClawbackCoinsCard` (`src/components/ClawbackCoinsCard.tsx`, line 80) both define `const pageSize = 10;` as a local constant. The `CoinList` component receives `maxRows` and pagination props but has no page-size selector UI. In contrast, the NFT list uses a `Pagination` component (`src/components/Pagination.tsx`) with a `Select` dropdown and configurable `pageSizeOptions`. The backend `GetCoins` API already accepts an arbitrary `limit` parameter, so no backend changes are needed.

### Tasks
1. **Add `pageSize` state and persistence to `OwnedCoinsCard`**
   - File: `src/components/OwnedCoinsCard.tsx` — replace hard-coded `const pageSize = 10;` with `useState` or `useLocalStorage` for persistence

2. **Add `pageSize` state and persistence to `ClawbackCoinsCard`**
   - File: `src/components/ClawbackCoinsCard.tsx` — apply the same change, sharing the same storage key for consistency

3. **Add a page-size selector to `CoinList` or `SimplePagination`**
   - File: `src/components/CoinList.tsx` — add `onPageSizeChange` prop and render a page-size `Select` dropdown. Sensible options: `[10, 25, 50, 100]`. Alternatively, replace `SimplePagination` with the full `Pagination` component.

4. **Thread new props through the component hierarchy**
   - Add `pageSize`, `onPageSizeChange`, `pageSizeOptions` to `CoinListProps` and pass from parent components

5. **Reset to page 0 when page size changes**
   - Add `pageSize` to the `useEffect` dependency arrays that reset `currentPage` on sort parameter changes

---

## #131 — Import OpenRarity rank for NFTs from Mintgarden API
**Opened by:** BrandtH22 (Nov 26, 2024) | **Type:** Enhancement | **Labels:** enhancement, nfts, ui

### Summary
NFT collectors want to see the OpenRarity rank of their NFTs within a collection and sort by that rank. The Mintgarden API provides rarity rank data for NFT collections. This requires fetching rarity data, persisting it locally, and adding a "Sort by Rank" option.

### Context
`NftRecord` in `crates/sage-api/src/records/nft.rs` has no `rank` field. The NFT sort modes are `Name` and `Recent` (defined in `crates/sage-api/src/requests/data.rs` and `src/hooks/useNftParams.ts`). Sage already communicates with the Mintgarden API for offer uploads (`src/lib/offerUpload.ts`), so HTTP client infrastructure exists.

### Tasks
1. **Add `rarity_rank` column to the NFT database table**
   - File: `crates/sage-database/src/tables/assets/nft.rs` — add nullable `rarity_rank INTEGER` column via migration

2. **Add database method to bulk-update rarity ranks**
   - File: `crates/sage-database/src/tables/assets/nft.rs` — add `update_nft_rarity_rank()` method

3. **Expose `rarity_rank` in the API record**
   - File: `crates/sage-api/src/records/nft.rs` — add `pub rarity_rank: Option<u32>` to `NftRecord`

4. **Add `Rank` variant to `NftSortMode`**
   - File: `crates/sage-api/src/requests/data.rs` — add `Rank` to the enum
   - File: `crates/sage-database/src/tables/assets/nft.rs` — add `ORDER BY` clause for rank sorting

5. **Create Mintgarden rarity fetch queue**
   - File: `crates/sage-wallet/src/queues/rarity_queue.rs` (new) — background queue fetching rarity data from Mintgarden API per collection, with rate limiting

6. **Add `Rank` sort option to the frontend**
   - File: `src/hooks/useNftParams.ts` — add `Rank = 'rank'` to enum
   - File: `src/components/NftOptions.tsx` — add sort toggle option for rank

7. **Display rarity rank on NFT cards**
   - File: `src/components/NftCard.tsx` — optionally show `rarity_rank` as a badge when available

---

## #119 — Add manual coin selection option for offers
**Opened by:** Rigidity (Nov 24, 2024) | **Type:** Enhancement | **Labels:** enhancement

### Summary
When creating offers, the wallet automatically selects which coins to lock. This enhancement lets users manually choose specific coins for the offered side, giving control over which coins remain liquid while others are committed.

### Context
The `MakeOffer` page uses `AssetSelector` for token/amount selection but has no coin-picking mechanism. The `Offered` struct in `crates/sage-wallet/src/wallet/offer/make_offer.rs` has `xch: u64` and `cats: IndexMap<Bytes32, u64>` but no coin ID fields. The `OwnedCoinsCard` component already has a full coin-selection UI with checkboxes that could be reused.

### Tasks
1. **Add optional `coin_ids` field to the `Offered` struct**
   - File: `crates/sage-wallet/src/wallet/offer/make_offer.rs` — add `xch_coin_ids: Option<Vec<Bytes32>>` and `cat_coin_ids` fields. Use them directly instead of `self.select_spends()` when provided.

2. **Add `coin_ids` to the API `OfferAmount` type**
   - File: `crates/sage-api/src/requests/offers.rs` — add optional `coin_ids: Option<Vec<String>>` to `OfferAmount`

3. **Thread coin IDs through the endpoint layer**
   - File: `crates/sage/src/endpoints/offers.rs` — parse provided `coin_ids` and include in `Offered` struct

4. **Modify `make_offer` to respect pre-selected coins**
   - Validate coins exist and are spendable, verify total meets required amount, inject into `Spends` directly

5. **Add coin-selection UI to offer creation page**
   - File: `src/components/selectors/AssetSelector.tsx` — add "Select coins" button that opens a coin picker, reusing `CoinList` component
   - File: `src/pages/MakeOffer.tsx` — track selected coin IDs per asset in offer state

---

## #7 — Add support for creating multi-issuance CATs
**Opened by:** Rigidity (Sep 22, 2024) | **Type:** Enhancement | **Labels:** enhancement, cats

### Summary
Sage only supports single-issuance CATs where supply is fixed at creation. Multi-issuance CATs use the `EverythingWithSignatureTail` puzzle, allowing the original issuer to mint additional tokens later by signing with a specific key.

### Context
The `Wallet::issue_cat` method in `crates/sage-wallet/src/wallet/cats.rs` already accepts an `Option<PublicKey>` parameter called `multi_issuance_key`. When `Some`, it uses `EverythingWithSignatureTailArgs` instead of the single-issuance TAIL. However, the endpoint handler in `crates/sage/src/endpoints/transactions.rs` always passes `None`. The API `IssueCat` request has no multi-issuance field, and the frontend `IssueToken` page has no toggle for it.

### Tasks
1. **Add `multi_issuance` flag to the `IssueCat` API request**
   - File: `crates/sage-api/src/requests/transactions.rs` — add `pub multi_issuance: Option<bool>` to `IssueCat`

2. **Pass multi-issuance key through the endpoint layer**
   - File: `crates/sage/src/endpoints/transactions.rs` — check the flag, derive a public key, and pass to `wallet.issue_cat()`. Store the key alongside the asset.

3. **Persist TAIL/issuance key data in the database**
   - File: `crates/sage-database/src/tables/assets/` — add storage for TAIL program hash and public key used for multi-issuance CATs

4. **Add "increase supply" / "re-issue" command**
   - File: `crates/sage-wallet/src/wallet/cats.rs` — add `reissue_cat()` method
   - File: `crates/sage-api/src/requests/transactions.rs` — add `ReissueCat` request type
   - File: `crates/sage/src/endpoints/transactions.rs` — add endpoint handler

5. **Add multi-issuance toggle to Issue Token UI**
   - File: `src/pages/IssueToken.tsx` — add Switch labeled "Multi-issuance (supply can be increased later)"

6. **Add "Increase Supply" UI for existing multi-issuance CATs**
   - File: `src/pages/Token.tsx` — show "Increase Supply" button when asset is multi-issuance
   - File: `crates/sage-api/src/records/token.rs` — add `multi_issuance: bool` to `TokenRecord`

7. **Handle syncing and detection of multi-issuance CATs**
   - File: `crates/sage-wallet/src/queues/cat_queue.rs` — detect `EverythingWithSignatureTail` TAIL type during sync and store the public key

---

## Implementation Priority Matrix

| Issue | Type | Complexity | Impact | Suggested Priority |
|-------|------|-----------|--------|-------------------|
| #723 | Bug | Low | Medium | ✅ Fixed in PR #740 |
| #726 | UX Bug | Low | Medium | ✅ Fixed in PR #740 |
| #691 | Enhancement | Low | Low | ✅ Fixed in PR #740 |
| #735 | Bug | Medium | High | P1 — Affects many users on upgrade |
| #565 | Build | Low | High | P1 — Blocks Play Store updates |
| #390 | Bug | Low | Medium | ✅ Fixed in PR #740 |
| #198 | Enhancement | Low | Medium | P2 — Small UX improvement, reuses existing patterns |
| #279 | Enhancement | Medium | Medium | P2 — Backend ready, needs UI only |
| #737 | Enhancement | Medium | High | P2 — Important for power users |
| #704 | Bug | Medium | Medium | P2 — Affects all Android users |
| #575 | Bug | Medium | Low | P2 — Needs reproduction steps first |
| #206 | Enhancement | High | High | P2 — Core security feature, infra already exists |
| #251 | Enhancement | Low | Medium | P2 — Small UX improvement, high user value |
| #381 | Enhancement | Medium | Medium | P2 — Enables transaction context display |
| #278 | Enhancement | Medium | Medium | P3 — Power-user feature, backend ready |
| #252 | Enhancement | Medium | Medium | P3 — Quality-of-life for active traders |
| #281 | Enhancement | Medium | Medium | P3 — Enables advanced trading workflows |
| #119 | Enhancement | Medium | Medium | P3 — Power-user coin management |
| #327 | Enhancement | Medium | Medium | P3 — Common wallet feature, UX improvement |
| #642 | Enhancement | Medium | Medium | P3 — Partially addressed by PR #714 |
| #729 | Enhancement | High | Low | P3 — Optimization for large wallets |
| #296 | Enhancement | Medium | Medium | P3 — Enables hardware wallet monitoring |
| #619 | Enhancement | Medium | Medium | P3 — WalletConnect UX improvement |
| #612 | Enhancement | Medium | High | P3 — Enables dApp multi-action transactions |
| #397 | Enhancement | Medium | Medium | P3 — Developer experience for CLI users |
| #626 | Enhancement | Medium | Low | P3 — Deferred until after UI refactors |
| #131 | Enhancement | High | Medium | P3 — Requires external API integration |
| #7 | Enhancement | High | Medium | P3 — Low-level wallet feature, SDK support exists |
| #628 | Enhancement | High | High | P4 — Large feature, PR #694 in progress |
| #587 | Enhancement | Very High | Medium | P4 — Major new protocol support |
| #617 | Enhancement | High | Medium | P4 — Complex offer system extension |
| #618 | Enhancement | High | Medium | P4 — Requires upstream SDK changes |
| #270 | Enhancement | Very High | Medium | P4 — Major feature, requires native SDK integration |
| #727 | Feature | High | Medium | P4 — Large feature, depends on PR #728 |

---

*File paths are relative to the repo root at `/Users/joshpainter/repos/xch-dev/Sage/`*
