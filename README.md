# Sage Wallet

A high-performance light wallet that offers users the ability to connect directly to peers on the Chia blockchain or to a trusted full node. Key features include WalletConnect for integration with decentralized applications (dApps) and DeFi services, support for Chia offer files, and compatibility with Chia's standards for NFTs and Asset Tokens (CATs). This wallet also facilitates the creation, viewing, and management of NFTs, the minting of new tokens, and the management of Decentralized Identifiers (DIDs). Sage Wallet is designed for ease of use, security, and future extensibility. It is currently available in beta, so it should be used with caution.

Sage is built upon the reliable foundation of [chia_rs](https://github.com/Chia-Network/chia_rs), [clvm_rs](https://github.com/Chia-Network/clvm_rs), and the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk). It's designed with maintainability and future extensibility in mind from the beginning.

## Installation

You can download binaries for any platform (including desktop and Android) on the releases page. For iOS, you can participate in the [public TestFlight](https://testflight.apple.com/join/BmUdFXpP).

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

### Starting the app

You can run the app with:

```bash
# For development purposes:
pnpm tauri dev

# If you need optimizations:
pnpm tauri dev --release
```

And build the application with:

```bash
pnpm tauri build
```

You can also run the app in the iOS or Android simulator, though it may take some prior setup:

```bash
pnpm tauri ios dev
pnpm tauri android dev
```

### Working in the database

If you are going to be working on the wallet database follow the steps below.

Sage uses [sqlx](https://github.com/launchbadge/sqlx) for database integration, so first sintall the [sqlx-cli](https://lib.rs/crates/sqlx-cli)

```bash
cargo install sqlx-cli
```

Next create a `.env` file in the project root with these contents:

```
DATABASE_URL=sqlite://./test.sqlite
```

In order to sync your local environment to incoming query or schema changes run (if you see rust compile errors like `error: error returned from database` you probably need to run this command:

```bash
sqlx db reset -y
```

If you add or change queries run this command to generate new sqlx SQL files:

```
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
