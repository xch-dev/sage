# Agent permissions UI: reuse the app permissions editors

## Intent

The Settings → Agents dialog has its own capability checklist and a current-or-all-wallets switch.
Replace both with the shared editors Sage apps use (`sage-app-ui`'s `PermissionEditor` and
`WalletScopeEditor`), so agents and apps grant access the same way and look the same.

## Goals

- The agent dialog shows a name field, then **Permissions | Wallets** tabs, as in the app "Change
  app access" review.
- Permissions: every agent-grantable capability, grouped and sorted by sensitivity, with help
  tooltips, exactly as the app editor renders it.
- Wallets: all wallets, or a checklist of specific wallets. Agents can now be limited to any set of
  wallets, which the backend already supports.
- Settings stays translatable.

## Non-goals

- No change to what agents can be granted, or to the backend's scope or capability checks.
- No restyling of the shared editors. The dialog matches the app permission screens, not the rest
  of Settings.

## Approaches

- **Permissions and wallets (chosen):** reuse both editors. This replaces the current-or-all switch
  and covers the parked rescope follow-up.
- Permissions only: keep the current-or-all switch. Rejected, because it leaves two scope UIs.

## Design

- **Backend:** `apps_list_agent_capabilities` returns `Vec<SageAppCapabilityDefinitionView>`, the
  type apps use, instead of `AgentCapabilityInfo`, which is removed. Bindings are regenerated with
  `pnpm generate:bindings`.
- **Mapping:**
  - Requested permissions: all agent-grantable capabilities, as optional.
  - Granted: the agent's capabilities, or all of them for a new agent.
  - Network permissions: empty.
  - The editor's `SageGrantedPermissionsInput` maps back to the capability list.
- **Wallet scope:**
  - A new agent defaults to selected wallets = the current wallet. With no wallet logged in, it
    defaults to an empty selection.
  - Editing an agent starts from its saved scope.
  - Save is disabled while the scope selects no wallets. This is my call: such an agent could do
    nothing.
- **Wallet list:** taken from the main app's existing wallet list command, mapped to
  `{ fingerprint, name, emoji }`.
- **`sage-app-ui` labels:**
  - Add optional label props to `PermissionEditor` (optional section title, "show more" text)
    and to `WalletScopeEditor` (heading, description, button labels, empty text, fingerprint
    line).
  - Add `collapseUngrantedOptional` (default `true`). The editor normally shows only granted
    optional entries until "Show N more" is clicked, so an entry you uncheck disappears. The agent
    dialog passes `false`, so every capability stays visible.
  - The defaults are today's English strings, so the system apps don't change.
  - Settings passes Lingui `t` strings.
- **Tailwind:** add `packages/sage-app-ui/src/**/*.{ts,tsx}` to the main `tailwind.config.js`
  `content`, as the system apps do.
- **Docs:** update the "Add an agent" steps in `docs/AGENTS.md`.

## Left to the builder

- The label prop shape (one `labels` object or individual props), following the package's existing
  `emptyText` / `requiredSectionTitle` style.
- The tabs component: the main app's `ui/tabs`.
- The dialog width needed to fit the editors.

## Constraints

- Never hand-edit `src/bindings.ts`, and don't run `pnpm extract`. New strings are picked up at the
  next release.
- Rebuild the package with `pnpm --filter sage-app-ui build` before type-checking the main app.
- No commits. The user commits.
