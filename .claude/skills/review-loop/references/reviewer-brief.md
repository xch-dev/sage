# Reviewer brief

Build each reviewer's prompt from the template below, filling in the Brief section of the state file, the base ref, the focus, and the checklist for that focus. Leave out the implementer's summary and the review log.

## Prompt template

```text
You are reviewing a change to Sage, a Chia light wallet (Rust backend, Tauri, React frontend; repo: xch-dev/sage), with fresh eyes. You have not seen the implementation being written, and that's on purpose: form your own view of the code.

What the change is for:
<the Brief section of the state file, verbatim>

Read the change with `git diff <base>...HEAD` (and `git log <base>..HEAD` for the commit messages). Read the surrounding code as needed. <For round 2+: "Since the last round, these areas changed: <files/functions>. Look closely there, but review the whole diff.">

Your focus this round: <Correctness | Tests | Simplification and reuse>. Other reviewers cover the other areas, so stay on yours.
<Or, with a single reviewer: "You cover all three focuses: correctness, tests, and simplification and reuse.">

<the checklist for that focus, from below, or all three for a single reviewer>

Rules:
- Don't edit files in the repo. You may write scratch files to <scratchpad>/review-loop/scratch/ and run tests to check a claim (`cargo test -p <crate> <name>`, `pnpm exec vitest run <path>`). Don't launch the Sage app.
- Judge the change against the goal above. Points under "Settled" were decided by the user or the issue; don't re-raise them. Points under "Previously rejected" were rejected by the implementer; raise one again only if you have evidence the rejection is wrong, and say what it is. Decisions marked "provisional, for the user" will be put to the user at the end; raise one only with an argument the entry doesn't already give, and it will be passed on to them.
- Prefer a few findings you're confident in over many guesses. If you aren't sure, say so in the finding.
- If there's nothing worth reporting in your focus, say "No findings" and stop.

Report each finding in this format:

### <one-line claim>
- Severity: blocker | major | minor | nit
- Where: <path:line>
- Scenario: <a concrete input (wallet state, request, user action) and the wrong result it gives, or for a non-bug, what's wrong and what it costs>
- Fix: <what to change>
- For the user: yes/no (see "Choices for the user" below)
```

Severities: **blocker** means lost or locked funds, a wrong spend or offer, corrupted wallet data, a crash, or a migration that breaks existing wallets; **major** means a real bug in a less common case, a missing test for core behaviour, or a design that will clearly need redoing; **minor** means a small bug in a rare case, a missing edge-case test, or a clear simplification; **nit** means style or naming.

## Choices for the user

Mark "For the user: yes" when the fix depends on a choice the user should confirm, and that Settled doesn't already answer. The implementer makes a provisional choice and the user picks at the end, so this doesn't block anything:

- UX: what the user sees, the wording of new strings, defaults, and confirmations before an action.
- Public formats: RPC endpoints and their request/response shapes, WalletConnect and Sage Apps bridge methods, config files, and anything other wallets or dApps depend on.
- Database schema and migrations, beyond what the goal needs.
- Wallet behaviour with a trade-off: fees, coin selection, which coins are spent or locked, and what's trusted from peers.
- Changing the scope of the issue, or behaviour outside it.

Include this section in every reviewer prompt, after the finding format.

## Checklist: correctness

General:

- Does the change do what the goal says, all of it, and nothing outside it?
- Edge cases: empty wallets, zero amounts, amounts near `u64::MAX` (mojos), CATs and NFTs alongside XCH, unsynced or partially synced wallets, hardened and unhardened derivations, coins that are pending or locked in an offer.
- Errors: is every failure reported to the caller with a useful message, rather than panicking (`unwrap`, indexing) or being swallowed?
- Untrusted input: data from offers (what the maker claims about assets), NFT and CAT metadata and names, external HTTP APIs, and dApps. Is it validated before it's trusted or shown, and can spam, invisible characters or a lookalike name mislead the user?
- Concurrency: is the `Sage` lock held across network calls? Can two requests read then write the same state and race?
- External network calls: do they have timeouts, tell transient errors (timeouts, 429, 5xx) from permanent ones, and avoid caching a temporary failure so it's never retried?
- Surfaces: if the feature exists in the Tauri UI, RPC and `sage-cli`, WalletConnect and the Sage Apps bridge, does the change behave the same on each?

Wallet and spends:

- Do the amounts balance: inputs = outputs + fee, with change sent to the right address? Can an overflow or a rounding step change an amount?
- Are coins selected from the right asset and excluded when already spent, pending or locked in an offer?
- Offers: are requested and offered amounts, royalties and fees what the user approved?
- Sync and queues: does the change hold up when a peer disconnects, times out, sends the same update twice, or the chain reorgs? Can a transaction stay pending forever or be resubmitted indefinitely?

Database:

- Are new queries parameterized (never values formatted into SQL), and do they hit an index where the table can be large?
- Is a new migration safe on an existing wallet database with data in it, and is it a new file rather than an edit to an old one?
- Are multi-step writes in a transaction where a partial write would leave the wallet inconsistent?

API and frontend:

- Do `sage-api` types, the handler, the entry in `endpoints.json` or `endpoints-tauri.json`, and `src/bindings.ts` agree? Is every new Tauri command registered in `src-tauri/src/lib.rs` (`sage_commands!` for `commands::*`, the desktop-only list for `apps::*`) and allowed in `src-tauri/permissions/*.toml`?
- Does the UI handle loading, error and empty states, and show amounts with the right precision for the asset? Can a stale result from an earlier request overwrite a newer one, the same work be queued twice, or form state be lost when the user toggles something?
- Are new user-facing strings wrapped with Lingui (`t`, `<Trans>`)? The `.po` catalogs should _not_ be changed in this PR.
- Sage Apps and WalletConnect: does every method that moves funds or signs anything go through user approval, and does the approval show what will actually happen? Can a dApp or app learn more than it should (for example by probing lookups) or call a method often enough to abuse it?
- Does any new way into offers or options check `offersEnabled` / `optionsEnabled` from `src/lib/features.ts`? The iOS App Store build turns them off.

The accepted risks listed under Settled are not open; don't report them, but do report problems next to them. Only those exact risks are settled: other untrusted input is still in scope.

## Checklist: tests

- Is there a test for each behaviour the change adds, including the cases where nothing should happen? Endpoints go in `crates/sage-rpc/src/tests/` or inline tests in `crates/sage/src`, wallet changes in `sage-wallet` (the peer simulator in `src/test.rs`), queries and migrations in `sage-database` (`src/tests.rs`), bridge and app logic inline in `sage-apps`, frontend logic in a `*.test.ts(x)` next to the file.
- Does each test actually exercise what it's named for, and assert on the result (balances, coin states, rows, rendered text) rather than only that nothing errored?
- Would each test fail if the behaviour it covers regressed? A test that passes either way is worth a finding.
- Bug fixes: is there a regression test that fails without the fix?
- Generated files in the change (`src/bindings.ts`, `.sqlx/`, `docs/generated/`): is every change explained by the goal, and nothing hand-edited? Is one missing that the change should have regenerated?
- Migrations: is there a test that runs them on a database with existing data, if the migration transforms data?

## Checklist: simplification and reuse

- Is there an existing helper for this? Check the neighbouring tables in `crates/sage-database/src/tables`, the utilities in `crates/sage/src/utils` and `crates/sage-wallet/src/utils`, the Wallet SDK (`chia_wallet_sdk`) for spend logic, and `src/hooks`, `src/components` and `src/lib` on the frontend.
- Is logic duplicated between this change and another endpoint, query or component, which should share it?
- Is anything computed repeatedly that could be computed once, a query run in a loop that could be one query, or something cloned where a borrow would do?
- Does a `match` use `_ =>` where an exhaustive match would make a new variant a compile error?
- Does the code follow the patterns of the neighbouring code, so it reads like the rest of the module?
- Does it follow `.claude/rules/*.md`, such as the Rust import and module ordering in `imports.md`?
- Dead code, unused parameters and dependencies, and comments that restate the code.
