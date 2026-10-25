# Sage Wallet

**The sagest wallet for the Chia blockchain.**

Sage is a fast, open source light wallet for [Chia](https://www.chia.net). It syncs by connecting directly to peers on the network (or to a full node you trust), so you get a responsive wallet without downloading the entire blockchain. It runs on Windows, macOS, Linux, Android, and iOS.

## Features

- **XCH and CATs** - Send, receive, and manage XCH and Chia Asset Tokens, or issue your own tokens.
- **NFTs** - Mint, view, organize, and transfer NFTs and collections.
- **DIDs** - Create and manage Decentralized Identifiers.
- **Offers** - Make, take, and manage Chia offer files for trustless peer-to-peer trading, plus token swaps.
- **Options** - Mint and exercise option contracts.
- **WalletConnect** - Connect to dApps and DeFi services.
- **Apps** - Run sandboxed apps inside the wallet, built with the Sage App SDK.
- **RPC and CLI** - Run Sage headless and automate it via its RPC server, with an OpenAPI spec.
- **Multiple wallets** - Manage several keys, including watch-only wallets, with custom themes and many languages.

Sage is built on the foundation of [chia_rs](https://github.com/Chia-Network/chia_rs), [clvm_rs](https://github.com/Chia-Network/clvm_rs), and the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk), and is designed with maintainability and extensibility in mind from the beginning.

> [!NOTE]
> Sage is under active development. Please use it with care, and keep a secure backup of your mnemonic.

## Installation

Download the latest build for your platform from the [releases page](https://github.com/xch-dev/sage/releases). For iOS, join the [public TestFlight](https://testflight.apple.com/join/BmUdFXpP).

If you want to build from source, see the [Development](#development) section for instructions.

## Contributing

This is an open source project, and we welcome pull requests to improve any part of the wallet.

The frontend is currently written in TypeScript with [React](https://react.dev/) and [Shadcn UI](https://ui.shadcn.com/), and can be found in the `src` directory.

The backend is written in Rust, powered by [Tauri v2](https://v2.tauri.app/). The frontend and backend communicate via serialized commands and events over IPC. The `src-tauri` directory contains the backend wrapper code, and `crates` is the individual libraries that make up the wallet backend.

Finally, the wallet driver code is written on the backend using the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk).

## Development

### Setting up the environment

These instructions should get you up and running with a source installation.

First, there are some prerequisites:

1. Clone the repo
2. Install Rust via [Rustup](https://rustup.rs)
3. Install [PNPM](https://pnpm.io/installation)
4. Setup system dependencies for [Tauri](https://v2.tauri.app/start/prerequisites/)

Install the frontend dependencies:

```bash
pnpm install
```

#### Windows

The build process on Windows requires CMake, Clang and NASM installed.

```powershell
choco install cmake llvm nasm
```

### Starting the app

You can run the app with:

```bash
# For development purposes:
pnpm tauri:dev

# If you need optimizations:
pnpm tauri:dev --release
```

And build the application with:

```bash
pnpm tauri:build
```

You can also run the app in the iOS or Android simulator, though it may take some prior setup:

```bash
pnpm tauri ios dev
pnpm tauri android dev
```

### Working in the database

If you are going to be working on the wallet database follow the steps below.

Sage uses [sqlx](https://github.com/launchbadge/sqlx) for database integration, so first install the [sqlx-cli](https://lib.rs/crates/sqlx-cli)

```bash
cargo install sqlx-cli
```

Next create a `.env` file in the project root with these contents:

```bash
DATABASE_URL=sqlite://./test.sqlite
```

In order to sync your local environment to incoming query or schema changes or if you see rust compile errors like `error: error returned from database` you probably need to run this command:

```bash
sqlx db reset -y
```

If you add or change queries run this command to generate new sqlx SQL files:

```bash
cargo sqlx prepare --workspace
```

Schema changes, including new indices, go into the `migrations` folder as SQL DDL scripts.

### Testing

Currently, only the wallet driver code has tests. These can be run with:

```bash
RUST_LOG=debug,sqlx=off cargo t -p sage-wallet
```

The `sqlx=off` portion gets rid of noisy log spam from SQLx.

Most of the tests for the underlying coin spend implementations live in the Wallet SDK repo.

### Before Release

The following things have to be done before a new release is published:

1. Bump the version of all crates
2. Bump the versions in `src-tauri/gen/apple/sage-tauri_iOS/Info.plist`
3. Run `pnpm prettier` to format the code
4. Run `pnpm extract` to extract new translations
5. Create a new tagged release
6. Upload to TestFlight and the Google Play Store
7. Generate the OpenAPI specification `cargo run --bin sage rpc generate_openapi -o dist/openapi.json`
8. Upload to docusaurus
