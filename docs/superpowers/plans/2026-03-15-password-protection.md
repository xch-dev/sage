# Password Protection Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add opt-in per-operation password protection to Sage wallet, gating secret key access, transaction signing, and hardened key generation behind user-provided passwords.

**Architecture:** Layer real passwords onto the existing Argon2 + AES-256-GCM encryption in `sage-keychain`. Add `password: Option<String>` to all API request structs that trigger signing or secret access. Thread the password through `transact()` → `sign()` → `extract_secrets()`. Add `password_protected: bool` to `KeyData::Secret` and expose via `KeyInfo`. Frontend prompts per-operation when the active wallet has a password set.

**Tech Stack:** Rust (sage-keychain, sage-api, sage crates), TypeScript/React (frontend), Tauri IPC

**Spec:** `docs/superpowers/specs/2026-03-15-password-protection-design.md`

---

## Chunk 1: Keychain Core — `password_protected` flag and `change_password`

### Task 1: Add `password_protected` to `KeyData::Secret`

**Files:**

- Modify: `crates/sage-keychain/src/key_data.rs:9-20`

- [ ] **Step 1: Add `password_protected` field to `KeyData::Secret`**

In `crates/sage-keychain/src/key_data.rs`, add the field to the `Secret` variant:

```rust
Secret {
    #[serde_as(as = "Bytes")]
    master_pk: [u8; 48],
    entropy: bool,
    encrypted: Encrypted,
    password_protected: bool,
}
```

**IMPORTANT:** `keys.bin` uses `bincode` serialization, NOT JSON. `#[serde(default)]` does NOT work with bincode — adding a field changes the binary layout and breaks deserialization of existing files. We must handle backward compatibility in `from_bytes()`.

- [ ] **Step 1b: Add backward-compatible deserialization to `from_bytes()`**

In `crates/sage-keychain/src/keychain.rs`, update `from_bytes()` to try the new format first, then fall back to the old format:

```rust
/// Legacy KeyData without password_protected field, for backward compat
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
enum LegacyKeyData {
    Public {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
    },
    Secret {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
        entropy: bool,
        encrypted: Encrypted,
    },
}

impl From<LegacyKeyData> for KeyData {
    fn from(legacy: LegacyKeyData) -> Self {
        match legacy {
            LegacyKeyData::Public { master_pk } => KeyData::Public { master_pk },
            LegacyKeyData::Secret { master_pk, entropy, encrypted } => KeyData::Secret {
                master_pk,
                entropy,
                encrypted,
                password_protected: false,
            },
        }
    }
}
```

Then update `from_bytes()`:

```rust
pub fn from_bytes(data: &[u8]) -> Result<Self, KeychainError> {
    let keys: HashMap<u32, KeyData> = bincode::deserialize(data)
        .or_else(|_| {
            // Fall back to legacy format without password_protected
            let legacy: HashMap<u32, LegacyKeyData> = bincode::deserialize(data)?;
            Ok(legacy.into_iter().map(|(k, v)| (k, v.into())).collect())
        })?;
    Ok(Self {
        rng: ChaCha20Rng::from_entropy(),
        keys,
    })
}
```

Add the necessary imports for `LegacyKeyData` (`serde_with::serde_as`, `Encrypted` etc.).

- [ ] **Step 2: Verify the project compiles**

Run: `cargo check -p sage-keychain`
Expected: compilation errors in `keychain.rs` where `KeyData::Secret` is constructed without the new field.

- [ ] **Step 3: Fix all `KeyData::Secret` construction sites in `keychain.rs`**

In `crates/sage-keychain/src/keychain.rs`, update `add_secret_key()` (line ~138) and `add_mnemonic()` (line ~166) to include `password_protected`:

For `add_secret_key()`:

```rust
self.keys.insert(
    fingerprint,
    KeyData::Secret {
        master_pk: master_pk.to_bytes(),
        entropy: false,
        encrypted,
        password_protected: !password.is_empty(),
    },
);
```

For `add_mnemonic()`:

```rust
self.keys.insert(
    fingerprint,
    KeyData::Secret {
        master_pk: master_pk.to_bytes(),
        entropy: true,
        encrypted,
        password_protected: !password.is_empty(),
    },
);
```

- [ ] **Step 4: Add `is_password_protected()` accessor to `Keychain`**

In `crates/sage-keychain/src/keychain.rs`, add:

```rust
pub fn is_password_protected(&self, fingerprint: u32) -> bool {
    matches!(
        self.keys.get(&fingerprint),
        Some(KeyData::Secret { password_protected: true, .. })
    )
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p sage-keychain`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/sage-keychain/src/key_data.rs crates/sage-keychain/src/keychain.rs
git commit -m "feat: add password_protected flag to KeyData::Secret"
```

### Task 2: Add `change_password` to `Keychain`

**Files:**

- Modify: `crates/sage-keychain/src/keychain.rs`

- [ ] **Step 1: Write a test for `change_password`**

Add at the bottom of `crates/sage-keychain/src/keychain.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Mnemonic;

    #[test]
    fn test_change_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();

        let fingerprint = keychain.add_mnemonic(&mnemonic, b"").unwrap();
        assert!(!keychain.is_password_protected(fingerprint));

        // Set a password
        keychain.change_password(fingerprint, b"", b"secret123").unwrap();
        assert!(keychain.is_password_protected(fingerprint));

        // Old password should fail
        assert!(keychain.extract_secrets(fingerprint, b"").is_err());

        // New password should work
        let (mnemonic_out, Some(_sk)) = keychain.extract_secrets(fingerprint, b"secret123").unwrap() else {
            panic!("expected secret key");
        };
        assert!(mnemonic_out.is_some());

        // Change to another password
        keychain.change_password(fingerprint, b"secret123", b"newpass").unwrap();
        assert!(keychain.extract_secrets(fingerprint, b"secret123").is_err());
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"newpass").unwrap() else {
            panic!("expected secret key");
        };

        // Remove password
        keychain.change_password(fingerprint, b"newpass", b"").unwrap();
        assert!(!keychain.is_password_protected(fingerprint));
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"").unwrap() else {
            panic!("expected secret key");
        };
    }

    #[test]
    fn test_change_password_wrong_old_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();

        let fingerprint = keychain.add_mnemonic(&mnemonic, b"correct").unwrap();

        assert!(keychain.change_password(fingerprint, b"wrong", b"newpass").is_err());
        // Original password still works
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"correct").unwrap() else {
            panic!("expected secret key");
        };
    }

    #[test]
    fn test_change_password_public_key_fails() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        let master_pk = master_sk.public_key();

        let fingerprint = keychain.add_public_key(&master_pk).unwrap();

        assert!(keychain.change_password(fingerprint, b"", b"pass").is_err());
    }

    #[test]
    fn test_password_protected_flag_on_import() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();

        let fp_no_pass = keychain.add_mnemonic(&mnemonic, b"").unwrap();
        assert!(!keychain.is_password_protected(fp_no_pass));

        // Need a different mnemonic to avoid KeyExists
        let mut keychain2 = Keychain::default();
        let fp_with_pass = keychain2.add_mnemonic(&mnemonic, b"secret").unwrap();
        assert!(keychain2.is_password_protected(fp_with_pass));
    }

    #[test]
    fn test_serialization_roundtrip_with_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let fingerprint = keychain.add_mnemonic(&mnemonic, b"pass123").unwrap();

        let bytes = keychain.to_bytes().unwrap();
        let keychain2 = Keychain::from_bytes(&bytes).unwrap();

        assert!(keychain2.is_password_protected(fingerprint));
        let (_m, Some(_sk)) = keychain2.extract_secrets(fingerprint, b"pass123").unwrap() else {
            panic!("expected secret key");
        };
    }

    #[test]
    fn test_legacy_format_backward_compat() {
        // Simulate a legacy keys.bin by serializing with LegacyKeyData
        use std::collections::HashMap;

        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let fingerprint = keychain.add_mnemonic(&mnemonic, b"").unwrap();

        // Serialize, then deserialize into legacy format, re-serialize as legacy,
        // then verify new from_bytes can read it.
        // Alternatively: construct a HashMap<u32, LegacyKeyData> manually,
        // serialize with bincode, and verify Keychain::from_bytes reads it.
        let mut legacy_map: HashMap<u32, LegacyKeyData> = HashMap::new();
        if let Some(KeyData::Secret { master_pk, entropy, encrypted, .. }) = keychain.keys.get(&fingerprint) {
            legacy_map.insert(fingerprint, LegacyKeyData::Secret {
                master_pk: *master_pk,
                entropy: *entropy,
                encrypted: encrypted.clone(),
            });
        }
        let legacy_bytes = bincode::serialize(&legacy_map).unwrap();

        // This should succeed via the fallback path
        let restored = Keychain::from_bytes(&legacy_bytes).unwrap();
        assert!(!restored.is_password_protected(fingerprint));
        let (_m, Some(_sk)) = restored.extract_secrets(fingerprint, b"").unwrap() else {
            panic!("expected secret key");
        };
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p sage-keychain`
Expected: FAIL — `change_password` method does not exist.

- [ ] **Step 3: Implement `change_password`**

In `crates/sage-keychain/src/keychain.rs`, add the method to `impl Keychain`:

```rust
pub fn change_password(
    &mut self,
    fingerprint: u32,
    old_password: &[u8],
    new_password: &[u8],
) -> Result<(), KeychainError> {
    let key_data = self.keys.get(&fingerprint).ok_or(KeychainError::KeyNotFound)?;

    let (entropy, master_pk, secret_data) = match key_data {
        KeyData::Public { .. } => return Err(KeychainError::NoSecretKey),
        KeyData::Secret {
            entropy,
            master_pk,
            encrypted,
            ..
        } => {
            // Decrypt once with old password — this verifies the password
            // and gives us the raw secret data to re-encrypt
            let data = decrypt::<SecretKeyData>(encrypted, old_password)?;
            (*entropy, *master_pk, data)
        }
    };

    // Re-encrypt the same secret data with new password
    let encrypted = encrypt(new_password, &mut self.rng, &secret_data)?;

    self.keys.insert(
        fingerprint,
        KeyData::Secret {
            master_pk,
            entropy,
            encrypted,
            password_protected: !new_password.is_empty(),
        },
    );

    Ok(())
}
```

Also add these error variants to `crates/sage-keychain/src/error.rs` (they do not currently exist):

```rust
#[error("Key not found")]
KeyNotFound,

#[error("No secret key")]
NoSecretKey,
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sage-keychain`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sage-keychain/
git commit -m "feat: add change_password and keychain tests"
```

---

## Chunk 2: API Layer — Request struct changes and new endpoint types

### Task 3: Add `password` field to all protected request structs

**Files:**

- Modify: `crates/sage-api/src/requests/keys.rs`
- Modify: `crates/sage-api/src/requests/transactions.rs`
- Modify: `crates/sage-api/src/requests/offers.rs`
- Modify: `crates/sage-api/src/requests/actions.rs`
- Modify: `crates/sage-api/src/requests/wallet_connect.rs`
- Modify: `crates/sage-api/src/requests/action_system.rs`

- [ ] **Step 1: Add `password: Option<String>` to `GetSecretKey` and `ImportKey`**

In `crates/sage-api/src/requests/keys.rs`:

`GetSecretKey` — change from `Copy + Serialize, Deserialize` to just `Clone + Serialize, Deserialize` (since `Option<String>` is not `Copy`), add:

```rust
pub struct GetSecretKey {
    pub fingerprint: u32,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub password: Option<String>,
}
```

`ImportKey` — add:

```rust
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub password: Option<String>,
```

- [ ] **Step 2: Add `password` to all transaction request structs**

In `crates/sage-api/src/requests/transactions.rs`, add to every struct that has `auto_submit: bool` (these all flow through `transact()`):

```rust
    /// Password for signing (required if wallet is password-protected)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub password: Option<String>,
```

Structs to modify: `SendXch`, `BulkSendXch`, `Combine`, `AutoCombineXch`, `Split`, `AutoCombineCat`, `IssueCat`, `SendCat`, `BulkSendCat`, `MultiSend`, `CreateDid`, `BulkMintNfts`, `TransferNfts`, `AddNftUri`, `AssignNftsToDid`, `TransferDids`, `NormalizeDids`, `MintOption`, `TransferOptions`, `ExerciseOptions`, `FinalizeClawback`, `SignCoinSpends`.

- [ ] **Step 3: Add `password` to offer request structs**

In `crates/sage-api/src/requests/offers.rs`, add `password: Option<String>` (same pattern) to: `MakeOffer`, `TakeOffer`, `CancelOffer`, `CancelOffers`.

- [ ] **Step 4: Add `password` to action request structs**

In `crates/sage-api/src/requests/actions.rs`, add to `IncreaseDerivationIndex` (search for the struct — it contains `index: u32`).

- [ ] **Step 5: Add `password` to WalletConnect signing structs**

In `crates/sage-api/src/requests/wallet_connect.rs`, add `password: Option<String>` to `SignMessageWithPublicKey` and `SignMessageByAddress`. Note these use `#[serde(rename_all = "camelCase")]` so the field will serialize as `password` (single word, no rename needed).

- [ ] **Step 6: Add `password` to `CreateTransaction`**

In `crates/sage-api/src/requests/action_system.rs`, add to `CreateTransaction`:

```rust
    /// Password for signing (required if wallet is password-protected)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub password: Option<String>,
```

- [ ] **Step 7: Verify compilation**

Run: `cargo check -p sage-api`
Expected: PASS (these are just data structs, no logic changes).

- [ ] **Step 8: Commit**

```bash
git add crates/sage-api/src/requests/
git commit -m "feat: add password field to all protected request structs"
```

### Task 4: Add `ChangePassword` request/response and `has_password` to `KeyInfo`

**Files:**

- Modify: `crates/sage-api/src/requests/keys.rs`
- Modify: `crates/sage-api/src/types/key_info.rs`

- [ ] **Step 1: Add `ChangePassword` / `ChangePasswordResponse` to keys.rs**

In `crates/sage-api/src/requests/keys.rs`, add (following the existing struct pattern with openapi/tauri derives):

```rust
/// Change the password for a wallet's secret key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Change the password used to encrypt a wallet's secret key."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePassword {
    /// Wallet fingerprint
    pub fingerprint: u32,
    /// Current password (empty string if no password is set)
    pub old_password: String,
    /// New password (empty string to remove password protection)
    pub new_password: String,
}

/// Response after changing the password
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Authentication & Keys"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePasswordResponse {}
```

- [ ] **Step 2: Add `has_password` to `KeyInfo`**

In `crates/sage-api/src/types/key_info.rs`, add to `KeyInfo`:

```rust
pub struct KeyInfo {
    pub name: String,
    pub fingerprint: u32,
    pub public_key: String,
    pub kind: KeyKind,
    pub has_secrets: bool,
    pub has_password: bool,
    pub network_id: String,
    pub emoji: Option<String>,
}
```

- [ ] **Step 3: Register `ChangePassword` in the endpoint macro system**

The endpoint macro system reads from `crates/sage-api/endpoints.json`. Each entry maps an endpoint name to a boolean indicating whether it is async (`true`) or sync (`false`).

Add `change_password` to `crates/sage-api/endpoints.json`:

```json
  "is_asset_owned": true,
  "change_password": false
```

`change_password` is `false` (sync) because it only calls `keychain.change_password()` and `save_keychain()` — no async operations.

Also ensure `ChangePassword` and `ChangePasswordResponse` are re-exported through the `sage_api` module hierarchy. The types in `crates/sage-api/src/requests/keys.rs` should be auto-exported via `pub use requests::*` in `crates/sage-api/src/lib.rs`. Verify by checking how `GetSecretKey` and `GetSecretKeyResponse` are exported.

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p sage-api`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sage-api/
git commit -m "feat: add ChangePassword endpoint and has_password to KeyInfo"
```

---

## Chunk 3: Backend Endpoints — Thread passwords through the call chain

### Task 5: Update `sign()` and `transact()` to accept passwords

**Files:**

- Modify: `crates/sage/src/utils/spends.rs`
- Modify: `crates/sage/src/endpoints/transactions.rs`

- [ ] **Step 1: Add `password` parameter to `sign()`**

In `crates/sage/src/utils/spends.rs`, change the signature:

```rust
pub(crate) async fn sign(
    &self,
    coin_spends: Vec<CoinSpend>,
    partial: bool,
    password: &[u8],
) -> Result<SpendBundle> {
    let wallet = self.wallet()?;

    let (_mnemonic, Some(master_sk)) =
        self.keychain.extract_secrets(wallet.fingerprint, password)?
    else {
        return Err(Error::NoSigningKey);
    };
    // ... rest unchanged
```

- [ ] **Step 2: Add `password` parameter to `transact()` and `transact_with()`**

In `crates/sage/src/endpoints/transactions.rs`, update:

```rust
pub(crate) async fn transact(
    &self,
    coin_spends: Vec<CoinSpend>,
    auto_submit: bool,
    password: &[u8],
) -> Result<TransactionResponse> {
    self.transact_with(coin_spends, auto_submit, ConfirmationInfo::default(), password)
        .await
}

pub(crate) async fn transact_with(
    &self,
    coin_spends: Vec<CoinSpend>,
    auto_submit: bool,
    info: ConfirmationInfo,
    password: &[u8],
) -> Result<TransactionResponse> {
    if auto_submit {
        let spend_bundle = self.sign(coin_spends.clone(), false, password).await?;
        self.submit(spend_bundle).await?;
    }
    // ... rest unchanged
```

- [ ] **Step 3: Extract password and pass it in all transaction endpoints (same file)**

Do NOT commit yet — complete all call site fixes in one go to keep the branch compilable.

- [ ] **Step 4: Define the password extraction pattern**

At the top of `crates/sage/src/endpoints/transactions.rs` (or in a shared util), the pattern for every endpoint is the same:

```rust
let password = req.password.unwrap_or_default().into_bytes();
```

Use this inline in each endpoint. No helper function needed — it's one line.

- [ ] **Step 5: Update every transaction endpoint**

For each of the 21 transaction endpoint methods, add the password extraction and pass it to `transact()` or `transact_with()`. The pattern for every endpoint is the same. Example for `send_xch`:

```rust
pub async fn send_xch(&self, req: SendXch) -> Result<TransactionResponse> {
    let wallet = self.wallet()?;
    let password = req.password.unwrap_or_default().into_bytes();
    let puzzle_hash = self.parse_address(req.address)?;
    let amount = parse_amount(req.amount)?;
    let fee = parse_amount(req.fee)?;
    let memos = parse_memos(req.memos)?;

    let coin_spends = wallet
        .send_xch(vec![(puzzle_hash, amount)], fee, memos, req.clawback)
        .await?;
    self.transact(coin_spends, req.auto_submit, &password).await
}
```

Apply the same pattern to all 21 endpoints: extract `password` from `req`, pass `&password` as the last arg to `self.transact()` or `self.transact_with()`.

Also update `sign_coin_spends` which calls `self.sign()` directly:

```rust
let password = req.password.unwrap_or_default().into_bytes();
let spend_bundle = self.sign(coin_spends, req.partial, &password).await?;
```

- [ ] **Step 6: Continue to remaining endpoints (do NOT commit yet)**

### Task 6: Thread password through offers, actions, wallet_connect, and keys

**Files:**

- Modify: `crates/sage/src/endpoints/offers.rs`
- Modify: `crates/sage/src/endpoints/actions.rs`
- Modify: `crates/sage/src/endpoints/wallet_connect.rs`
- Modify: `crates/sage/src/endpoints/keys.rs`
- Modify: `crates/sage/src/endpoints/action_system.rs` (for `create_transaction`)

- [ ] **Step 1: Update `offers.rs`**

`make_offer()` and `take_offer()` call `extract_secrets` directly. `cancel_offer()` and `cancel_offers()` call `transact()`. Update all four:

For `make_offer` and `take_offer`:

```rust
let password = req.password.unwrap_or_default().into_bytes();
// ...
let (_mnemonic, Some(master_sk)) =
    self.keychain.extract_secrets(wallet.fingerprint, &password)?
```

For `cancel_offer` and `cancel_offers`:

```rust
let password = req.password.unwrap_or_default().into_bytes();
// ... pass &password to self.transact()
```

- [ ] **Step 2: Update `actions.rs`**

In `increase_derivation_index()`:

```rust
let password = req.password.unwrap_or_default().into_bytes();
// ...
let (_mnemonic, Some(master_sk)) =
    self.keychain.extract_secrets(wallet.fingerprint, &password)?
```

- [ ] **Step 3: Update `wallet_connect.rs`**

Both `sign_message_with_public_key` and `sign_message_by_address`:

```rust
let password = req.password.unwrap_or_default().into_bytes();
// ...
let (_mnemonic, Some(master_sk)) =
    self.keychain.extract_secrets(wallet.fingerprint, &password)?
```

- [ ] **Step 4: Update `keys.rs`**

`import_key()` — pass password to `add_secret_key` and `add_mnemonic`:

```rust
let password_bytes = req.password.unwrap_or_default().into_bytes();
// ...
self.keychain.add_secret_key(&master_sk, &password_bytes)?
// ...
self.keychain.add_mnemonic(&mnemonic, &password_bytes)?
```

`get_secret_key()` — pass password to `extract_secrets`:

```rust
let password = req.password.unwrap_or_default().into_bytes();
let (mnemonic, Some(secret_key)) = self.keychain.extract_secrets(req.fingerprint, &password)?
```

`get_key()` and `get_keys()` — populate `has_password` on `KeyInfo`:

```rust
has_password: self.keychain.is_password_protected(fingerprint),
```

- [ ] **Step 5: Update `action_system.rs`**

In the `create_transaction()` method:

```rust
let password = req.password.unwrap_or_default().into_bytes();
// ... pass &password to self.transact_with()
```

- [ ] **Step 6: Verify full compilation**

Run: `cargo check -p sage`
Expected: PASS — all `b""` usages replaced, all call sites updated.

- [ ] **Step 7: Run existing tests**

Run: `cargo test -p sage-rpc`
Expected: FAIL — existing tests (like `test_send_xch`) pass `SendXch { ... }` without a `password` field. Since `password` has `#[serde(default)]`, deserialization should still work. But if tests construct structs directly, they'll need `password: None` added. Check and fix.

- [ ] **Step 8: Fix any test compilation issues**

In `crates/sage-rpc/src/tests.rs`, add `password: None` to any struct literal that now requires it (e.g., `SendXch`, `ImportKey`).

- [ ] **Step 9: Run tests again**

Run: `cargo test -p sage-rpc`
Expected: PASS

- [ ] **Step 10: Commit (all call chain changes in one compilable commit)**

```bash
git add crates/sage/src/utils/spends.rs crates/sage/src/endpoints/ crates/sage-rpc/src/tests.rs
git commit -m "feat: thread password through all protected endpoints"
```

### Task 7: Add `change_password` endpoint

**Files:**

- Modify: `crates/sage/src/endpoints/keys.rs`

- [ ] **Step 1: Implement `change_password` endpoint**

In `crates/sage/src/endpoints/keys.rs`, add:

```rust
pub fn change_password(&mut self, req: ChangePassword) -> Result<ChangePasswordResponse> {
    let old_password = req.old_password.into_bytes();
    let new_password = req.new_password.into_bytes();
    self.keychain.change_password(req.fingerprint, &old_password, &new_password)?;
    self.save_keychain()?;
    Ok(ChangePasswordResponse {})
}
```

Make sure `ChangePassword` and `ChangePasswordResponse` are imported at the top.

- [ ] **Step 2: Verify endpoint is registered**

`change_password` should already be registered in `crates/sage-api/endpoints.json` from Task 4, Step 3. Verify it appears there as `"change_password": false`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p sage`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/sage/src/endpoints/keys.rs
git commit -m "feat: add change_password endpoint"
```

---

## Chunk 4: Integration Tests

### Task 8: Add integration tests for password protection

**Files:**

- Modify: `crates/sage-rpc/src/tests.rs`

- [ ] **Step 1: Add test for password-protected key import and secret retrieval**

```rust
#[tokio::test]
async fn test_password_protected_import() -> Result<()> {
    let mut app = TestApp::new().await?;

    // Import with password
    let response = app.import_key(ImportKey {
        key: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        name: "Test".to_string(),
        save_secrets: true,
        password: Some("mypassword".to_string()),
        // ... other required fields with defaults
    }).await?;

    let fingerprint = response.fingerprint;

    // Verify has_password is true
    let key = app.get_key(GetKey { fingerprint: Some(fingerprint) }).await?.key.unwrap();
    assert!(key.has_password);

    // Getting secret without password should fail
    let result = app.get_secret_key(GetSecretKey {
        fingerprint,
        password: None,
    }).await;
    assert!(result.is_err());

    // Getting secret with correct password should work
    let result = app.get_secret_key(GetSecretKey {
        fingerprint,
        password: Some("mypassword".to_string()),
    }).await?;
    assert!(result.secrets.is_some());

    Ok(())
}
```

- [ ] **Step 2: Add test for password-protected transaction signing**

```rust
#[tokio::test]
async fn test_password_protected_send() -> Result<()> {
    let mut app = TestApp::new().await?;

    // Setup with password - use the test helper but import with password
    // This may need adjusting based on the TestApp::setup_bls helper
    let fingerprint = app.setup_bls_with_password(1000, "secret").await?;

    app.wait_for_coins().await;

    // Send without password should fail
    let result = app.send_xch(SendXch {
        address: "txch1...".to_string(), // use a valid test address
        amount: Amount::u64(100),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        password: None,
    }).await;
    assert!(result.is_err());

    // Send with password should succeed
    let result = app.send_xch(SendXch {
        address: "txch1...".to_string(),
        amount: Amount::u64(100),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        password: Some("secret".to_string()),
    }).await;
    assert!(result.is_ok());

    Ok(())
}
```

Note: The exact test setup will depend on how `TestApp::setup_bls` works. You may need to add a `setup_bls_with_password` helper that imports a key with a password, or modify the import call in the test directly.

- [ ] **Step 3: Add test for change_password**

```rust
#[tokio::test]
async fn test_change_password() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(0).await?;

    // Initially no password
    let key = app.get_key(GetKey { fingerprint: Some(fingerprint) }).await?.key.unwrap();
    assert!(!key.has_password);

    // Set password
    app.change_password(ChangePassword {
        fingerprint,
        old_password: "".to_string(),
        new_password: "secret".to_string(),
    }).await?;

    // Now has_password should be true
    let key = app.get_key(GetKey { fingerprint: Some(fingerprint) }).await?.key.unwrap();
    assert!(key.has_password);

    // Old empty password should fail
    let result = app.get_secret_key(GetSecretKey {
        fingerprint,
        password: None,
    }).await;
    assert!(result.is_err());

    // New password should work
    let result = app.get_secret_key(GetSecretKey {
        fingerprint,
        password: Some("secret".to_string()),
    }).await?;
    assert!(result.secrets.is_some());

    Ok(())
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test -p sage-keychain && cargo test -p sage-rpc`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sage-rpc/src/tests.rs
git commit -m "test: add integration tests for password protection"
```

---

## Chunk 5: Tauri Command Layer and Frontend (Skeleton)

### Task 9: Ensure Tauri commands compile with new request structs

**Files:**

- Modify: `src-tauri/src/commands.rs` (if needed — the macro should auto-generate)

- [ ] **Step 1: Check if the Tauri command layer auto-generates from the endpoint macro**

The `impl_endpoints_tauri!` macro in `src-tauri/src/commands.rs` auto-generates Tauri commands from the endpoint definitions. If `change_password` was added to the endpoint list in the macro, it should auto-generate.

Run: `cargo check -p sage-desktop` (or whatever the Tauri package name is)
Expected: PASS if macro handles everything. If not, manually add the `change_password` command.

- [ ] **Step 2: Verify the full workspace compiles**

Run: `cargo check --workspace`
Expected: PASS

- [ ] **Step 3: Commit if any changes needed**

```bash
git add src-tauri/
git commit -m "feat: wire change_password through Tauri command layer"
```

### Task 10: Frontend — this task is a placeholder for frontend work

The frontend implementation is a significant body of work that depends on the Sage React app structure. The key pieces are:

1. **Password prompt dialog component** — a reusable modal that collects a password
2. **Wiring** — every protected Tauri `invoke()` call needs to collect the password first (if `has_password` is true for the active wallet) and include it in the request
3. **Settings UI** — "Set Password", "Change Password", "Remove Password" buttons
4. **Biometric toggle** — future work, depends on selecting a Tauri keyring/biometric plugin

This task should be planned separately once the backend is complete and tested, as it requires exploring the React app structure, identifying all `invoke()` call sites, and designing the prompt flow.

- [ ] **Step 1: Document the frontend contract**

Create a brief document listing:

- All Tauri command names that now accept `password`
- The `has_password` field on `KeyInfo` for conditional prompting
- The `change_password` command for settings UI

- [ ] **Step 2: Commit**

```bash
git commit --allow-empty -m "docs: frontend password protection contract ready"
```
