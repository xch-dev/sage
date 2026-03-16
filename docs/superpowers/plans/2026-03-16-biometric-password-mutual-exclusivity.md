# Biometric-Password Mutual Exclusivity Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make biometric and password mutually exclusive — password takes precedence, removing all keychain password storage infrastructure.

**Architecture:** Biometric is a standalone gate for no-password wallets. Password wallets always show the password dialog. The two never interact. Remove `tauri-plugin-keychain`, the `notifyDecryptError` bridge, all skip/stale tracking refs, and all keychain lifecycle methods.

**Tech Stack:** TypeScript/React (Tauri v2), Rust (Cargo dependencies)

---

## Chunk 1: Remove keychain infrastructure and simplify PasswordContext

### Task 1: Simplify PasswordContext — remove all keychain code

**Files:**
- Modify: `src/contexts/PasswordContext.tsx`

- [ ] **Step 1: Remove keychain imports and helpers**

Remove the entire keychain section (lines 9–49): the lazy-loaded `keychainPromise`, `getKeychain()`, `keychainGet()`, `keychainSave()`, `keychainRemove()` functions.

Also remove the `keychainKey()` helper (line 82–84).

- [ ] **Step 2: Remove the notifyDecryptError bridge**

Remove the module-level callback infrastructure (lines 54–61): `onDecryptErrorCallback`, `notifyDecryptError()` export.

- [ ] **Step 3: Remove keychain lifecycle methods from context type**

Update `PasswordContextType` to remove `clearKeychainEntry`, `updateKeychainEntry`, and `clearAllKeychainEntries`. The interface should only have:

```typescript
export interface PasswordContextType {
  requestPassword: (hasPassword: boolean, fingerprint?: number) => Promise<string | null | undefined>;
}
```

- [ ] **Step 4: Remove stale-tracking refs and callback registration from provider**

Inside `PasswordProvider`, remove:
- `skipKeychainRef`
- `lastKeychainFingerprintRef`
- The `onDecryptErrorCallback = () => { ... }` registration block

- [ ] **Step 5: Simplify the requestPassword decision tree**

Replace the 4-case decision tree with 3 mutually exclusive cases:

```typescript
const requestPassword = useCallback(
  async (hasPassword: boolean, targetFingerprint?: number): Promise<string | null | undefined> => {
    // Case 1: Has password → password takes precedence, show dialog
    if (hasPassword) {
      return new Promise<string | null | undefined>((resolve) => {
        pendingRef.current = { resolve };
        setOpen(true);
      });
    }

    // Case 2: No password, biometric enabled → standalone biometric gate with 5-min cache
    if (biometricEnabled && isMobile) {
      const now = performance.now();
      if (
        lastBiometricPromptRef.current !== null &&
        now - lastBiometricPromptRef.current < BIOMETRIC_CACHE_MS
      ) {
        return null; // Within cache window, skip prompt
      }

      try {
        const { authenticate } = await import('@tauri-apps/plugin-biometric');
        await authenticate('Authenticate to continue', {
          allowDeviceCredential: false,
        });
        lastBiometricPromptRef.current = now;
        return null;
      } catch {
        return undefined; // biometric failed/cancelled
      }
    }

    // Case 3: No password, no biometric → no auth needed
    return null;
  },
  [biometricEnabled],
);
```

Note: `targetFingerprint` parameter is no longer used (was only needed for keychain lookup) but keep it in the signature for now since call sites pass it. It's harmless and avoids touching all call sites.

- [ ] **Step 6: Simplify handleSubmit and PasswordRequest — remove keychain save logic**

These changes must be done together with Step 5 to avoid transient type errors.

Replace handleSubmit — note the dependency array changes from `[biometricEnabled]` to `[]` since keychain save is removed:

```typescript
const handleSubmit = useCallback((password: string) => {
  setOpen(false);
  pendingRef.current?.resolve(password);
  pendingRef.current = null;
}, []);
```

Remove the `fingerprint` field from the `PasswordRequest` interface:

```typescript
interface PasswordRequest {
  resolve: (password: string | null | undefined) => void;
}
```

- [ ] **Step 7: Remove lifecycle methods from provider**

Remove the `clearKeychainEntry`, `updateKeychainEntry`, and `clearAllKeychainEntries` useCallback definitions.

Update the context value to only provide `requestPassword`:

```typescript
<PasswordContext.Provider value={{ requestPassword }}>
```

- [ ] **Step 8: Remove useWallet import and usage**

Remove `import { useWallet } from '@/contexts/WalletContext';` (line 3) and `const { wallet } = useWallet();` (line 90) from `PasswordProvider`. These were only used for `wallet?.fingerprint` as the keychain default fingerprint, which is no longer needed.

Keep `useBiometric` — it IS still needed for `biometricEnabled` in Case 2.

- [ ] **Step 9: Commit**

```bash
git add src/contexts/PasswordContext.tsx
git commit -m "refactor: make biometric and password mutually exclusive

Remove all keychain password storage — password wallets always use the
dialog, biometric is a standalone gate for no-password wallets only."
```

### Task 2: Remove notifyDecryptError from ErrorContext

**Files:**
- Modify: `src/contexts/ErrorContext.tsx`

- [ ] **Step 1: Remove the notifyDecryptError import and call**

Remove the import line:
```typescript
import { notifyDecryptError } from './PasswordContext';
```

Remove the `notifyDecryptError()` call from the `addError` handler. Keep the toast:
```typescript
if (reason.includes('decrypt')) {
  toast.error(t`Incorrect password`);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/contexts/ErrorContext.tsx
git commit -m "refactor: remove notifyDecryptError bridge from ErrorContext"
```

### Task 3: Remove keychain usage from Settings

**Files:**
- Modify: `src/pages/Settings.tsx`

- [ ] **Step 1: Remove keychain calls from GlobalSettings toggleBiometric**

In `GlobalSettings`, remove `clearAllKeychainEntries` from the `usePassword()` destructure. Simplify `toggleBiometric` to remove the keychain cleanup:

```typescript
const toggleBiometric = async (value: boolean) => {
  try {
    if (value) {
      await enableIfAvailable();
    } else {
      await disable();
    }
  } catch (error) {
    addError(error as CustomError);
  }
};
```

- [ ] **Step 2: Remove keychain calls from WalletSettings password handlers**

In `WalletSettings`, remove `clearKeychainEntry` and `updateKeychainEntry` from the `usePassword()` destructure.

Remove the keychain update block in `handlePasswordSubmit` (the section after `setPasswordDialogOpen(false)` that calls `updateKeychainEntry` / `clearKeychainEntry`).

- [ ] **Step 3: Commit**

```bash
git add src/pages/Settings.tsx
git commit -m "refactor: remove keychain lifecycle from Settings"
```

### Task 4: Remove keychain usage from WalletCard

**Files:**
- Modify: `src/components/WalletCard.tsx`

- [ ] **Step 1: Remove clearKeychainEntry from WalletCard**

Change the `usePassword()` destructure to only get `requestPassword`:
```typescript
const { requestPassword } = usePassword();
```

In `deleteSelf`, remove the `clearKeychainEntry` call after successful deletion:
```typescript
const deleteSelf = async () => {
  const password = await requestPassword(info.has_password, info.fingerprint);
  if (password === undefined) {
    setIsDeleteOpen(false);
    return;
  }
  await commands
    .deleteKey({ fingerprint: info.fingerprint })
    .then(async () => {
      setKeys(keys.filter((key) => key.fingerprint !== info.fingerprint));
    })
    .catch(addError);
  setIsDeleteOpen(false);
};
```

- [ ] **Step 2: Commit**

```bash
git add src/components/WalletCard.tsx
git commit -m "refactor: remove keychain cleanup from wallet deletion"
```

### Task 5: Remove usePassword hook's keychain exports

**Files:**
- Modify: `src/hooks/usePassword.ts`

- [ ] **Step 1: Verify usePassword.ts needs no changes**

`usePassword.ts` is a thin wrapper that returns the full context. Since `PasswordContextType` was already simplified in Task 1, this file should work as-is with no changes needed.

- [ ] **Step 2: Verify no other files reference removed methods**

Run: `grep -r 'clearKeychainEntry\|updateKeychainEntry\|clearAllKeychainEntries\|notifyDecryptError' src/`

Expected: No matches.

## Chunk 2: Remove tauri-plugin-keychain dependency

### Task 6: Remove tauri-plugin-keychain from Rust dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml` — remove `tauri-plugin-keychain = "2"` line
- Modify: `src-tauri/src/lib.rs` — remove `.plugin(tauri_plugin_keychain::init())` line
- Modify: `src-tauri/capabilities/mobile.json` — remove `"keychain:default"` from permissions array

- [ ] **Step 1: Remove from Cargo.toml**

Remove the line `tauri-plugin-keychain = "2"` from `[dependencies]`.

- [ ] **Step 2: Remove plugin init from lib.rs**

Remove `.plugin(tauri_plugin_keychain::init())` from the mobile builder chain.

- [ ] **Step 3: Remove capability from mobile.json**

Remove `"keychain:default"` from the permissions array (and the trailing comma from the previous entry).

- [ ] **Step 4: Remove from package.json**

Remove `"tauri-plugin-keychain": "^2.0.1"` from dependencies.

- [ ] **Step 5: Update lockfiles**

Run `pnpm install` to update the JS lockfile. Then run `cargo update` (from `src-tauri/`) to remove the keychain crate from `Cargo.lock`.

**Important:** All JS-side keychain usages (Tasks 1–5) must be completed before this step — otherwise `pnpm install` will remove the package while dynamic imports still reference it.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/capabilities/mobile.json package.json pnpm-lock.yaml Cargo.lock
git commit -m "chore: remove tauri-plugin-keychain dependency"
```

## Chunk 3: Update spec

### Task 7: Update the design spec

**Files:**
- Modify: `docs/superpowers/specs/2026-03-15-password-protection-design.md`

- [ ] **Step 1: Update the spec**

Key changes to the spec:
1. **Overview** — Replace "Optionally, users can enable biometric unlock... as a convenience layer that stores the password in the OS keychain" with language about mutual exclusivity
2. **Biometric-Password Bridge section** — Replace entirely. Remove keychain plugin references, store-on-first-use, retrieval flow steps 4-6. Replace with simple mutual exclusivity rule
3. **Decision tree** — Simplify to 3 cases (password→dialog, no password+biometric→biometric gate, no password+no biometric→no auth)
4. **Keychain lifecycle table** — Remove entirely
5. **Settings UI changes** — Remove keychain cleanup on toggle disable
6. **Design decisions** — Update "Graceful degradation" and remove keychain-related decisions
7. **Error Handling** — Remove "Stale keychain entry" and "Device restore" sections
8. **New dependency section** — Remove `tauri-plugin-keychain` references
9. **What's NOT Changing** — Update to note biometric remains as standalone gate only

- [ ] **Step 2: Commit**

```bash
git add docs/superpowers/specs/2026-03-15-password-protection-design.md
git commit -m "docs: update spec — biometric and password are mutually exclusive"
```

### Task 8: Verify build

- [ ] **Step 1: Run TypeScript type check**

Run: `pnpm tsc --noEmit`
Expected: No errors.

- [ ] **Step 2: Run linter**

Run: `pnpm lint`
Expected: No errors.

- [ ] **Step 3: Run Rust check**

Run: `cargo check` (from `src-tauri/`)
Expected: No errors.
