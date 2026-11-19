---
name: dev-my-wallets
description: Launch a Sage dev build (`pnpm tauri:dev`) without a SAGE_ROOT, so it uses the user's real default data directory with their actual keys, wallets and backend config. Monitor the logs while it runs, and report what was notable when the user closes the app. Use when the user asks to run, launch or try the dev build with their real, actual, own, normal or existing wallets or data, or invokes /dev-my-wallets. For fresh test wallets use dev-fresh-wallets instead.
---

# Dev build with the user's own wallets

Launch the dev build against the default root, `~/Library/Application Support/com.rigidnetwork.sage` on macOS. This root holds the same keys, wallet databases and `config.toml` the installed Sage uses, real keys included, so take more care here than with a fresh root.

Frontend state is not shared with the installed app. Settings stored in the webview, such as the default fee, and WalletConnect sessions are kept per origin, and the dev build loads from `http://localhost:1420`. So the dev build has its own copy of that state, shared with every other dev build run without a `SAGE_ROOT`. If the user wants to test with their existing WalletConnect sessions, tell them those won't be there.

Follow `.claude/skills/dev-fresh-wallets/references/dev-session.md` for the whole session: pausing ongoing work, preflight, launch, monitoring, work requested while the app runs, and the report when it closes. This file covers only what's specific to the real root.

## The root

The `<ENV>` for the launch command is `env -u SAGE_ROOT`. The user's shell profile might export `SAGE_ROOT`, and it would silently point the build somewhere else, so unset it explicitly instead of just leaving it out.

## Extra preflight

Run these together with the general preflight, before launching:

- **The installed Sage isn't running.** Two instances must not share a root. Check with `pgrep -fl sage-tauri`, and identify each process with `ps -o pid,command -p <pid>`. `/Applications/Sage.app/...` is the installed app, and `.../target/debug/sage-tauri` is another dev build, possibly from another worktree. If the installed app is running, ask the user to quit it; don't kill it. A dev build from another worktree might be using a different `SAGE_ROOT`. If you can't tell, ask. Recent builds also lock the root with `sage.lock`, but the installed release may be too old to do that.
- **Database migrations, in both directions.** Any migration error makes Sage offer to delete and resync the wallet database, behind a "Database Migration Required" dialog whose OK button is focused, so a single Enter wipes it. That dialog appears in two cases:
  - _A database is newer than this build._ This happens when the branch is behind a build that already ran on this root, such as another worktree's. In this case the dev build offers the reset, or panics at startup if it's `sage-apps.sqlite3`.
  - _This build migrates a database past the installed release._ In this case the installed release offers the reset the next time it opens that wallet. For `sage-apps.sqlite3`, the installed release panics at startup instead. That lasts until a release with those migrations ships.

  A migration that changed or failed causes the same dialog, because sqlx compares the checksum of every applied migration with the build's file and rejects a failed one. That happens, for example, when two branches each add a different migration with the same number, or a migration is edited after an earlier dev run applied it.

  The app must be closed (see above) for the check. Get the installed release's version with `defaults read /Applications/Sage.app/Contents/Info.plist CFBundleShortVersionString`, and run `git fetch --tags --quiet`. Then run this from the repository root. It never writes to the databases or creates them:

  ```bash
  python3 -I .claude/skills/dev-my-wallets/scripts/check_migrations.py --release v<version>
  ```

  If Sage isn't installed in `/Applications`, leave out `--release`. The script prints the active wallet, and one line per database. Launching opens the active wallet's databases, the shared databases and `sage-apps.sqlite3`; other wallets are opened only when the user switches to them. It exits with status 2 if launching would bring up the dialog, 1 if this build will apply migrations the release doesn't have, or that it can't compare with a release, and 0 otherwise. Act on every line:
  - **"WOULD SHOW THE RESET DIALOG":** don't launch. Tell the user which databases are affected and why. If a migration is applied that the build doesn't have, the branch is behind whatever migrated the database, and usually needs a rebase onto `origin/main`. If a migration differs or failed, another branch's or an older build's version of it was applied. Tell them never to confirm a "Database Migration Required" dialog in a dev build on their real data, and let them decide what to do.
  - **"would show the reset dialog if switched to":** launching is fine. List those wallets and tell the user not to switch to them in this build, or to cancel the dialog if they do. "(it fails in v... too)" means the installed release already can't open them, so this build doesn't make it worse.
  - **"will apply" or "will be created with" migrations the release lacks:** ask before launching. Say which migrations and how many databases. Wallet and shared databases are only migrated when a wallet on that network is opened, while `sage-apps.sqlite3` is migrated at launch. Explain what happens to the installed release afterwards, as described above. If a migration isn't released yet, the release also fails if that migration changes before it ships. Don't continue without their go-ahead.
  - **"release comparison skipped",** or no `--release`: say so. Treat every pending migration as one the release lacks.
  - **"unreadable":** say which database and why, and treat it as unknown when deciding. An unreadable active wallet or shared database is a reason to ask before launching.
  - Keep the output. The report compares against it.

- **RPC port.** If `config.toml` in the root has the RPC server enabled, `lsof -nP -iTCP:<port> -sTCP:LISTEN` should be empty. Otherwise the build will fail to bind. Mention it if so.

## While it runs

- Never read, copy or print `keys.bin`, and don't put mnemonics, private keys or other secrets from the logs into messages. Redact them if they show up.
- Don't write to the root. Inspect the wallet databases only with `sqlite3 -readonly`, and only when the user asks.
- If the app shows a "Database Migration Required" dialog, tell the user right away not to confirm it, and explain why.
- The root's own `log/app.log.*` mixes this session's lines with the installed app's earlier ones, so use `dev.log` for the report.

## Report additions

When the app closes, also report:

- **Which databases were migrated.** Run the script again and compare its output with the preflight's. sqlx doesn't log migrations. The log's "Migrating to version" lines are Sage's own data migrations and are worth listing too.
- **Wallet-level errors tied to a fingerprint**, such as sync failures, along with the fingerprint. Fingerprints are public, so they're fine to show.
