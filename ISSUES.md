# Sage Wallet - Open Issues: Summary & Implementation Tasks

**Date:** February 10, 2026 | **Version:** 0.12.8 | **Open Issues:** 10

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

## #726 — Import error messages need more specificity
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

## #723 — Wallet error on single-sided request-only offer
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

## #691 — NFT edition total = 0 should display as infinity
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

## Implementation Priority Matrix

| Issue | Type | Complexity | Impact | Suggested Priority |
|-------|------|-----------|--------|-------------------|
| #723 | Bug | Low | Medium | P1 — Quick fix, prevents user-facing error |
| #726 | UX Bug | Low | Medium | P1 — Improves first-time user experience |
| #691 | Enhancement | Low | Low | P1 — Simple display fix, matches convention |
| #735 | Bug | Medium | High | P1 — Affects many users on upgrade |
| #737 | Enhancement | Medium | High | P2 — Important for power users |
| #704 | Bug | Medium | Medium | P2 — Affects all Android users |
| #575 | Bug | Medium | Low | P3 — Needs reproduction steps first |
| #642 | Enhancement | Medium | Medium | P3 — Partially addressed by PR #714 |
| #729 | Enhancement | High | Low | P3 — Optimization for large wallets |
| #727 | Feature | High | Medium | P4 — Large feature, depends on PR #728 |

---

*File paths are relative to the repo root at `/Users/joshpainter/repos/xch-dev/Sage/`*
