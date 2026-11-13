# Testing The UI

- To check a UI change in a browser, run `pnpm sandbox` in the background and open the URLs from
  `.sandbox/instance.json` with the browser preview tools. Never launch the Sage desktop app for this.
- The sandbox has two wallets on a simulated chain. Use `bob` as the counterparty, either in a second
  tab or by calling his bridge with `POST <bridge>/bob/invoke/<command>`.
- See `docs/sandbox.md` for the bridge API, what the sandbox can't do, and how to extend it.
- Stop the sandbox when you are done.
