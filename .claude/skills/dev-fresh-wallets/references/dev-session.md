# Running a dev session

Both dev skills (`dev-fresh-wallets` and `dev-my-wallets`) follow the steps below. The only difference is the `SAGE_ROOT` the launch gets, plus the preflight checks each skill adds. In the commands, `$S` is the session directory, `<scratchpad>/dev-session/<YYYYMMDD-HHMMSS>/`, and `$REPO` is the worktree root. Shell state doesn't carry between Bash calls, so write the full paths into every command instead of relying on variables.

The user wants to try the app by hand. Your job while it runs is to stay out of the way, watch the logs, and help when something goes wrong.

## 1. Pause ongoing work

If the skill was invoked partway through other work (implementing something, a review loop, running checks), pause it before you launch:

- Finish the tool call that's running and any edit you're halfway through, so the tree builds. Don't start new edits.
- Stop background checks that compile Rust, such as `cargo clippy` or `cargo test`. They hold the build directory lock and would hold up the dev build. Note which ones you stopped so you can rerun them later. You can't pause running delegates or subagents. Let them finish, but don't act on what they return until the work resumes.
- Write `$S/paused.md`: what you were doing, what's done, the next step, and any checks you stopped. That way the work can resume correctly even after the context has been summarized.

The work stays paused until the user says to continue or closes the app (section 6). A message asking for something new while the app is running isn't "continue" for the paused work. Do what the message asks (section 5) and leave the paused work alone.

## 2. Preflight

Run these in one go:

- **What's being built.** Get the branch, `git rev-parse --short HEAD`, and `git status --short`. Uncommitted changes are part of the build, so tell the user up front if there are any.
- **Ports.** Vite needs 1420 and the system-apps watcher needs 1421, and both use strict ports: `lsof -nP -iTCP:1420 -iTCP:1421 -sTCP:LISTEN`. If either is taken, find out what holds it (`lsof -a -p <pid> -d cwd`). It's usually a dev session running from another Sage worktree. Tell the user and ask before stopping it. Never kill a process you didn't start without asking.
- **Dependencies.** If `node_modules` is missing, or `pnpm-lock.yaml` is newer than `node_modules/.modules.yaml`, run `pnpm install` first.

## 3. Launch

Start the dev command detached, so the session isn't tied to a tool's timeout, and record its pid and exit code:

```bash
mkdir -p "$S" && cd "$REPO" && nohup sh -c '<ENV> pnpm tauri:dev <ARGS> > "$S/dev.log" 2>&1; echo "exit=$?" > "$S/exit"' > /dev/null 2>&1 < /dev/null &
echo $! > "$S/pid"
```

`<ENV>` comes from the skill. `<ARGS>` are extra arguments the user gave, such as `--release`, which `pnpm tauri:dev` passes through to `tauri dev`. Write `$S` out as an absolute path inside the `sh -c` string.

Then start two watchers:

1. **Exit waiter.** A Bash call with `run_in_background: true` and `timeout: 7200000`:
   `while kill -0 "$(cat $S/pid)" 2>/dev/null; do sleep 2; done; cat $S/exit`.
   It finishes when the dev command exits, which happens when the user closes the app, and you're re-invoked when it does. Closing the window quits the app, and `tauri dev` exits with it. A rebuild restarts the app without stopping `tauri dev`, so rebuilds don't trigger the waiter. If the waiter hits its timeout while the pid is still alive, start it again. Don't call `t3_background_wait` on it: that would keep the thread busy, and the user should be able to message you while they test.

2. **Log monitor.** A Monitor with `timeout_ms: 1800000` that strips ANSI codes and passes on only the lines worth acting on:

   ```bash
   tail -n +1 -F "$S/dev.log" \
     | perl -pe 'BEGIN { $| = 1 } s/\e\[[0-9;?]*[A-Za-z]//g' \
     | grep -E --line-buffered ' (ERROR|WARN) |panicked|^error(\[E[0-9]+\])?:|Running `|Rebuilding|Sage wallet initialized|Internal server error|Pre-transform error|already in use|ELIFECYCLE|Error:'
   ```

   When it expires, re-arm it with `tail -n 0 -F` so lines you've already seen aren't sent again. If a line keeps repeating and floods the monitor (peer connection warnings are the usual cause), re-arm with a `grep -v` for it. Still count it in the final report.

Tell the user that the build is starting, what's being built (branch, HEAD, whether the tree is dirty), the root it's using, and that you'll report when they close the app. Then end your turn. The first build after a Rust change can take a few minutes. `Running \`target/...\`` means the window is about to open, so tell the user it's up when you see that line.

## 4. While it runs

Each monitor event re-invokes you. Keep responses short. The user is busy in the app.

- **Startup failures.** These include a port in use, a compile error, a failed `dev:prepare`, or the waiter finishing within seconds of launch. Read the end of `dev.log`, explain what went wrong, and fix it if it's environmental (for example, a missing `pnpm install`). If it's a compile error in the user's changes, tell them; don't fix it unless they ask.
- **Panics and ERROR lines.** Look up the code that logged the line (read-only) and check whether the branch touches it (`git diff origin/main...HEAD`, plus uncommitted changes). Then tell the user in a line or two what happened and the likely cause. Group repeats instead of reporting each one.
- **WARN lines.** Note them for the final report. Mention them now only if they look caused by the branch or might explain what the user is seeing.
- **Things the log can't show.** Frontend console errors and failed bridge calls show up only in the webview console. If the user reports a UI problem that isn't in the log, ask them to right-click and choose Inspect Element (debug builds have devtools) and paste what the console shows.
- **No edits on your own.** Any file change in the worktree reloads the frontend or rebuilds and restarts the app while the user is using it. Only change files when the user asks (section 5).

## 5. Work requested while the app runs

When the user asks for changes while the app is running, make them in this worktree and keep the dev command running. Don't stop or restart it, since it picks up the changes itself:

- Frontend changes (`src/`) hot-reload in place.
- Rust changes in `src-tauri` or the workspace crates rebuild the app and restart it. A restart closes and reopens the window and locks the wallet again, so tell the user before you save a Rust change. Batch the edits so the app restarts once, not once per file.
- Built-in system apps and the app SDK packages are rebuilt by the `dev:system-apps` watcher.
- `dev:prepare` runs only at launch, so it won't pick up changes to `builtin-apps/src/runtime`, `builtin-apps/src/sandbox-test` or the app SDK's generated types (from `crates/sage-apps`). If you change those, run `pnpm dev:prepare` yourself; don't restart the dev command.
- If a command signature changes, run `pnpm generate:bindings`. It doesn't launch Sage.

After each batch, watch the monitor for the result (an HMR update, `Rebuilding` followed by `Running`, or a compile error). Tell the user when the change is live. If it broke the build, fix it, because you caused it. Running the validation checks is fine. Run Rust checks after the rebuild finishes, since they contend for the same build lock.

## 6. When the app closes

The exit waiter finishing re-invokes you. Stop the log monitor (TaskStop). Then check for leftover processes: `lsof -nP -iTCP:1420 -iTCP:1421 -sTCP:LISTEN` should show nothing from this session. If it does, kill only the processes your launch started (check the cwd as in section 2).

Read the whole log, not just the events the monitor sent:

```bash
perl -pe 's/\e\[[0-9;?]*[A-Za-z]//g' "$S/dev.log" > "$S/clean.log"
grep -E ' (ERROR|WARN) ' "$S/clean.log" | sed -E 's/^[^ ]+ +//' | sort | uniq -c | sort -rn
grep -nE 'panicked|^error(\[E[0-9]+\])?:|Rebuilding|Running `|Internal server error' "$S/clean.log"
cat "$S/exit"
```

Report:

- **How it ended.** A clean close is `exit=0`. Anything else, or a panic near the end of the log, is a crash, so quote the relevant lines.
- **Session length and rebuilds**, and whether each rebuild succeeded.
- **Panics and errors.** Deduplicate them, give counts, and for each one say whether it's likely caused by the branch or is ambient (for example, peers dropping or a node timing out). Tie them to code where you can.
- **Warnings worth knowing about.** Count the rest instead of listing them.
- **Frontend errors** from Vite.
- **Anything else unusual**, such as slow sync or a lot of reconnects.

Say there was nothing notable if there wasn't, and give the path to `dev.log` either way. Add any skill-specific items, then resume the paused work if there is any (read `$S/paused.md`, rerun the checks you stopped, and continue).
