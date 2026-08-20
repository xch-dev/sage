# `sage-app-sdk`

TypeScript SDK for building apps that run inside [Sage Wallet](https://github.com/xch-dev/sage).

> Sage Apps are a work in progress. APIs are mostly stable, but minor changes may still occur.

## Install

```sh
npm install sage-app-sdk
```

## Use the Sage client

```ts
import { getSageClient } from 'sage-app-sdk';

const client = await getSageClient();
```

The SDK also provides Sage runtime integration, generated bridge types, lifecycle hooks, and theme utilities.

## Finalize an app manifest

The package installs the `sage-app` command. Add a script to your app:

```json
{
  "scripts": {
    "sage:finalize": "sage-app finalize-manifest --source ./sage-manifest.json --dist ./dist"
  }
}
```

Build the app, then run:

```sh
npm run sage:finalize
```

This writes the finalized `sage-manifest.json` into the app's distribution directory.

## License

[Apache-2.0](https://github.com/xch-dev/sage/blob/main/LICENSE)
