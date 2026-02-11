# Sage Wallet - Security Audit & Application Report

**Date:** February 10, 2026
**Version Audited:** 0.12.8 (commit 0b6a05fb)
**Repository:** https://github.com/xch-dev/sage
**Scope:** Full codebase review — Rust backend, TypeScript frontend, build configuration, dependencies

---

## Executive Summary

Sage is a well-structured Chia blockchain light wallet with strong architectural foundations: memory-safe Rust backend, type-safe IPC via Specta, and modern React frontend. However, the wallet has **critical security gaps** in its key protection model — private keys are encrypted with an empty password, there is no user authentication gate, and sensitive files lack permission controls. These issues are particularly concerning for a financial application that holds real value. The codebase shows no signs of malicious intent; these appear to be deferred features in a beta product.

---

## 1. Critical Findings

### 1.1 Keychain Encryption Uses Empty Password
**Severity:** CRITICAL
**Impact:** Private keys and seed phrases can be trivially decrypted from the keys.bin file

**Details:** The keychain module (`crates/sage-keychain/src/encrypt.rs`) implements proper AES-256-GCM encryption with Argon2 key derivation. However, every call site throughout the codebase passes `b""` (empty byte array) as the password:

| File | Line | Call |
|------|------|------|
| `crates/sage/src/endpoints/keys.rs` | 141 | `add_secret_key(&master_sk, b"")` |
| `crates/sage/src/endpoints/keys.rs` | 155 | `add_mnemonic(&mnemonic, b"")` |
| `crates/sage/src/endpoints/keys.rs` | 330 | `extract_secrets(req.fingerprint, b"")` |
| `crates/sage/src/endpoints/actions.rs` | 201 | `extract_secrets(wallet.fingerprint, b"")` |
| `crates/sage/src/endpoints/offers.rs` | 156, 194 | `extract_secrets(wallet.fingerprint, b"")` |
| `crates/sage/src/endpoints/wallet_connect.rs` | 181, 220 | `extract_secrets(wallet.fingerprint, b"")` |
| `crates/sage/src/utils/spends.rs` | 15 | `extract_secrets(wallet.fingerprint, b"")` |

**Risk:** If an attacker obtains the `keys.bin` file (via malware, backup theft, or device access), they can decrypt all private keys instantly. The Argon2 KDF provides zero protection with an empty password since the salt is stored alongside the ciphertext.

**Note:** PR #720 (open) introduces secure-element integration which would partially mitigate this on supported hardware.

---

### 1.2 No User Authentication Gate
**Severity:** CRITICAL
**Impact:** Anyone with device access can send funds without entering a password

**Details:** The login mechanism (`crates/sage-api/src/requests/keys.rs`) requires only a fingerprint (uint32):

```rust
pub struct Login {
    pub fingerprint: u32,
}
```

Once logged in, all sensitive operations are available without further authentication:
- `get_secret_key()` — exports the mnemonic/private key
- `sign_coin_spends()` — signs arbitrary transactions
- `send_xch()`, `send_cat()` — sends funds
- `create_offer()`, `accept_offer()` — creates/accepts offers

**Mitigating factor:** On mobile, biometric authentication is available via `tauri-plugin-biometric` for WalletConnect signing operations. However, this does not protect against direct command invocation on desktop.

---

### 1.3 Content Security Policy Disabled
**Severity:** CRITICAL
**Impact:** No protection against XSS, script injection, or content injection attacks

**Details:** In `src-tauri/tauri.conf.json:23`:
```json
"security": {
    "csp": null
}
```

Setting CSP to `null` disables all content security policies. This means:
- Inline scripts can execute without restriction
- External resources can be loaded from any origin
- No protection against XSS via injected content (e.g., malicious NFT metadata)

**Risk scenario:** A maliciously crafted NFT with XSS payload in its metadata could potentially execute arbitrary JavaScript in the wallet context, gaining access to Tauri IPC commands.

---

## 2. High Severity Findings

### 2.1 No File Permission Controls on Sensitive Files
**Severity:** HIGH

**Details:** Sensitive files are created using default OS permissions (`crates/sage/src/sage.rs`):

```rust
fs::create_dir_all(&self.path)?;           // line 65 - default umask
fs::write(self.path.join("keys.bin"), ...)?; // line 507 - default perms
fs::write(self.path.join("config.toml"), ...)?; // line 498
```

No `chmod`, `set_permissions`, or `PermissionsExt` calls found in the entire codebase. On multi-user systems or with permissive umask settings, `keys.bin` could be world-readable.

**Affected files:**
- `keys.bin` — Encrypted (but with empty password) private keys
- `config.toml` — Wallet configuration
- `wallets.toml` — Wallet metadata
- `networks.toml` — Network configuration
- `wallets/{fingerprint}/{network}.sqlite` — Transaction/coin databases

---

### 2.2 SQLite Database Not Encrypted
**Severity:** HIGH

**Details:** The SQLite database stores coin data, transaction history, addresses, and derived keys (`crates/sage/src/sage.rs:414-422`). It uses WAL mode and normal synchronous setting, but no encryption:

```rust
SqliteConnectOptions::new()
    .filename(&db_path)
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal)
    .busy_timeout(Duration::from_secs(60))
```

**Risk:** The database contains derivation paths, transaction history, coin amounts, and addresses — sufficient to profile a user's wealth and activity even without private keys. SQLCipher or similar would provide encryption at rest.

---

### 2.3 No Rate Limiting on Sensitive Commands
**Severity:** HIGH

**Details:** All 60+ Tauri commands are registered without rate limiting, throttling, or attempt tracking (`src-tauri/src/lib.rs:25-144`). A compromised frontend or malicious extension could:
- Enumerate all wallets via `get_keys()`
- Export private keys via repeated `get_secret_key()` calls
- Submit transactions in rapid succession

---

## 3. Medium Severity Findings

### 3.1 WalletConnect Project ID Hardcoded
**Severity:** MEDIUM
**File:** `src/contexts/WalletConnectContext.tsx`

The WalletConnect project ID (`7a11dea2c7ab88dc4597d5d44eb79a18`) is hardcoded in the source. While not a direct vulnerability, this could be abused for quota exhaustion or impersonation if leaked.

---

### 3.2 Debug Logging May Leak Sensitive Data
**Severity:** MEDIUM
**File:** `crates/sage/src/endpoints/wallet_connect.rs:254`

```rust
debug!("{spend_bundle:?}");
```

Full spend bundle data (including amounts, addresses, and puzzle reveals) is logged at debug level. If debug logging is enabled in production builds, this exposes transaction details to log files.

---

### 3.3 No Memo Size Limits
**Severity:** MEDIUM
**File:** `crates/sage/src/utils/parse.rs`

The `parse_memos()` function accepts arbitrary-length hex strings without bounds checking:

```rust
pub fn parse_memos(input: Vec<String>) -> Result<Vec<Bytes>> {
    let mut memos = Vec::new();
    for memo in input {
        memos.push(Bytes::from(hex::decode(memo)?)); // No size limit
    }
    Ok(memos)
}
```

Extremely large memos could cause memory pressure or oversized transactions.

---

### 3.4 Self-Signed TLS Certificates for RPC
**Severity:** MEDIUM
**Files:** `crates/sage/src/sage.rs:192-210`, `crates/sage-rpc/src/lib.rs:92-107`

The RPC server uses self-signed certificates generated by `chia_wallet_sdk::load_ssl_cert()`. While acceptable for localhost operation (bound to `127.0.0.1`), this provides no authentication of the server identity if the RPC port is exposed.

---

### 3.5 Theme Background Image URL Not Sanitized
**Severity:** MEDIUM
**Files:** `src/components/Header.tsx`, `src/components/Layout.tsx`

Theme background images are applied directly as CSS:
```tsx
backgroundImage: `url(${currentTheme.backgroundImage})`
```

Since themes can be loaded from NFT metadata (external sources), a malicious theme could set `backgroundImage` to track users via a remote URL or exploit CSS injection.

---

## 4. Low Severity Findings

### 4.1 No Explicit Memory Clearing for Secrets
**Severity:** LOW
**File:** `crates/sage-keychain/src/key_data.rs`

```rust
pub struct SecretKeyData(pub Vec<u8>);
```

Private key material is stored in a plain `Vec<u8>` that is not zeroed on drop. While Rust's memory safety prevents use-after-free, the data may persist in memory, swap, or core dumps. The `zeroize` crate would provide secure clearing.

---

### 4.2 Keychain Serialized with Bincode (No Integrity Check)
**Severity:** LOW
**File:** `crates/sage-keychain/src/keychain.rs:30-39`

```rust
pub fn from_bytes(data: &[u8]) -> Result<Self, KeychainError> {
    let keys = bincode::deserialize(data)?;
    Ok(Self { rng: ChaCha20Rng::from_entropy(), keys })
}
pub fn to_bytes(&self) -> Result<Vec<u8>, KeychainError> {
    Ok(bincode::serialize(&self.keys)?)
}
```

The `keys.bin` file is a raw bincode serialization of the key HashMap. There is no HMAC or integrity check — a corrupted or tampered file could deserialize to unexpected state.

---

### 4.3 Incomplete TODO Notes in Security-Sensitive Code
**Severity:** LOW
**File:** `crates/sage/src/endpoints/wallet_connect.rs:248`

```rust
// TODO: Should this be the normal way of sending transactions?
```

Indicates unresolved security design decisions in the WalletConnect transaction signing path.

---

## 5. Positive Findings

### 5.1 Unsafe Code Denied
The workspace-level lint configuration denies all unsafe code:
```toml
unsafe_code = "deny"
```
No `unsafe` blocks were found in any crate. This is excellent practice for a financial application.

### 5.2 Strong Cryptographic Library Choices
- **AES-256-GCM** (aes-gcm 0.10.3) — authenticated encryption
- **Argon2** (argon2 0.5.3) — memory-hard KDF
- **Rustls with AWS LC RS** — FIPS-validated TLS
- **ChaCha20Rng** — cryptographically secure RNG for key operations
- **BIP39 2.0.0** — standard mnemonic implementation

### 5.3 RPC Server Bound to Localhost Only
```rust
let addr: SocketAddr = ([127, 0, 0, 1], app.config.rpc.port).into();
```

### 5.4 Type-Safe IPC via Specta
The Tauri IPC bridge uses Specta for compile-time TypeScript type generation, eliminating type mismatches between Rust and TypeScript. This prevents a class of serialization bugs.

### 5.5 SQL Injection Prevention
All database queries use SQLx's compile-time checked prepared statements. No string interpolation in SQL queries was found.

### 5.6 Comprehensive Error Type System
The `Error` enum in `crates/sage/src/error.rs` has 40+ specific variants, mapped to API-level `ErrorKind` categories. This prevents information leakage while providing useful error context.

---

## 6. Application Information

### Version & Release Cadence
| Version | Date | Key Changes |
|---------|------|-------------|
| 0.12.8 | Feb 8, 2026 | Tab order fix, RPC actions, QR sharing, wallet switching |
| 0.12.7 | Dec 3, 2025 | Clawback finalization, balance bug fixes |
| 0.12.6 | Nov 8, 2025 | Theme updates, offers page, Hong Kong DID |
| 0.12.5 | Oct 16, 2025 | Arbor compatibility, NFT minter hashes, dialog backgrounds |
| 0.12.4 | Sep 30, 2025 | Derivation improvements, revocable CATs, key propagation |
| 0.12.3 | Sep 22, 2025 | Edition options, tooltips, theme loading, coin totals |
| 0.12.2 | Sep 19, 2025 | Major theming overhaul, Dexie swap, glass themes |

### Open Issues (as of Feb 10, 2026)
- **#737** Show spendable balance
- **#735** App upgrade overwrites user theme selection
- **#729** Reflections about synchronization
- **#727** Deep links as alternative to WalletConnect
- **#726** Import error messages need more specificity
- **#723** Wallet error on single-sided request-only offer
- **#704** Android keyboard closes during token search
- **#642** Upload offer short codes / QR support (bug)
- **#575** Try from int error (bug)

### Open Pull Requests
- **#720** Secure-element integration for vault support (since Dec 15, 2025)
- **#728** Scheme handler (since Jan 16, 2026)
- **#694** Webhooks (since Oct 5, 2025)

### Known Bug History
34 total bug-labeled issues tracked. Notable patterns:
- UI state synchronization issues (checkboxes, balances, theme loading)
- Offer system edge cases (options in offers, single-sided offers)
- Syncing reliability (timeouts, data loss during batch sync, double-spend on retry)
- Mobile-specific issues (keyboard auto-close, safe area handling)

### Dependency Summary
- **Rust workspace dependencies:** 60+ crates
- **npm dependencies:** 110+ packages
- **Key blockchain deps:** chia-wallet-sdk 0.33.0, chia_rs, clvm_rs
- **No known CVEs** found in current dependency versions (as of audit date)

---

## 7. Recommendations Priority

| Priority | Finding | Effort |
|----------|---------|--------|
| P0 | Implement user password for keychain encryption | Medium |
| P0 | Add CSP to tauri.conf.json | Low |
| P0 | Add authentication gate for sensitive operations | Medium |
| P1 | Set restrictive file permissions (0o600) on keys.bin | Low |
| P1 | Add rate limiting to sensitive Tauri commands | Medium |
| P1 | Consider SQLCipher for database encryption | Medium |
| P2 | Sanitize theme URLs and metadata | Low |
| P2 | Add zeroize to SecretKeyData | Low |
| P2 | Add integrity checking to keys.bin | Low |
| P2 | Remove or guard debug logging of spend bundles | Low |
| P3 | Add memo size limits | Low |
| P3 | Move WalletConnect project ID to config | Low |

---

*This audit was performed through static code analysis only. Dynamic testing, fuzzing, and penetration testing were not in scope.*
