# Password Protection for Sage Wallet

**Issue:** [xch-dev/sage#206](https://github.com/xch-dev/sage/issues/206)
**Date:** 2026-03-15
**Status:** Implemented (backend + frontend password); In Progress (biometric-password bridge)

## Overview

Add opt-in password protection to Sage wallet, requiring authentication for three categories of sensitive operations: displaying secrets, signing transactions/offers, and generating hardened keys. Optionally, users can enable biometric unlock (Touch ID, Face ID, Windows Hello) as a convenience layer that stores the password in the OS keychain.

## Design Decisions

- **Per-operation authentication** — every protected operation prompts for the password. No session caching.
- **Opt-in** — existing wallets continue working without a password. Users can enable protection via "Set Password."
- **Per-key passwords** — each key in the keychain has its own password (or no password). This follows from the existing data model where each `KeyData::Secret` has its own `Encrypted` struct with its own salt. The frontend should use the active wallet's fingerprint to determine which key's password to prompt for.
- **Biometric included in design** — designed as a frontend-only convenience layer on top of the password system.

## Architecture

### Password Is Never Stored (Backend)

The password is a transient input, not persisted state. The existing encryption infrastructure in `sage-keychain` handles everything:

1. At key import (or password set): Argon2 derives a 256-bit AES key from `password + random 32-byte salt`
2. The wallet secret (mnemonic entropy or raw secret key) is encrypted with AES-256-GCM
3. `keys.bin` stores only `{ciphertext, nonce, salt}` — no password, no derived key
4. On each protected operation: user provides password, Argon2 re-derives the key, AES-GCM decrypts. Wrong password fails AES-GCM authentication.
5. Argon2 default parameters provide computational cost that mitigates brute-force attempts against the encrypted data at rest.

### Password Sentinel Convention

The empty byte string `b""` is the "no password" sentinel. This is what existing keys are encrypted with today. The convention is:

- `Option<String>` in request structs: `None` and `Some("")` both map to `b""` at the backend via `req.password.unwrap_or_default().into_bytes()`
- `ChangePassword` uses `String` (not `Option`): an empty `old_password` means the key currently has no password; an empty `new_password` removes password protection

### Has-Password Indicator

Add a `has_password: bool` field to the `KeyInfo` struct returned by `get_key()` / `get_keys()`. Determined by attempting a trial decryption with `b""` at key load time and caching the result, or by adding a `password_protected: bool` field to `KeyData::Secret`.

Preferred approach: add `password_hint: Option<String>` or a simple `password_protected: bool` to `KeyData::Secret`. This avoids trial decryption and is serialized into `keys.bin`. Set to `true` when a non-empty password is used at encryption time. Exposed via `KeyInfo` to the frontend.

### Biometric-Password Bridge (Mobile)

Biometric is a frontend-only concern. The backend never knows whether a password came from typing or biometric retrieval.

**Plugin:** `tauri-plugin-keychain` — wraps iOS Keychain and Android AccountManager. Key-value API: `saveItem(key, password)`, `getItem(key)`, `removeItem(key)`. Supports multiple entries per app (one per wallet).

**Keychain key format:** `sage-password-{fingerprint}` — one entry per wallet.

**Global setting:** Biometric unlock is a single global toggle (the existing `useLocalStorage('biometric', false)` flag). When enabled, it applies to all wallets that have a password. No per-wallet configuration needed.

**Store-on-first-use:** Passwords are not stored in the keychain at enable time. Instead, the first time a password-protected operation occurs for a given wallet after biometric is enabled, the user types their password normally. On successful backend operation, the password is stored in the keychain. Subsequent operations for that wallet use biometric retrieval.

**Retrieval flow:**

1. `requestPassword(hasPassword)` is called at a protected operation call site
2. If `hasPassword` is false and biometric not enabled → return `null` (no auth needed)
3. If `hasPassword` is false and biometric enabled → biometric prompt (standalone gate), return `null` on success, `undefined` on failure
4. If `hasPassword` is true and biometric enabled → attempt `retrieve(sage-password-{fingerprint})` from OS keychain (triggers biometric prompt)
5. If retrieval succeeds → return the stored password (no dialog shown)
6. If retrieval fails (no entry, user cancels, enrollment changed, lockout) → fall back to password dialog
7. If `hasPassword` is true and biometric not enabled → show password dialog

**The OS keychain uses the platform's secure element** (Secure Enclave on iOS, TEE on Android) without Sage needing to manage SE keys directly.

> **TODO:** Add macOS Touch ID + Keychain and Windows Hello + Credential Manager support for desktop platforms.

## Protected Operations

There are 7 code points where `b""` is passed to `extract_secrets` or `add_mnemonic`/`add_secret_key`, plus 2 encrypt-at-creation sites. However, because `sign()` is called through `transact()` and `transact_with()`, the password must flow through a much larger API surface.

### 1. Display mnemonic/secret key (1 site)

| Call site                               | Function           |
| --------------------------------------- | ------------------ |
| `crates/sage/src/endpoints/keys.rs:353` | `get_secret_key()` |

### 2. Sign transactions and offers

The central signing path is:

```
endpoint method → transact() / transact_with() → sign() → extract_secrets()
```

**Direct `extract_secrets` call sites** (5 sites):

| Call site                                         | Function                                                        |
| ------------------------------------------------- | --------------------------------------------------------------- |
| `crates/sage/src/utils/spends.rs:15`              | `sign()` — called by `transact_with()` and `sign_coin_spends()` |
| `crates/sage/src/endpoints/offers.rs:170`         | `make_offer()` — calls `extract_secrets` directly               |
| `crates/sage/src/endpoints/offers.rs:208`         | `take_offer()` — calls `extract_secrets` directly               |
| `crates/sage/src/endpoints/wallet_connect.rs:181` | `sign_message_with_public_key()`                                |
| `crates/sage/src/endpoints/wallet_connect.rs:220` | `sign_message_by_address()`                                     |

**Transaction endpoints that flow through `transact()` → `sign()`** (21 endpoints):

`send_xch`, `bulk_send_xch`, `combine`, `auto_combine_xch`, `split`, `auto_combine_cat`, `issue_cat`, `send_cat`, `bulk_send_cat`, `multi_send`, `create_did`, `bulk_mint_nfts`, `transfer_nfts`, `add_nft_uri`, `assign_nfts_to_did`, `transfer_dids`, `normalize_dids`, `mint_option`, `transfer_options`, `exercise_options`, `finalize_clawback`

Plus `cancel_offer`, `cancel_offers`, and `create_transaction` (action system) which also flow through `transact()` / `transact_with()`.

### 3. Generate hardened keys (1 site)

| Call site                                  | Function                      |
| ------------------------------------------ | ----------------------------- |
| `crates/sage/src/endpoints/actions.rs:201` | `increase_derivation_index()` |

### 4. Key import — encrypt at creation (2 sites)

| Call site                               | Function                         |
| --------------------------------------- | -------------------------------- |
| `crates/sage/src/endpoints/keys.rs:141` | `import_key()` — secret key path |
| `crates/sage/src/endpoints/keys.rs:178` | `import_key()` — mnemonic path   |

Note: `import_key()` also generates hardened derivations using the in-memory master key during import. This does NOT need the password since the key is already decrypted at that point.

## Changes

### `sage-keychain` crate

**`keychain.rs`** — Add one new method:

```rust
pub fn change_password(
    &mut self,
    fingerprint: u32,
    old_password: &[u8],
    new_password: &[u8],
) -> Result<(), KeychainError>
```

Decrypts with old password, re-encrypts with new password, replaces the `KeyData::Secret` entry.

**`key_data.rs`** — Add `password_protected: bool` to `KeyData::Secret`:

```rust
Secret {
    master_pk: [u8; 48],
    entropy: bool,
    encrypted: Encrypted,
    password_protected: bool,  // new
}
```

Note: this changes the `keys.bin` serialization format. Existing files will fail to deserialize. Handle with a versioned deserialization fallback: try deserializing the new format first, fall back to the old format (defaulting `password_protected` to `false`).

### `sage-api` crate (request structs)

Add `password: Option<String>` to **all request structs that trigger signing, secret access, or key import**:

**Direct secret access:**

- `ImportKey`
- `GetSecretKey`

**Signing via `transact()` path — all transaction request structs:**

- `SendXch`, `BulkSendXch`, `Combine`, `AutoCombineXch`, `Split`, `AutoCombineCat`, `IssueCat`, `SendCat`, `BulkSendCat`, `MultiSend`, `CreateDid`, `BulkMintNfts`, `TransferNfts`, `AddNftUri`, `AssignNftsToDid`, `TransferDids`, `NormalizeDids`, `MintOption`, `TransferOptions`, `ExerciseOptions`, `FinalizeClawback`

**Signing via direct `extract_secrets` or `sign()`:**

- `SignCoinSpends`, `MakeOffer`, `TakeOffer`, `CancelOffer`, `CancelOffers`, `CreateTransaction`

**Hardened derivation:**

- `IncreaseDerivationIndex`

**WalletConnect signing:**

- `SignMessageWithPublicKey`, `SignMessageByAddress`

**New request/response pair:**

- `ChangePassword { fingerprint: u32, old_password: String, new_password: String }`
- `ChangePasswordResponse {}`

**`KeyInfo`** — add `has_password: bool` field.

### `sage` crate (endpoints)

**`spends.rs`**: `sign()` takes `password: &[u8]` parameter, passes to `extract_secrets`.

**`transactions.rs`**: `transact()` and `transact_with()` take `password: &[u8]` parameter, pass to `sign()`. Every transaction endpoint extracts password from its request struct via `req.password.unwrap_or_default().into_bytes()` and passes to `transact()`.

**`keys.rs`**: `import_key()` passes password to `add_mnemonic()`/`add_secret_key()`. `get_secret_key()` passes password to `extract_secrets()`. `get_key()`/`get_keys()` populate `has_password` from `KeyData`.

**`offers.rs`**: `make_offer()`, `take_offer()` pass password to `extract_secrets()`. `cancel_offer()`, `cancel_offers()` pass password to `transact()`.

**`actions.rs`**: `increase_derivation_index()` passes password to `extract_secrets()`.

**`wallet_connect.rs`**: Both signing methods pass password to `extract_secrets()`.

New `change_password()` endpoint.

### Frontend (TypeScript/React)

#### PasswordContext (`src/contexts/PasswordContext.tsx`)

A React context provider that serves as the **single entry point for all operation authentication** — password, biometric, or both. Provides:

```typescript
requestPassword(hasPassword: boolean): Promise<string | null | undefined>
```

**Return values:**

- `string` → use this password (typed by user or retrieved from keychain via biometric)
- `null` → no password needed, all auth passed (biometric gate passed or no auth required)
- `undefined` → auth cancelled or failed, abort the operation

**Internal decision tree:**

```text
hasPassword=false, biometric not enabled  → return null (no auth needed)
hasPassword=false, biometric enabled      → biometric prompt, return null on success, undefined on fail
hasPassword=true,  biometric enabled      → try keychain retrieval, fall back to dialog on failure
hasPassword=true,  biometric not enabled  → show password dialog
cancelled at any point                    → return undefined
```

On desktop (no keychain available), the biometric path is skipped — behaves as if biometric is not enabled.

Uses a `useRef`-based pending promise pattern to bridge the dialog UI with the async call site.

**Provider placement:** Inside `I18nProvider` and `WalletProvider` (required because `PasswordProvider` reads `useWallet()` for the fingerprint and `useBiometric()` for the enabled state). Wraps `WalletConnectProvider` and all downstream providers, so `usePassword()` is available everywhere.

Provider tree: `BiometricProvider` → `I18nProvider` → `WalletProvider` → `PasswordProvider` → `PeerProvider` → `WalletConnectProvider` → `PriceProvider` → `RouterProvider`

#### PasswordDialog (`src/components/dialogs/PasswordDialog.tsx`)

A reusable modal dialog rendered by `PasswordProvider`. Features:

- Auto-focuses the password input on open
- Clears password state on open/close
- Supports Enter key to submit
- Cancel closes the dialog and resolves the promise with `null`

#### usePassword hook (`src/hooks/usePassword.ts`)

Thin wrapper around `PasswordContext` with a guard that throws if used outside `PasswordProvider`.

#### Call site pattern

Every protected operation follows the same unified pattern — a single call that handles password, biometric, or both:

```typescript
const password = await requestPassword(wallet?.has_password ?? false);
if (password === undefined) return; // auth cancelled or failed
```

The separate `promptIfEnabled()` biometric call is removed from all call sites. `requestPassword` is now the sole auth gate. Password is then passed to the backend command. Call sites that were updated:

| File                       | Operations                                                                                                                 |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `ConfirmationDialog.tsx`   | `signCoinSpends` (Sign Transaction button, Submit button)                                                                  |
| `WalletCard.tsx`           | `getSecretKey` (View Details dialog)                                                                                       |
| `Settings.tsx`             | `increaseDerivationIndex` (when hardened keys enabled)                                                                     |
| `Offers.tsx`               | `cancelOffers` (Cancel All Active)                                                                                         |
| `OfferRowCard.tsx`         | `cancelOffer` (individual offer cancel)                                                                                    |
| `useOfferProcessor.ts`     | `makeOffer` (create offer flow)                                                                                            |
| `Offer.tsx`                | `takeOffer` (take offer flow)                                                                                              |
| `WalletConnectContext.tsx` | All WC command handling via `HandlerContext`                                                                               |
| WalletConnect commands     | `signCoinSpends`, `signMessage`, `signMessageByAddress`, `send`, `createOffer`, `takeOffer`, `cancelOffer`, `bulkMintNfts` |

#### WalletConnect integration

The `HandlerContext` interface was extended with `requestPassword` and `hasPassword`. WalletConnect command handlers prompt for the password before executing protected operations, using the same pattern as direct UI call sites.

#### Password management in Settings

A new **Security** section in Wallet Settings (only shown for hot wallets with `has_secrets`):

- **Set Password** — shown when `has_password` is `false`. Opens a dialog with New Password + Confirm Password fields.
- **Change Password** — shown when `has_password` is `true`. Opens a dialog with Current Password + New Password + Confirm Password fields.
- **Remove Password** — shown when `has_password` is `true`. Opens a dialog with Current Password field. Uses destructive button variant.

All three operations call `commands.changePassword()` with appropriate `old_password`/`new_password` values (empty string = no password). On success, refreshes `KeyInfo` via `commands.getKey()` and shows a success toast.

#### Error feedback

Wrong password errors (`ErrorKind::Unauthorized` with reason containing "decrypt") are surfaced as a toast notification "Incorrect password" via the global `ErrorContext.addError` handler. This provides consistent feedback across all password-protected operations without requiring per-call-site error handling. Other unauthorized errors (e.g., wallet transition race conditions) continue to be silently discarded.

#### Keychain lifecycle

| Event                                              | Action                                                                                       |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Password-protected operation succeeds (first time) | Store password in keychain for that wallet's fingerprint                                     |
| Password changed in Settings                       | Update keychain entry with new password                                                      |
| Password removed in Settings                       | Delete keychain entry; biometric reverts to standalone gate mode                             |
| Biometric toggle disabled                          | Delete all `sage-password-*` keychain entries                                                |
| Wallet deleted                                     | Delete that wallet's keychain entry                                                          |
| OS biometric enrollment changes                    | OS invalidates keychain items; next retrieval fails → dialog fallback → re-stored on success |

#### Settings UI changes

The biometric toggle remains in the **Preferences** section of Global Settings (not per-wallet Security) because it is a global setting that applies to all wallets. It is only visible on mobile when biometric hardware is available and enrolled.

When the toggle is disabled, all `sage-password-*` keychain entries are cleared.

#### Design decisions

- **No password at import time** — users set a password later via Settings. Simpler UX, same security outcome.
- **No session caching** — every protected operation prompts independently. The OS handles its own biometric caching at the system level. Sage does not cache passwords in memory after biometric retrieval.
- **Single dialog instance** — `PasswordProvider` renders one `PasswordDialog` at the provider level, avoiding duplicate dialog instances across components.
- **Unified auth entry point** — `requestPassword` subsumes the standalone `promptIfEnabled()` biometric check. Call sites make one auth call instead of two. The `BiometricContext` continues to exist for state management (`enabled`, `available`) but `promptIfEnabled()` is no longer called directly at operation sites.
- **Global biometric setting** — one toggle applies to all wallets. Per-wallet passwords are stored in the OS keychain on first successful use. No per-wallet biometric configuration needed.
- **Graceful degradation** — keychain retrieval failure (enrollment change, lockout, corruption, device restore) always falls back to the password dialog. The biometric path is an optimistic fast path, never a hard requirement.

## Error Handling

- **Wrong password**: AES-GCM authentication fails → `KeychainError::Decrypt` → frontend shows "Incorrect password" toast.
- **Public-key-only wallets**: `extract_secrets` returns `(None, None)` — no prompt needed. Frontend checks `has_secret_key` and `has_password` to decide.
- **Lost password**: No recovery. AES-256-GCM + Argon2 is irreversible without the password. UI warns at password-set time. Matches industry standard (Chia reference wallet, MetaMask).
- **Biometric enrollment changes**: OS invalidates keychain items when biometric enrollment changes. Next retrieval fails → falls back to password dialog → password re-stored in keychain on successful operation.
- **Biometric lockout**: After too many failed OS-level biometric attempts, the OS locks biometric temporarily. `requestPassword` detects retrieval failure and goes straight to the password dialog.
- **Stale keychain entry**: If the user changed their password but the keychain has the old one, the backend rejects with "Could not decrypt." The "Incorrect password" toast fires, the stale entry is cleared, and next attempt falls back to the dialog.
- **Device restore**: Keychain items may not survive a device restore depending on OS backup settings and access control flags. Same handling: retrieval fails → dialog fallback → re-stored on success.
- **App backgrounded during biometric**: OS may cancel the biometric prompt. Treated as cancellation → `requestPassword` returns `undefined`.
- **WalletConnect while backgrounded**: Biometric prompts cannot appear while backgrounded on iOS. `requestPassword` skips biometric and falls back to dialog (which will appear when the app returns to foreground).

## Migration

Existing keys encrypted with `b""` continue to work — the user simply never gets prompted. To add protection, the user triggers "Set Password" which calls `change_password(fingerprint, b"", new_password)`.

The `keys.bin` format change (adding `password_protected` to `KeyData::Secret`) requires a deserialization fallback: try new format first, fall back to old format with `password_protected: false`. On next save, the file is written in the new format.

### New dependency (biometric bridge)

- `tauri-plugin-keychain` — Rust crate (`tauri-plugin-keychain = "2"`) + JS bindings (`tauri-plugin-keychain`) for iOS Keychain / Android AccountManager access. Mobile-only. Added to `src-tauri/Cargo.toml` (mobile target) and `package.json`.
- Mobile capabilities (`src-tauri/capabilities/mobile.json`) — add `keychain:default` permission.

## What's NOT Changing

- `encrypt.rs` — AES-256-GCM + Argon2 implementation is already correct
- `keys.bin` encryption scheme — same Argon2 + AES-256-GCM, just with real passwords instead of `b""`
- Any sync, peer, or database logic
- `SendTransactionImmediately`, `SubmitTransaction`, `ViewCoinSpends` — these operate on pre-signed spend bundles or read-only data and do not call `extract_secrets()` or `sign()`
- Backend — no backend changes for the biometric bridge. It's entirely frontend.
