---
name: dev-fresh-wallets
description: Launch a Sage dev build (`pnpm tauri:dev`) with its own temporary SAGE_ROOT, so the user can test with fresh wallets and settings without touching their real ones. Monitor the logs while it runs, and report what was notable when the user closes the app. Use when the user asks to run, launch or try the dev build with fresh, new, empty, clean, throwaway or temporary wallets or data, or in a separate profile or root, or invokes /dev-fresh-wallets. For the user's real wallets use dev-my-wallets instead, and for agent-driven browser checks use the sandbox.
---

# Dev build with fresh wallets

Launch the dev build with a `SAGE_ROOT` of its own, so the user starts with no wallets, default settings, and webview storage of its own. Their real wallets, settings and WalletConnect sessions aren't touched.

Follow `.claude/skills/dev-fresh-wallets/references/dev-session.md` for the whole session: pausing ongoing work, preflight, launch, monitoring, work requested while the app runs, and the report when it closes. This file covers only what's specific to a fresh root.

## The root

- By default, use a new directory, `$S/root`, inside the session directory. It's temporary. The scratchpad belongs to this session, and macOS clears `/private/tmp` on reboot.
- If the user asks for the same wallets as an earlier launch in this session, reuse the most recent root that has a `wallets.toml`: `ls -dt <scratchpad>/dev-session/*/root 2>/dev/null`. Roots from other sessions can't be found this way.
- If the user gives a path, use it as is, creating it if needed. That's how to keep a root across sessions, for example `~/sage-profiles/<name>`, which the README uses as its example.
- Pass an absolute path. `pnpm tauri:dev` starts the app from `src-tauri`, so a relative path would resolve against that directory.

The `<ENV>` for the launch command is `SAGE_ROOT="<absolute root>"`.

## Extra preflight

Nothing else needs checking. The root is new, so it can't be in use by another instance. RPC is off by default, so there's no port conflict with the user's installed Sage. Both can run side by side.

## While it runs

The root's own log at `<root>/log/app.log.*` contains the same lines as `dev.log` without the cargo and Vite output, so watch `dev.log` only. If the user wants to inspect state, the root holds `config.toml`, `wallets.toml`, `networks.toml` and `wallets/<fingerprint>/<network>.sqlite`. You can read the databases with `sqlite3`, but only after the app has closed or with `-readonly`.

The root stores fresh test keys in `keys.bin`. Don't copy them anywhere or print them.

## Report additions

When the app closes, also report:

- The root's path. If it's in the scratchpad, say that it can be reused for the rest of this session, and that to keep it longer the user can copy it somewhere durable and pass that path next time.
- What was created: wallet fingerprints and networks from `wallets.toml` and `wallets/`, and anything changed from the defaults in `config.toml`.

Leave the root in place. Delete it only if the user asks.
