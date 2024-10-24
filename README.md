# Sage Wallet

The sagest wallet for the Chia blockchain.

Sage is:

1. A high-performance light wallet that can connect directly to peers on the Chia network.
2. Built upon the reliable foundation of [chia_rs](https://github.com/Chia-Network/chia_rs), [clvm_rs](https://github.com/Chia-Network/clvm_rs), and the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk).
3. Designed with maintainability and future extensibility in mind from the beginning.

## Disclaimer

Sage is still in beta testing and isn't guaranteed to be stable, so it should be used with caution. Please make sure you backup your keys and don't put too much XCH in mainnet wallets.

## Installation

You can download binaries for any platform (including desktop and Android) on the releases page. For iOS, you can participate in the [public TestFlight](https://testflight.apple.com/join/BmUdFXpP).

If you want to build from source, see the [Development](#development) section for instructions.

## Contributing

This is an open source project, and we welcome pull requests to improve any part of the wallet.

The frontend is currently written in TypeScript with [React](https://react.dev/) and [Material UI](https://mui.com/), and can be found in the `src` directory.

The backend is written in Rust, powered by [Tauri v2](https://v2.tauri.app/). The frontend and backend communicate via serialized commands and events over IPC.

Finally, the wallet driver code is written on the backend using the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk).

## Development

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

You can run the app in development mode with:

```bash
pnpm tauri dev
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
