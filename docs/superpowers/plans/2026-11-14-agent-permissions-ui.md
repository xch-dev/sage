# Agent Permissions UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Settings → Agents grants capabilities and wallet scope with the same `sage-app-ui` editors
Sage apps use.

**Architecture:** The backend returns `SageAppCapabilityDefinitionView`s for agent capabilities.
Small pure helpers map an agent to the editors' inputs. The agent dialog renders `PermissionEditor`
and `WalletScopeEditor` under Permissions | Wallets tabs. The shared editors gain optional label
props, defaulting to today's English, so the system apps don't change.

**Tech Stack:** Rust (sage-apps, specta), React 18 + Lingui, `sage-app-ui` (tsup), vitest + jsdom +
Testing Library, Tailwind.

**Spec:** `docs/superpowers/specs/2026-11-14-agent-permissions-ui-design.md`

## Global Constraints

- Never hand-edit `src/bindings.ts`. Regenerate it with `pnpm generate:bindings`.
- Do not run `pnpm extract` or touch `src/locales/**/*.po`.
- Wrap every new user-facing string in the main app in Lingui (`t`, `<Trans>`).
- The `sage-app-ui` defaults stay byte-for-byte today's English strings.
- Rebuild the package (`pnpm --filter sage-app-ui build`) before type-checking or testing the main
  app.
- Rust doc comments are one line. Follow `.claude/rules/imports.md`.
- No git commits.

## Review Focus

1. **Unchecking a capability in the dialog makes it vanish.** The editor shows only granted
   optional entries until "Show N more" is clicked. Expected: in the agent dialog every capability
   stays visible. Pinned by Task 2's `collapseUngrantedOptional` prop and Task 3's test.
2. **An agent scoped only to a deleted wallet.** Nothing in the list is checked, yet the scope is
   non-empty. Expected: Save is disabled until a listed wallet is selected. Pinned by
   `hasSelectableWallet` tests.
3. **No wallet is logged in when creating.** The default selection is empty. Expected: Save is
   disabled, with a hint saying to pick a wallet or choose All wallets. Pinned by the
   `defaultAgentWalletScope` test and the hint in Task 3.
4. **The dialog opens before the capability definitions have loaded.** Expected: a new agent still
   defaults to all capabilities once they arrive. Pinned by the `initialAgentCapabilities` test.
5. **The package's Tailwind classes aren't generated in the main app.** Expected: the editors are
   styled. This is a manual check after Task 3, by building the frontend and viewing the dialog.

---

### Task 1: Return capability definitions for agents

**Files:**

- Modify: `crates/sage-apps/src/agents/commands.rs` (remove `AgentCapabilityInfo`; change
  `apps_list_agent_capabilities`; add a test)
- Modify: `crates/sage-apps/src/lib.rs` (drop the `AgentCapabilityInfo` export)
- Regenerate: `src/bindings.ts`

**Interfaces:**

- Produces:
  `commands.appsListAgentCapabilities(): Promise<SageAppCapabilityDefinitionView[]>`, where
  `SageAppCapabilityDefinitionView = { key: string; label: string; description: string; flags: SageAppCapabilityFlagsView }`.

- [ ] **Step 1: Write the failing test `listed_agent_capabilities_are_grantable_definitions`** in
      the `commands.rs` tests module.
  - Call `apps_list_agent_capabilities().await.unwrap()` and serialize it with
    `serde_json::to_value`.
  - Assert the `key`s equal `agent_grantable_capabilities()` mapped through `key()`, in the same
    order.
  - Assert every item has `flags.userGrantable == true`.
- [ ] **Step 2:** Run `cargo test -p sage-apps listed_agent_capabilities`. Expected: it fails to
      compile or fails, because the current response has no `key` or `flags`.
- [ ] **Step 3:** Make `apps_list_agent_capabilities() -> Result<Vec<SageAppCapabilityDefinitionView>>`
      map each capability through `get_user_capability_definition(..).into()`. Delete
      `AgentCapabilityInfo` and its `lib.rs` export.
- [ ] **Step 4:** Run `cargo test -p sage-apps`. Expected: everything passes.
- [ ] **Step 5:** Run `pnpm generate:bindings`. Expected: `src/bindings.ts` gains
      `SageAppCapabilityDefinitionView` and `SageAppCapabilityFlagsView`, and loses
      `AgentCapabilityInfo`.

### Task 2: Label and collapse props on the shared editors

**Files:**

- Modify: `packages/sage-app-ui/src/components/PermissionsEditor/PermissionsEditor.tsx`
- Modify: `packages/sage-app-ui/src/components/WalletScopeEditor.tsx`

**Interfaces:**

- Produces, on `PermissionEditorProps`:
  - `optionalSectionTitle?: string`, default `'Optional permissions'`.
  - `showMoreLabel?: (count: number) => string`. The default reproduces
    `Show ${count} more optional permission${count === 1 ? '' : 's'}`.
  - `collapseUngrantedOptional?: boolean`, default `true`. When it is `false`, every optional entry
    is always visible and no "show more" button renders.
  - These are passed through `PermissionEditor` → `PermissionSections`.
- Produces `export type WalletScopeEditorLabels` and the prop
  `labels?: Partial<WalletScopeEditorLabels>` on `WalletScopeEditor`. The keys and their defaults
  are today's strings:

  | Key               | Default                                    |
  | ----------------- | ------------------------------------------ |
  | `heading`         | `'Wallet availability'`                    |
  | `description`     | `'Choose which wallets can use this app.'` |
  | `allWallets`      | `'All wallets'`                            |
  | `selectedWallets` | `'Selected wallets only'`                  |
  | `noWallets`       | `'No wallets found.'`                      |
  | `fingerprint`     | `(f) => \`Fingerprint ${f}\``              |
  | `unnamedWallet`   | `(f) => \`Wallet ${f}\``                   |

- [ ] **Step 1:** Add the props with the defaults above, without changing any rendering when the
      props are omitted.
- [ ] **Step 2:** Run `pnpm --filter sage-app-ui build`. Expected: it succeeds.
- [ ] **Step 3:** Run `pnpm --filter sage-system-app-sdk build && pnpm run build:system-apps`, or
      the narrowest command that type-checks the system apps. Expected: it succeeds, which proves
      the props are optional.

### Task 3: Agent dialog uses the editors

**Files:**

- Create: `src/lib/agentPermissions.ts`
- Test: `src/lib/agentPermissions.test.ts`
- Test: `src/components/AgentPermissionsPanel.test.tsx`
- Create: `src/components/AgentPermissionsPanel.tsx`, holding the tabs and both editors, so it can
  be rendered in a test without the dialog.
- Modify: `src/components/AgentSettings.tsx` (state types, use the panel, remove the old checklist
  and switch)
- Modify: `tailwind.config.js` (`content` gains `'packages/sage-app-ui/src/**/*.{ts,tsx}'`)
- Modify: `docs/AGENTS.md` ("Add an agent" step 2 becomes: "Choose which wallets it can use: all
  wallets, or specific wallets (the current wallet is selected by default).")

**Interfaces:**

- Consumes:
  - Task 1's `SageAppCapabilityDefinitionView`.
  - Task 2's props.
  - `RequestedPermissionsView` and `PermissionEditorProps` from `sage-app-ui`. Derive the granted
    type as `PermissionEditorProps['grantedPermissions']`; the main app doesn't depend on
    `sage-system-app-sdk`.
- Produces, in `src/lib/agentPermissions.ts`:
  - `agentRequestedPermissions(definitions: SageAppCapabilityDefinitionView[]): RequestedPermissionsView`
    returns all keys as `optional`, none `required`, and no network.
  - `agentGrantedPermissions(capabilities: UserBridgeCapability[])` returns
    `{ capabilities, network: { whitelist: [], whitelistByNetwork: {} } }`.
  - `initialAgentCapabilities(agent: SageAgent | null, definitions: SageAppCapabilityDefinitionView[]): UserBridgeCapability[]`
    returns the agent's capabilities, or every definition key for a new agent.
  - `defaultAgentWalletScope(currentFingerprint?: number): SageAppWalletScope` returns
    `selectedWallets` with `[currentFingerprint]`, or `[]` when it's undefined.
  - `hasSelectableWallet(scope: SageAppWalletScope, wallets: { fingerprint: number }[]): boolean`
    is `true` for `allWallets`, and otherwise checks whether any selected fingerprint is in
    `wallets`.
- Produces
  `AgentPermissionsPanel({ definitions, capabilities, onCapabilitiesChange, walletScope, onWalletScopeChange, wallets, disabled })`:
  - It renders Lingui-labelled tabs, `t`Permissions`` and `t`Wallets``.
  - Permissions: `PermissionEditor` with `collapseUngrantedOptional={false}` and
    `optionalSectionTitle={t`Capabilities`}`.
  - Wallets: `WalletScopeEditor` with Lingui `labels`. The description is
    `t`Choose which wallets this agent can use.``.

- [ ] **Step 1: Write the failing `src/lib/agentPermissions.test.ts`**, covering:
  - `agentRequestedPermissions([defA, defB])` deep-equals
    `{ capabilities: { required: [], optional: [defA.key, defB.key] } }`.
  - `initialAgentCapabilities(null, [defA, defB])` equals `[defA.key, defB.key]`, and
    `initialAgentCapabilities(agentWith(['wallet.get_coins']), [defA, defB])` equals
    `['wallet.get_coins']`.
  - `defaultAgentWalletScope(7)` equals `{ kind: 'selectedWallets', fingerprints: [7] }`, and
    `defaultAgentWalletScope()` equals `{ kind: 'selectedWallets', fingerprints: [] }`.
  - `hasSelectableWallet` is:
    - `true` for `{ kind: 'allWallets' }` with `[]`;
    - `false` for `selectedWallets [9]` with wallets `[{ fingerprint: 7 }]`;
    - `true` for `selectedWallets [7]` with the same wallets;
    - `false` for `selectedWallets []`.
- [ ] **Step 2: Write the failing `src/components/AgentPermissionsPanel.test.tsx`** (jsdom; follow
      `MnemonicDisplay.test.tsx`). Render with two definitions, one granted:
  - Both capability labels are visible, including the ungranted one.
  - There is no "Show" button.
  - Unchecking the granted checkbox calls `onCapabilitiesChange` with `[]`.
- [ ] **Step 3:** Run `pnpm --filter sage-app-ui build && pnpm test:unit src/lib/agentPermissions src/components/AgentPermissionsPanel`.
      Expected: FAIL, because the modules don't exist yet.
- [ ] **Step 4:** Implement `agentPermissions.ts` and `AgentPermissionsPanel.tsx`.
- [ ] **Step 5:** Wire it into `AgentDialog`:
  - The state becomes `capabilities: UserBridgeCapability[]` and `walletScope: SageAppWalletScope`.
  - On open, reset from `initialAgentCapabilities` and from the agent's scope, or from
    `defaultAgentWalletScope(currentFingerprint)` for a new agent.
  - When the definitions finish loading after open, re-initialize a new agent's capabilities.
  - Load the wallets with `commands.getWallets({})`, mapped to `{ fingerprint, name, emoji }`.
  - `valid = name.trim() && hasSelectableWallet(walletScope, wallets)`.
  - When there's no selectable wallet, show
    `<Trans>Select at least one wallet, or choose All wallets.</Trans>`.
  - Submit sends `capabilities` and `walletScope`.
  - Remove the `allWallets` switch, `buildScope` and `keepsExistingScope`, plus any `Checkbox` and
    `Switch` imports that become unused.
  - Widen `DialogContent` (for example to `sm:max-w-xl`) if the editors don't fit.
- [ ] **Step 6:** Run `pnpm test:unit src/lib/agentPermissions src/components/AgentPermissionsPanel`.
      Expected: PASS.
- [ ] **Step 7: Verification:** run `pnpm prettier:check`, `pnpm lint`, `pnpm build:frontend`
      (`tsc -b` plus vite), `cargo fmt --all -- --files-with-diff --check` and
      `cargo clippy --workspace --all-features --all-targets`. Expected: all clean. Review Focus #5
      is a visual check of the dialog in a browser or the dev app.
