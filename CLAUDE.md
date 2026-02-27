# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sage is a light wallet for the Chia blockchain built with Tauri v2 (Rust backend + React/TypeScript frontend). It supports desktop (macOS, Linux, Windows) and mobile (iOS, Android) platforms.

## Common Commands

### Frontend
```bash
pnpm dev                    # Vite dev server (port 1420)
pnpm tauri dev              # Full app in dev mode
pnpm tauri dev --release    # Dev mode with optimizations
pnpm build                  # Build frontend only
pnpm tauri build            # Build complete application
pnpm lint                   # ESLint
pnpm prettier               # Format code
pnpm prettier:check         # Check formatting
pnpm extract                # Extract i18n translations
pnpm compile                # Compile i18n translations
```

### Backend (Rust)
```bash
cargo clippy --workspace --all-features --all-targets   # Lint
cargo fmt --all -- --files-with-diff --check             # Check formatting
cargo test -p sage-wallet                                # Run wallet tests (main test suite)
cargo test --workspace --all-features                    # Run all tests
```

### Database (requires sqlx-cli)
```bash
# Needs .env with DATABASE_URL=sqlite://./test.sqlite
sqlx db reset -y
cargo sqlx prepare --workspace
```

## Architecture

### Data Flow
```
React Frontend (src/) → IPC (tauri-specta) → Tauri Commands (src-tauri/) → sage crate → sage-wallet → sage-database (SQLite)
```

TypeScript bindings are auto-generated from Rust types via `specta`/`tauri-specta` into `src/bindings.ts`.

### Rust Workspace Crates (`crates/`)
- **sage** — Top-level orchestration, sync management
- **sage-wallet** — Core wallet logic, blockchain sync, coin drivers
- **sage-database** — SQLite via sqlx, compile-time checked queries
- **sage-api** — API definitions shared between Tauri and OpenAPI/RPC
- **sage-api/macro** — Proc macros for API generation
- **sage-keychain** — BIP39 mnemonics, AES-GCM encryption, Argon2 key derivation
- **sage-config** — TOML configuration management
- **sage-client** — RPC client
- **sage-rpc** — Axum-based RPC server
- **sage-cli** — CLI binary
- **sage-assets** — External asset fetching

### Frontend (`src/`)
- **components/ui/** — Shadcn UI components (New York style, Radix primitives)
- **pages/** — Route pages (hash router for desktop compatibility)
- **hooks/** — Custom React hooks
- **contexts/** — React contexts (Wallet, Peer, Price, Error, etc.)
- **state.ts** — Zustand global stores
- **locales/** — Lingui i18n (en-US, de-DE, zh-CN, es-MX)
- **themes/** — CSS variable-based theming with light/dark mode

### Key Patterns
- **State**: Zustand for global state, React Context for scoped state
- **Forms**: react-hook-form + zod validation
- **Tables**: @tanstack/react-table
- **i18n**: Lingui with PO format (extract → compile workflow)
- **Rust errors**: `thiserror` custom error types
- **Async**: Tokio runtime, Arc+Mutex for shared state, MPSC channels for sync events
- **Chia SDK**: `chia-wallet-sdk` v0.33.0 is the primary blockchain dependency

## Code Style

### Rust
- Edition 2024, toolchain 1.89.0
- `unsafe_code = "deny"` workspace-wide
- Strict clippy: `all = deny`, `pedantic = warn`
- Rustfmt with edition 2024 settings

### Frontend
- TypeScript strict mode, ESM modules
- pnpm (v10.13.1) as package manager
- Tailwind CSS for styling
- Path alias: `@/*` → `./src/*`

## Platform-Specific
- **src-tauri/** — Tauri wrapper and entry point
- **tauri-plugin-sage/** — Custom native plugin (iOS/Android platform code)
- Mobile uses conditional compilation (`cfg(mobile)`)
- Windows build requires CMake, Clang, NASM
