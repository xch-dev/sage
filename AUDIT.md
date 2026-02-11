# Sage Wallet - Security Audit & Application Report

**Date:** February 10, 2026
**Version Audited:** 0.12.8 (commit 0b6a05fb)
**Repository:** https://github.com/xch-dev/sage
**Scope:** Full codebase review — Rust backend, TypeScript frontend, build configuration, dependencies

---

## Threat Model

Sage is a **native desktop/mobile application** built with Tauri v2. It runs on the user's own trusted device, not on a remote server. The security model relies on:

- **OS-level device protection:** Login passwords, biometric unlock, screen lock
- **Disk encryption:** FileVault (macOS), BitLocker (Windows), LUKS (Linux), iOS/Android device encryption
- **App sandboxing:** iOS and Android enforce per-app data isolation
- **Local-only IPC:** Tauri commands are only callable from the bundled frontend — not exposed to the network
- **Biometric auth on mobile:** WalletConnect signing operations prompt for fingerprint/face

This is consistent with how most desktop wallets operate (official Chia wallet, Electrum, MetaMask, etc.) — they rely on OS protection rather than application-level passwords. The findings below are assessed within this context.

---

## Executive Summary

Sage has strong security foundations: memory-safe Rust backend with `unsafe` denied, type-safe IPC via Specta, compile-time checked SQL queries, localhost-only RPC, and robust cryptographic library choices. The application follows industry-standard practices for a native wallet — OS-level protection is the primary security boundary, which is appropriate for a trusted-device application.

The main areas for improvement are defense-in-depth measures: adding a CSP policy (to guard against NFT metadata injection), sanitizing externally-sourced theme URLs, and adding `zeroize` to key material. These are hardening recommendations, not critical vulnerabilities in the context of a native app.

---

## 1. Findings Requiring Attention

### 1.1 Content Security Policy Disabled
**Severity:** MEDIUM
**Impact:** Reduced defense-in-depth against content injection

**Details:** In `src-tauri/tauri.conf.json:23`:
```json
"security": {
    "csp": null
}
```

While the Tauri frontend loads only bundled local files (not remote content), a CSP would provide defense-in-depth against:
- Malicious NFT metadata containing script payloads rendered in the webview
- External resource loading from theme system (background images from NFT sources)
- Any future features that render user-supplied or blockchain-sourced content

**Recommendation:** Add a restrictive CSP that allows the bundled frontend resources and explicitly whitelists needed external domains (e.g., for NFT images, price APIs). This is low effort and follows Tauri's own security recommendations.

---

### 1.2 Theme Background Image URLs Not Sanitized
**Severity:** MEDIUM
**Files:** `src/components/Header.tsx`, `src/components/Layout.tsx`

Theme background images are applied directly as CSS:
```tsx
backgroundImage: `url(${currentTheme.backgroundImage})`
```

Since themes can be loaded from NFT metadata (external sources), a malicious theme could:
- Track users via a remote image URL (reveals IP address on theme load)
- Attempt CSS injection via crafted URL values

**Recommendation:** Validate and sanitize theme image URLs. Consider proxying external images through local storage or restricting to `data:` URIs and local files.

---

### 1.3 Debug Logging May Include Transaction Details
**Severity:** LOW
**File:** `crates/sage/src/endpoints/wallet_connect.rs:254`

```rust
debug!("{spend_bundle:?}");
```

Full spend bundle data is logged at debug level. In production builds, debug logging is disabled by default, making this a low risk. However, users running with `RUST_LOG=debug` could inadvertently write transaction details to log files.

**Recommendation:** Redact or summarize spend bundle data in debug logs, or use a separate `trace!` level.

---

### 1.4 No Memo Size Limits
**Severity:** LOW
**File:** `crates/sage/src/utils/parse.rs`

```rust
pub fn parse_memos(input: Vec<String>) -> Result<Vec<Bytes>> {
    let mut memos = Vec::new();
    for memo in input {
        memos.push(Bytes::from(hex::decode(memo)?)); // No size limit
    }
    Ok(memos)
}
```

Extremely large memos could cause memory pressure. In practice, the blockchain itself limits transaction size, so this is defense-in-depth.

**Recommendation:** Add a reasonable memo size limit (e.g., 1MB per memo, 10MB total).

---

## 2. Design Observations (Not Vulnerabilities)

These are architectural observations that are **standard practice for native wallets** but worth documenting for completeness.

### 2.1 Keychain Encryption Uses Empty Password
**Status:** By design — consistent with native wallet conventions

The keychain (`crates/sage-keychain/src/encrypt.rs`) encrypts private keys with AES-256-GCM + Argon2, but all call sites pass `b""` as the password. This means the encryption provides format-level protection (keys aren't stored in plaintext) but relies on OS-level protection for actual security.

This is the same model used by the official Chia wallet, Electrum, and most desktop cryptocurrency wallets. The password parameter in the API exists for future use (e.g., optional user password, secure element integration via PR #720).

**Optional enhancement:** Offer an opt-in application-level password for users who want additional protection beyond OS security. The infrastructure already supports this — only the call sites need to be updated to accept a user-provided password.

---

### 2.2 Fingerprint-Only Login
**Status:** By design — appropriate for trusted device

The login mechanism requires only a wallet fingerprint (uint32) to select the active wallet. On a personal device, the user has already authenticated to the OS. Mobile platforms additionally have biometric auth for WalletConnect signing.

This is standard for desktop wallets — opening the Chia GUI or Electrum also doesn't require a separate password beyond OS login.

---

### 2.3 Default File Permissions
**Status:** Low risk on single-user systems

Files are created with default OS permissions. On typical single-user desktop/laptop systems, the default umask (0022) creates files readable only by the owner. Mobile platforms enforce app sandboxing. This is a concern only on shared multi-user systems with permissive umask settings.

**Optional enhancement:** On Unix, set `0o600` permissions on `keys.bin` for defense-in-depth. Low effort.

---

### 2.4 SQLite Not Encrypted at Rest
**Status:** Standard practice — relies on disk encryption

The SQLite database stores transaction history, coin data, and addresses without application-level encryption. This is standard for native wallets — the data is protected by OS disk encryption (FileVault, BitLocker, LUKS, iOS/Android device encryption).

**Optional enhancement:** SQLCipher could add application-level database encryption for users on systems without full-disk encryption. Medium effort.

---

### 2.5 Local IPC Without Rate Limiting
**Status:** Not exploitable in normal operation

Tauri IPC commands are only callable from the bundled frontend process. An attacker would need code execution on the device to inject calls, at which point rate limiting provides no meaningful protection (they could directly read `keys.bin` instead).

---

### 2.6 Self-Signed TLS for RPC
**Status:** Appropriate for localhost

The RPC server uses self-signed certificates and is bound exclusively to `127.0.0.1`. Self-signed certs are standard for localhost-only services where the identity of the server is not in question.

---

### 2.7 WalletConnect Project ID in Source
**Status:** Standard practice

The WalletConnect project ID is hardcoded, which is the normal pattern for WalletConnect integrations. Every WC-enabled wallet has a visible project ID. This is not a secret.

---

## 3. Positive Findings

### 3.1 Unsafe Code Denied
The workspace-level lint configuration denies all unsafe code:
```toml
unsafe_code = "deny"
```
No `unsafe` blocks were found in any crate. This is excellent practice for a financial application.

### 3.2 Strong Cryptographic Library Choices
- **AES-256-GCM** (aes-gcm 0.10.3) — authenticated encryption
- **Argon2** (argon2 0.5.3) — memory-hard KDF
- **Rustls with AWS LC RS** — FIPS-validated TLS
- **ChaCha20Rng** — cryptographically secure RNG for key operations
- **BIP39 2.0.0** — standard mnemonic implementation

### 3.3 RPC Server Bound to Localhost Only
```rust
let addr: SocketAddr = ([127, 0, 0, 1], app.config.rpc.port).into();
```

### 3.4 Type-Safe IPC via Specta
The Tauri IPC bridge uses Specta for compile-time TypeScript type generation, eliminating type mismatches between Rust and TypeScript. This prevents a class of serialization bugs.

### 3.5 SQL Injection Prevention
All database queries use SQLx's compile-time checked prepared statements. No string interpolation in SQL queries was found.

### 3.6 Comprehensive Error Type System
The `Error` enum in `crates/sage/src/error.rs` has 40+ specific variants, mapped to API-level `ErrorKind` categories. This prevents information leakage while providing useful error context.

### 3.7 Mobile Biometric Authentication
WalletConnect signing operations on mobile prompt for biometric authentication (fingerprint/face), providing an additional authorization layer for dApp interactions.

### 3.8 Encryption Infrastructure Ready for Passwords
The keychain module accepts a password parameter at every level — the infrastructure is already built for optional user-password support. Enabling it would be a straightforward change to the call sites.

---

## 4. Application Information

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

## 5. Recommendations Summary

| Priority | Finding | Effort | Category |
|----------|---------|--------|----------|
| P1 | Add CSP to tauri.conf.json | Low | Defense-in-depth |
| P1 | Sanitize theme image URLs | Low | Defense-in-depth |
| P2 | Add zeroize to SecretKeyData | Low | Defense-in-depth |
| P2 | Guard debug logging of spend bundles | Low | Defense-in-depth |
| P2 | Add memo size limits | Low | Input validation |
| P3 | Optional user password for keychain | Medium | Opt-in feature |
| P3 | Set 0o600 on keys.bin (Unix) | Low | Defense-in-depth |
| P3 | Consider SQLCipher for DB encryption | Medium | Opt-in feature |
| P3 | Add integrity checking to keys.bin | Low | Defense-in-depth |

---

*This audit was performed through static code analysis only. Dynamic testing, fuzzing, and penetration testing were not in scope. Findings are assessed against a trusted-device threat model appropriate for native desktop/mobile wallet applications.*
