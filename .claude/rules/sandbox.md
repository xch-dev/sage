# Testing The UI

- To check a UI change in a browser, run `pnpm sandbox` in the background and open the URLs from
  `.sandbox/instance.json` with the browser preview tools. Never launch the Sage desktop app for this.
- This repo asks for browser checks of UI changes, so use the preview tools for them even when you
  weren't asked to. Open the sandbox with `open: true` so the user can watch, and leave the tab open
  when you reply, so they can look at the result.
- The sandbox has two wallets on a simulated chain. Use `bob` as the counterparty, either in a second
  tab or by calling his bridge with `POST <bridge>/bob/invoke/<command>`.
- See `docs/sandbox.md` for the bridge API, what the sandbox can't do, and how to extend it.
- Stop the sandbox when you are done, unless you left a preview tab open on it. In that case, keep it
  running, tell the user so, and stop it when they're done with it.
