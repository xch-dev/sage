# Sage Wallet - Project Reference

## Overview
Sage is a high-performance Chia blockchain light wallet (beta). It connects directly to peers or a trusted full node. Built with **Tauri v2** (Rust backend + React/TypeScript frontend). Supports desktop (Win/Mac/Linux), iOS (TestFlight), and Android.

**Repo:** xch-dev/sage | **License:** Apache-2.0 | **Version:** 0.12.8 | **Stars:** 52 | **Commits:** 2,839
**Primary maintainers:** Rigidity (~1,629 commits), dkackman (~1,162 commits)

## Tech Stack
- **Frontend:** React 18.3, TypeScript 5.9, Vite 7.3, Tailwind CSS 3.4, shadcn/ui (Radix), Zustand, React Router v6 (hash), Lingui (i18n), Framer Motion
- **Backend:** Rust 2024 edition, Tauri 2.10, Tokio, SQLite/SQLx, Axum (RPC), chia-wallet-sdk 0.33
- **Crypto:** AES-256-GCM + Argon2 (keychain), BLS signatures (chia_rs), BIP39 mnemonics, Rustls (TLS)
- **Type bridge:** Specta (Rust→TS codegen) → `src/bindings.ts` (63KB auto-generated)

## Architecture

### Rust Crates (11)
| Crate | Purpose |
|-------|---------|
| `sage` | Core business logic, endpoints, config orchestration |
| `sage-api` | API types, records, request/response definitions (Specta + OpenAPI) |
| `sage-wallet` | Wallet driver, sync manager, peer communication, **main test suite** |
| `sage-database` | SQLite abstraction, queries (SQLx with compile-time checked SQL) |
| `sage-config` | TOML config management (wallet, network, migration) |
| `sage-keychain` | Password-based key encryption (AES-GCM + Argon2), BIP39 |
| `sage-rpc` | Axum HTTP/TLS RPC server with OpenAPI docs |
| `sage-client` | RPC client library |
| `sage-cli` | CLI binary (`sage` command) |
| `sage-assets` | External data fetching (NFT metadata, prices, icons) |
| `tauri-plugin-sage` | Custom Tauri plugin (NFC, biometric, platform-specific) |

### Frontend Structure
```
src/
├── App.tsx          # Router + 9 nested context providers
├── bindings.ts      # Auto-generated Tauri IPC types (Specta)
├── state.ts         # Zustand stores (wallet, offer, navigation)
├── components/      # 113 files
│   ├── ui/          # 26 shadcn primitives
│   ├── confirmations/ # 11 tx confirmation components
│   ├── dialogs/     # 8 modal dialogs
│   └── selectors/   # 5 asset/token selectors
├── pages/           # 31 route pages
├── contexts/        # 8 React contexts
├── hooks/           # 26 custom hooks
├── lib/             # Utilities (utils, themes, forms, exports)
├── walletconnect/   # WC v2 integration (commands, handler)
├── themes/          # Built-in themes (circuit, glass, xch, win95...)
└── locales/         # en-US, de-DE, zh-CN, es-MX (.po files)
```

### Key Patterns
- **Command/Event IPC:** Frontend calls Rust via `commands.*()`, receives updates via `events.syncEvent.listen()`
- **Error flow:** Rust `Error` → `ErrorKind` enum → frontend `ErrorContext` → toast/modal
- **State:** Zustand for app state, React Context for services (wallet, peer, price, WC, biometric)
- **Forms:** react-hook-form + Zod validation with custom amount/address validators
- **Tables:** TanStack React Table + React Virtual for large lists

## Important File Paths
- **Workspace config:** `Cargo.toml` (60+ workspace deps)
- **Tauri config:** `src-tauri/tauri.conf.json` (CSP is null!)
- **Endpoints:** `crates/sage/src/endpoints/` (keys, actions, data, offers, transactions, wallet_connect, settings, themes)
- **Sync engine:** `crates/sage-wallet/src/sync_manager.rs` (22KB)
- **Database schema:** `migrations/0001_setup.sql` (15.5KB) + 4 migration files
- **Keychain:** `crates/sage-keychain/src/encrypt.rs` (AES-GCM + Argon2)
- **Main struct:** `crates/sage/src/sage.rs` (Sage struct, initialization, DB setup)

## Build & Dev Commands
```bash
pnpm install                    # Install frontend deps
pnpm tauri dev                  # Dev mode
pnpm tauri dev --release        # Optimized dev
pnpm tauri build                # Production build
pnpm tauri ios dev              # iOS simulator
pnpm tauri android dev          # Android simulator
pnpm extract                    # Extract i18n strings
pnpm prettier                   # Format code
cargo sqlx prepare --workspace  # Regenerate SQLx query cache
RUST_LOG=debug,sqlx=off cargo t -p sage-wallet  # Run tests
```

## Code Quality Config
- **Rust lints:** `unsafe_code = deny`, `unwrap_used = warn`, `clippy::all = deny`
- **TS:** strict mode, noUnusedParameters, noFallthroughCasesInSwitch
- **CI:** Prettier check, Clippy, Cargo tests, cargo-machete, rustfmt
- **Platforms built:** macOS universal, Linux x64/arm64, Windows x64/arm64, iOS, Android

## Database
SQLite with WAL mode, foreign keys enabled. Key tables: coins, assets, nfts, dids, cats, collections, derivations, offers, transactions, mempool_items, blocks, files.

## Security Model (Trusted Device)
Sage runs on the user's own device — security relies on OS-level protection (login, disk encryption, app sandboxing), consistent with Chia GUI, Electrum, MetaMask, etc.

- **Keychain encryption uses empty password `b""`** — by design, developer plans to add optional user password during wallet setup (infrastructure already supports it)
- **CSP is `null`** in tauri.conf.json — should be set for defense-in-depth against NFT metadata injection (MEDIUM)
- **Theme image URLs not sanitized** — could leak IP via remote image in NFT-sourced themes (MEDIUM)
- **No file permission controls** on keys.bin — low risk on single-user systems, optional 0o600 enhancement
- **Fingerprint-only login** — standard for desktop wallets on trusted devices; mobile adds biometric auth for WalletConnect
- **PR #720 (open):** secure-element integration for vault support

## Current State
- Beta status, actively developed (~10 releases in 6 months)
- 9 open issues, 3 open PRs (secure-element, scheme handler, webhooks)
- No frontend tests exist; backend tests only in sage-wallet crate
- 4 supported languages (en-US, de-DE, zh-CN, es-MX)
- Theme system via NFT metadata (theme-o-rama library)
