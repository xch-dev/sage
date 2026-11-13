---
name: review-loop
description: Take a piece of Sage work (an issue, a feature, a bug fix, a follow-up to an open PR) all the way to a reviewed PR that passes the same checks as CI, running rounds of independent code review and fixing what they find, without the user having to ask for each review pass. Use this whenever the user asks to implement something "with reviews", to "drive this to completion", "review until clean", "keep reviewing and fixing", "take this through review", or hands over a task in the Sage repo and expects a finished, reviewed PR rather than a first draft, even if they don't say "review loop". Not for a one-off review of an existing diff with no fix loop; use code-review for that.
---

# Review loop

The user's usual workflow on Sage was: an agent implements something, then the user asks for a review pass, the agent fixes the findings, and this repeats until the work is clean. This skill runs that loop for them. Past rounds caught real bugs, missing tests and rough edges in features like pre-split offers, option cancellation, handle recipients and NFT metadata updaters (`git log --oneline origin/main | grep -i review`).

The loop is: **scope and write the brief → implement → review → verify and fix → repeat until a stop condition → final checks → commit, PR and report.**

## 1. Scope the work and write the state file

Read the source first: `gh issue view <n> --comments` or `gh pr view <n> --comments`, plus the code it touches.

**Tracking issues.** If the issue lists several independent items, don't put them all on one branch. Do the items the request points to. If it names only the issue, ask which items: that's the one question worth asking up front, since a wrong guess wastes the whole run. Give each item its own branch, state file and loop, stack the branches with `gh stack`, and link the issue with "Part of #n" rather than "Closes #n".

**Open choices.** The user prefers that you make decisions and bring them up at the end, rather than stopping to ask. When the source leaves open anything on the reviewer brief's "for the user" list (UX and copy, RPC and public formats, database schema, wallet behaviour, scope), make the call: pick what fits the issue's intent and the neighbouring code, and the more conservative option when it's close. Record it under Decisions as provisional (see the template), and keep going. Ask up front only when no reasonable choice exists, such as an issue whose goals contradict each other.

**Follow-ups to an open PR.** Do what the user asked; if they didn't say, continue on their own `rigidity/` branches and stack on anyone else's. Base is set so that reviewers see only the new work.

- _Continuing on the PR's branch_ (`gh pr checkout <n>`): keep its branch name, set Base to the commit the follow-up starts from, and push to that PR in section 7.
- _Stacking on top_: if the PR is already in a stack, `gh stack checkout <n>`; otherwise `gh pr checkout <n>` first, so its branch exists locally at the PR's head, then adopt it with `gh stack init <pr-branch>`. Then `gh stack add rigidity/<slug>`. Base is the parent branch, which stays right after a restack. The new branch gets its own PR in section 7, linking the original. A PR from a fork can't be stacked this way; continue on a new branch from its head, open a PR against the original PR's base that links it, and say so in the report.

The state file is the memory of the loop. Reviewers start with fresh context, and after a few rounds the implementer's own context is long, so without it later rounds re-raise settled points or undo earlier decisions. Write it to the session scratchpad (not the repo), at `<scratchpad>/review-loop/<branch-slug>.md`. Every review round reads it, and every fix round updates it.

```markdown
# <short title>

## Brief (shared with reviewers)

- Goal: <what this change must achieve, in the user's terms>
- Serves: <issue/PR link>
- Base: <origin/main, the parent branch in a gh stack, or the commit a PR follow-up starts from>
- Branch: <rigidity/...>
- Decisions: <each decision and why, noting who made it: the issue, the user, or the implementer. Mark choices that are the user's to confirm as "provisional, for the user: X, because Y; the alternative is Z". Include generated files that changed (`src/bindings.ts`, `.sqlx/`, `docs/generated/`) and why.>
- Out of scope: <things deliberately left alone, and why>
- Settled: <points the user or the issue decided; not open for review. Always include the accepted security risks: the keychain's empty password, biometric/password auth enforced only in the frontend, and unvalidated coin-state updates from peers.>
- Previously rejected: <findings the implementer rejected in earlier rounds: the claim and the evidence>

## Decisions for the user

- <each provisional decision: the options, what you picked and why, and what switching would take>

## Review log (implementer only)

### Round 1

- [confirmed, major] <finding> -> fixed in <commit>, regression test <where>
- [rejected] <finding> -> <evidence>
- [deferred] <finding> -> <why, and where it's recorded>
```

## 2. Implement

Work on a `rigidity/` branch (lowercase; never `Rigidity/`), unless you're continuing someone else's PR branch. For a bug fix, write the failing regression test first. The usual places:

- **Endpoints**: request and response types in `crates/sage-api` (`requests/`, `records/`, `types/`), and the handler as a `Sage` method in `crates/sage/src/endpoints/<area>.rs`. List it in `crates/sage-api/endpoints.json`, which generates the RPC route, client, CLI and Tauri command (`impl_endpoints!`, `impl_endpoints_tauri!`), or in `endpoints-tauri.json` for a Tauri command left out of RPC, client and CLI. The few commands written by hand in `src-tauri/src/commands.rs` are in neither file. Tests go in `crates/sage-rpc/src/tests/` (the RPC against the peer simulator) or inline in `crates/sage/src`.
- **Registering Tauri commands** (fails only at runtime if missed): add every new command to `src-tauri/src/lib.rs`, in `sage_commands!` for `commands::*` (shared with mobile) or in `specta_builder()`'s desktop-only list for `apps::*` Sage Apps host commands, and allow it in `src-tauri/permissions/main-host.toml` (or `bridge.toml` for app webviews). The `permissions_allow_every_command` test catches a registered command that isn't allowed, but not one that was never registered.
- **Database**: queries in `crates/sage-database/src/tables/`. Schema changes go in a new file under `migrations/`; never edit an existing migration, since wallets in the field have already run it. Migration and backfill tests go in `crates/sage-database/src/tests.rs`. `cargo sqlx prepare` needs a `.env` in the worktree root with `DATABASE_URL=sqlite://./test.sqlite` and a fresh `sqlx db reset -y` (see the README); a new worktree has no `.env`.
- **Wallet and sync**: `crates/sage-wallet` (`wallet/` for spends, `sync_manager/` and `queues/` for syncing). Tests use the peer simulator in `crates/sage-wallet/src/test.rs`; follow the neighbouring tests rather than building a new harness.
- **Frontend**: pages in `src/pages`, shared pieces in `src/components`, `src/hooks` and `src/lib`, WalletConnect in `src/walletconnect`. Unit tests sit next to the file as `*.test.ts(x)` and run with Vitest. Wrap new strings with Lingui as `.claude/rules/translations.md` says. Run `pnpm install --frozen-lockfile` before the first frontend command; a new worktree's `node_modules` may be empty or stale.
- **Sage Apps**: the bridge, manifests and permissions in `crates/sage-apps` (tests inline), the SDKs in `packages/` and the built-in apps in `builtin-apps/`. `pnpm generate:types` refreshes the gitignored SDK types for local type checks; `pnpm build` regenerates them itself.

Follow `.claude/rules/*.md` (imports, generated artifacts, translations, validation). Use what's already there before writing helpers: look for an existing query, endpoint, hook or component that does most of the job, and follow the neighbouring code. A feature that exists on several surfaces (the Tauri UI, RPC and `sage-cli`, WalletConnect, the Sage Apps bridge) usually needs the change on each.

Before each review round, including the first, regenerate what the change affects, check `git status --short`, and commit the output with the change that caused it, so reviewers see the real diff:

- a `sage-api` type or Tauri command: `pnpm generate:bindings` (`src/bindings.ts`)
- a `query!` or a migration: `cargo sqlx prepare --workspace` (`.sqlx/`)
- a Sage Apps bridge method or capability: `pnpm generate:app-docs` (`docs/generated/`)

Read each regenerated file: a change you didn't expect usually means a mistake.

Commit before the first review round, so each round's fixes are a separate diff.

## 3. Review rounds

Each round, launch three reviewers **in parallel** with the Agent tool (`general-purpose`, `run_in_background: false`), one per focus:

1. **Correctness**: wrong results, lost or misreported funds, broken sync or database state, crashes, and UI that shows or does the wrong thing.
2. **Tests**: whether the tests would catch a regression in each behaviour the change adds, and whether generated files changed as intended.
3. **Simplification and reuse**: duplicated logic, existing helpers that should be used, needless work, structure that doesn't match the surrounding code.

For a small diff (roughly under 100 changed lines), one reviewer covering all three focuses is enough; three reviewers on a ten-line change mostly produce nits.

Build each prompt from [references/reviewer-brief.md](references/reviewer-brief.md), which holds the template, the Sage checklist for each focus, the severities, and which choices are for the user. Give reviewers the Brief section, the base ref, and the focus, and nothing else: not your summary of the change, your reasoning about why it's correct, or the review log. They read the diff (`git diff <base>...HEAD`) and form their own view, and that independence is what makes a round worth running. From round 2 they still review the whole diff against the base, since fixes break things elsewhere; name the areas that changed since the last round so they look closely there.

The built-in `code-review` skill is an alternative for the correctness focus on a large diff (`/code-review high`), but it doesn't see the Brief, so check its findings against the goal and the Settled points yourself.

## 4. Verify, then fix

Reviewers are sometimes confidently wrong. Before acting on a finding, check it:

- For a claimed bug, reproduce it: write a failing test that shows it, in the places section 2 names for that area (`sage-rpc` tests for endpoints, the `sage-wallet` simulator, `sage-database`, `sage-apps`, or Vitest for the frontend). A finding you can't reproduce is rejected unless you can see from the code that it's right. Don't launch the Sage GUI to check a claim.
- For a claimed missing test, check whether an existing case already covers it.
- For a simplification, check that the suggested helper does what the reviewer thinks.
- For a finding marked "for the user", it's confirmed if the choice really is open: neither the source nor Settled answers it. Handle it as in section 5.

Mark each finding in the review log as **confirmed**, **rejected** (with the evidence), or **deferred** (real but out of scope, with where it's recorded), and add rejected ones to the Brief's Previously rejected list and deferred ones to Out of scope (with where they're recorded), so later rounds don't raise them again. Fix every confirmed blocker and major finding, and minor ones when they're cheap. Each confirmed correctness finding gets a regression test. Nits are optional.

Then run the quick checks for what you touched (`cargo test -p <crate>`, `pnpm exec vitest run <path>`), regenerate generated files as in section 2, update the Brief, and commit (section 7).

## 5. When to stop

- **Clean**: a round with no confirmed findings at major or above. Stop there; never start a round just for minor findings or nits.
- **Out of rounds**: stop after round 3 even if findings remain, and list them in the report. If round 3's fixes were more than trivial, first run one more round with only the correctness reviewer, to check the fixes; fix what it confirms and report it, without another round. A change that still has major findings after three rounds usually has a design problem that another round won't fix, and the user should look.

Whatever the stop, still run the final checks and open or update the PR, so the user has the work in hand.

**Decisions during the loop.** These don't end the loop or interrupt the user. When a confirmed finding raises a choice that's the user's to confirm (the list is in the reviewer brief), make it as in section 1: keep the current choice or switch, whichever the evidence favours, record it under Decisions as provisional so later reviewers don't raise it again, and add it to Decisions for the user. A reviewer's new argument against a provisional decision goes into that entry, so the user sees both sides. The PR and the report carry every provisional decision, and the user picks at the end.

## 6. Final checks

These are the checks in the Tests job of `.github/workflows/build.yml`, plus `-D warnings` on clippy (stricter than CI), `pnpm lint` (required by `.claude/rules/validation.md`), the type check and the frontend unit tests. Fix anything they report.

Always, since CI runs it on every PR:

```bash
pnpm install --frozen-lockfile
pnpm prettier:check
```

If any Rust input changed (`*.rs`, `Cargo.toml`, `Cargo.lock`, `migrations/`, `.sqlx/`). `SQLX_OFFLINE=true` makes them read the committed `.sqlx/` as CI does, rather than a local `.env` database:

```bash
SQLX_OFFLINE=true cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all -- --files-with-diff --check
SQLX_OFFLINE=true cargo test --workspace --all-features
cargo machete --skip-target-dir
```

If any TypeScript changed, including a regenerated `src/bindings.ts`:

```bash
pnpm lint
pnpm exec tsc -b
pnpm test:unit
```

- If you changed a `query!` or a migration, check that `cargo sqlx prepare --workspace --check` passes, so the committed `.sqlx/` matches.
- If you changed a type in `sage-api` or a Tauri command, check that `pnpm generate:bindings` leaves `src/bindings.ts` unchanged after your commit.
- If you changed the Sage Apps bridge, check that `pnpm generate:app-docs` leaves `docs/generated/` unchanged.
- CI's desktop builds also run `pnpm build` (SDKs, builtin apps and the frontend; mobile runs `build:frontend`). Run it if you changed `crates/sage-apps`, `packages/`, `builtin-apps/` or the build scripts, and say in the report if you didn't.
- Local clippy only compiles for the host platform. If you changed `cfg(mobile)`, `cfg(not(mobile))` or other platform-gated code, say in the report that it wasn't checked locally.
- If these checks led to changes beyond formatting, say so in the report, since no reviewer saw them.
- Report any check that failed for reasons unrelated to the change rather than calling the run clean.

## 7. Commits, PR and report

**Commits.** One focused commit per logical change. Messages follow the git log: a plain sentence as the subject that says what changed ("Bind coin IDs as query parameters instead of formatting them into SQL"), or "Address review findings on <area>" with a `-` bulleted body when one round fixes several small things. No prefix like `feat:`. No AI attribution of any kind: no `Co-Authored-By`, no "Generated with" line.

**PR.** Push and open a PR, ready for review rather than a draft, with `gh pr create --base <branch>`, where the branch is Base without the `origin/` prefix (`main`, not `origin/main`), or push to the existing PR when continuing on its branch. The title follows the same style; the body says what changed and why, links the issue ("Closes #n", or "Part of #n" for a tracking issue) or the PR it follows up, lists the provisional decisions for the user, and notes that the next release needs `pnpm extract` if new strings were added. For stacked work, use `gh stack` rather than setting PR bases by hand: `gh stack submit --auto --open` creates the PRs with generated titles (without `--open` they're drafts), so set each title and body afterwards with `gh pr edit <n> --title ... --body ...`.

**Report.** End with this, in the reply to the user:

```markdown
## <title> — <PR link>

<two or three sentences: what was done, and what the user should look at first>

### Review rounds

- Round 1: <n> findings, <n> confirmed. Caught: <the substantive ones, one line each>
- Round 2: ...
- Stopped because: <clean / out of rounds>

### Rejected findings

- <finding> — <why it was rejected>

### Left for you

- <each provisional decision: what you picked and why, the alternatives, and what switching would take>
- <confirmed findings left unfixed, if the loop ran out of rounds>

### Checks

<each final check and whether it passed; note generated files that changed and that they're intended>
```

Keep the state file until the user has seen the report; it's the place to look if they ask why something was decided.
