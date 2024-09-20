# Sage Wallet

The sagest wallet for the Chia blockchain.

Sage is:

1. A high-performance light wallet that can connect directly to peers on the Chia network.
2. Built upon the reliable foundation of [chia_rs](https://github.com/Chia-Network/chia_rs), [clvm_rs](https://github.com/Chia-Network/clvm_rs), and the [Chia Wallet SDK](https://github.com/xch-dev/chia-wallet-sdk).
3. Designed with maintainability and future extensibility in mind from the beginning.

## Disclaimer

Sage is still in beta and has not been thoroughly tested yet. For now, it should only be used on testnet11, and you should back up your keys elsewhere.

The app is not guaranteed to be stable and can have breaking database and config changes between releases. If you are looking for stability, I recommend choosing a different wallet until Sage is out of beta.

## Installation

There are currently no pre-built executables for this app. In the meantime, you can see the [Development](#development) section for source installation instructions.

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
