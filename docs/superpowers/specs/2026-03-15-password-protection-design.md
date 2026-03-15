# Password Protection for Sage Wallet

**Issue:** [xch-dev/sage#206](https://github.com/xch-dev/sage/issues/206)
**Date:** 2026-03-15
**Status:** Design

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

### Biometric Layer

Biometric is a frontend-only concern. The backend never knows whether a password came from typing or biometric retrieval.

1. User enables biometric after setting a password
2. Frontend stores the password in the OS keychain with biometric access control (via a Tauri keyring/biometric plugin), keyed by wallet fingerprint
3. On protected operations: frontend tries biometric retrieval first, falls back to manual entry
4. Backend receives the password in the request struct either way

The OS keychain uses the platform's secure element under the hood (Secure Enclave on macOS/iOS, TEE on Android, TPM on Windows) without Sage needing to manage SE keys directly.

## Protected Operations

There are 7 code points where `b""` is passed to `extract_secrets` or `add_mnemonic`/`add_secret_key`, plus 2 encrypt-at-creation sites. However, because `sign()` is called through `transact()` and `transact_with()`, the password must flow through a much larger API surface.

### 1. Display mnemonic/secret key (1 site)

| Call site | Function |
|-----------|----------|
| `crates/sage/src/endpoints/keys.rs:353` | `get_secret_key()` |

### 2. Sign transactions and offers

The central signing path is:

```
endpoint method → transact() / transact_with() → sign() → extract_secrets()
```

**Direct `extract_secrets` call sites** (5 sites):

| Call site | Function |
|-----------|----------|
| `crates/sage/src/utils/spends.rs:15` | `sign()` — called by `transact_with()` and `sign_coin_spends()` |
| `crates/sage/src/endpoints/offers.rs:170` | `make_offer()` — calls `extract_secrets` directly |
| `crates/sage/src/endpoints/offers.rs:208` | `take_offer()` — calls `extract_secrets` directly |
| `crates/sage/src/endpoints/wallet_connect.rs:181` | `sign_message_with_public_key()` |
| `crates/sage/src/endpoints/wallet_connect.rs:220` | `sign_message_by_address()` |

**Transaction endpoints that flow through `transact()` → `sign()`** (21 endpoints):

`send_xch`, `bulk_send_xch`, `combine`, `auto_combine_xch`, `split`, `auto_combine_cat`, `issue_cat`, `send_cat`, `bulk_send_cat`, `multi_send`, `create_did`, `bulk_mint_nfts`, `transfer_nfts`, `add_nft_uri`, `assign_nfts_to_did`, `transfer_dids`, `normalize_dids`, `mint_option`, `transfer_options`, `exercise_options`, `finalize_clawback`

Plus `cancel_offer`, `cancel_offers`, and `create_transaction` (action system) which also flow through `transact()` / `transact_with()`.

### 3. Generate hardened keys (1 site)

| Call site | Function |
|-----------|----------|
| `crates/sage/src/endpoints/actions.rs:201` | `increase_derivation_index()` |

### 4. Key import — encrypt at creation (2 sites)

| Call site | Function |
|-----------|----------|
| `crates/sage/src/endpoints/keys.rs:141` | `import_key()` — secret key path |
| `crates/sage/src/endpoints/keys.rs:178` | `import_key()` — mnemonic path |

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

- Reusable password prompt dialog component
- Prompt shown before each protected operation (only when `has_password` is true for the active wallet)
- "Set Password" in wallet settings (calls `change_password` with empty old password)
- "Change Password" in wallet settings
- "Remove Password" in wallet settings (calls `change_password` with empty new password, with confirmation dialog warning about reduced security)
- Biometric opt-in toggle (calls keyring/biometric plugin)
- Biometric retrieval logic with fallback to manual entry
- Password change also updates the biometric keychain entry

### New dependency

- A Tauri keyring or biometric plugin (e.g., `tauri-plugin-biometry`) for the optional biometric layer

## Error Handling

- **Wrong password**: AES-GCM authentication fails → `KeychainError::Decrypt` → frontend shows "Incorrect password"
- **Public-key-only wallets**: `extract_secrets` returns `(None, None)` — no prompt needed. Frontend checks `has_secret_key` and `has_password` to decide.
- **Lost password**: No recovery. AES-256-GCM + Argon2 is irreversible without the password. UI warns at password-set time. Matches industry standard (Chia reference wallet, MetaMask).
- **Biometric invalidation**: OS invalidates keychain items when biometric enrollment changes. Falls back to manual password entry. User re-enables biometric by entering password again.

## Migration

Existing keys encrypted with `b""` continue to work — the user simply never gets prompted. To add protection, the user triggers "Set Password" which calls `change_password(fingerprint, b"", new_password)`.

The `keys.bin` format change (adding `password_protected` to `KeyData::Secret`) requires a deserialization fallback: try new format first, fall back to old format with `password_protected: false`. On next save, the file is written in the new format.

## What's NOT Changing

- `encrypt.rs` — AES-256-GCM + Argon2 implementation is already correct
- `keys.bin` encryption scheme — same Argon2 + AES-256-GCM, just with real passwords instead of `b""`
- Any sync, peer, or database logic
- `SendTransactionImmediately`, `SubmitTransaction`, `ViewCoinSpends` — these operate on pre-signed spend bundles or read-only data and do not call `extract_secrets()` or `sign()`
