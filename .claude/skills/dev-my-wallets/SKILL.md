---
name: dev-my-wallets
description: Launch a Sage dev build (`pnpm tauri:dev`) without a SAGE_ROOT, so it uses the user's real default data directory with their actual wallets, settings and WalletConnect sessions. Monitor the logs while it runs, and report what was notable when the user closes the app. Use when the user asks to run, launch or try the dev build with their real, actual, own, normal or existing wallets or data, or invokes /dev-my-wallets. For fresh test wallets use dev-fresh-wallets instead.
---

# Dev build with the user's own wallets

Launch the dev build against the default root, `~/Library/Application Support/com.rigidnetwork.sage` on macOS. This is the same data the installed Sage uses, real keys included, so take more care here than with a fresh root.

Follow `.claude/skills/dev-fresh-wallets/references/dev-session.md` for the whole session: pausing ongoing work, preflight, launch, monitoring, work requested while the app runs, and the report when it closes. This file covers only what's specific to the real root.

## The root

The `<ENV>` for the launch command is `env -u SAGE_ROOT`. The user's shell profile might export `SAGE_ROOT`, and it would silently point the build somewhere else, so unset it explicitly instead of just leaving it out.

## Extra preflight

Run these together with the general preflight, before launching:

- **The installed Sage isn't running.** Two instances must not share a root. Check with `pgrep -fl sage-tauri`, and identify each process with `ps -o pid,command -p <pid>`. `/Applications/Sage.app/...` is the installed app, and `.../target/debug/sage-tauri` is another dev build, possibly from another worktree. If the installed app is running, ask the user to quit it; don't kill it. A dev build from another worktree might be using a different `SAGE_ROOT`. If you can't tell, ask.
- **Database migrations.** The dev build runs its migrations on every wallet database it opens, and the installed release won't open a database that has migrations it doesn't know about. Get the installed version (`defaults read /Applications/Sage.app/Contents/Info.plist CFBundleShortVersionString`), run `git fetch --tags --quiet`, and list the migrations added since then: `git diff --name-only --diff-filter=A v<version> HEAD -- migrations crates/sage-apps/src/db/migrations`. Include uncommitted new files from `git status --short` in the same directories. If there are any, ask the user before launching. Tell them which migrations they are, and that once a wallet is opened in this build, the installed Sage may fail to open that wallet until a release with those migrations ships. Don't continue without their go-ahead. A previous run might already have applied the same migrations, but you can't check that reliably while databases use WAL, so ask anyway and mention the possibility.
- **RPC port.** If `config.toml` in the root has the RPC server enabled, `lsof -nP -iTCP:<port> -sTCP:LISTEN` should be empty. Otherwise the build will fail to bind. Mention it if so.

## While it runs

- Never read, copy or print `keys.bin`, and don't put mnemonics, private keys or other secrets from the logs into messages. Redact them if they show up.
- Don't write to the root. Inspect the wallet databases only with `sqlite3 -readonly`, and only when the user asks.
- The root's own `log/app.log.*` mixes this session's lines with the installed app's earlier ones, so use `dev.log` for the report.

## Report additions

When the app closes, also report:

- Any migration that ran (look for migration or schema lines in the log), so the user knows which wallets were upgraded.
- Wallet-level errors tied to a fingerprint, such as sync failures, along with the fingerprint. Fingerprints are public, so they're fine to show.
