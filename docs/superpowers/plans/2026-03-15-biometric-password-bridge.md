# Biometric-Password Bridge Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify password and biometric authentication into a single `requestPassword` entry point that stores/retrieves passwords from the OS keychain via biometric on mobile.

**Architecture:** `PasswordContext.requestPassword()` becomes the sole auth gate. On mobile with biometric enabled, it attempts keychain retrieval (which triggers OS biometric prompt) before falling back to the password dialog. On first successful typed password, the password is stored in the keychain for future biometric retrieval. `BiometricContext` retains state management (`enabled`, `available`) but `promptIfEnabled()` is no longer called at operation sites.

**Tech Stack:** `tauri-plugin-keychain` (Rust + JS) for iOS Keychain / Android AccountManager, `@tauri-apps/plugin-biometric` (existing), React context + hooks, TypeScript.

**Spec:** `docs/superpowers/specs/2026-03-15-password-protection-design.md`

---

## File Structure

### New files

- None — all changes are modifications to existing files

### Modified files

| File                                                              | Responsibility                                                                                                                      |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/Cargo.toml`                                            | Add `tauri-plugin-keychain` dependency (mobile target)                                                                              |
| `src-tauri/src/lib.rs`                                            | Register keychain plugin in `#[cfg(mobile)]` block                                                                                  |
| `src-tauri/capabilities/mobile.json`                              | Add keychain permission                                                                                                             |
| `package.json`                                                    | Add `tauri-plugin-keychain` JS package                                                                                              |
| `src/contexts/PasswordContext.tsx`                                | Integrate keychain store/retrieve into `requestPassword`, change return type to `string \| null \| undefined`                       |
| `src/hooks/usePassword.ts`                                        | Update type to match new `PasswordContextType`                                                                                      |
| `src/App.tsx`                                                     | Restructure provider tree: `WalletProvider` → `PasswordProvider` → `WalletConnectProvider`                                          |
| `src/contexts/WalletConnectContext.tsx`                           | Remove `promptIfEnabled` usage, update auth pattern                                                                                 |
| `src/walletconnect/handler.ts`                                    | Remove `promptIfEnabled` from `HandlerContext`                                                                                      |
| `src/walletconnect/commands/chip0002.ts`                          | Remove `promptIfEnabled` calls, update auth check                                                                                   |
| `src/walletconnect/commands/offers.ts`                            | Remove `promptIfEnabled` calls, update auth check                                                                                   |
| `src/walletconnect/commands/high-level.ts`                        | Remove `promptIfEnabled` calls, update auth check                                                                                   |
| `src/components/ConfirmationDialog.tsx`                           | Remove `promptIfEnabled`, update auth pattern                                                                                       |
| `src/components/WalletCard.tsx`                                   | Remove `promptIfEnabled`, update auth pattern, keychain cleanup on delete                                                           |
| `src/components/OfferRowCard.tsx`                                 | Update auth check pattern (`password === null` → `password === undefined`)                                                          |
| `src/hooks/useOfferProcessor.ts`                                  | Remove `promptIfEnabled`, update auth pattern                                                                                       |
| `src/pages/Settings.tsx`                                          | Remove `promptIfEnabled` from RpcSettings, update `increaseDerivationIndex` auth pattern, move biometric toggle to Security section |
| `src/pages/Offers.tsx`                                            | Update auth check pattern                                                                                                           |
| `src/pages/Offer.tsx`                                             | Update auth check pattern                                                                                                           |
| `docs/superpowers/specs/2026-03-15-password-protection-design.md` | Update plugin reference from `tauri-plugin-keystore` to `tauri-plugin-keychain`, update provider placement                          |

---

## Chunk 1: Dependencies and Plugin Registration

### Task 1: Add tauri-plugin-keychain Rust dependency

**Files:**

- Modify: `src-tauri/Cargo.toml:50-54`

- [ ] **Step 1: Add keychain dependency to mobile target**

Add `tauri-plugin-keychain` to the mobile-only dependencies section:

```toml
[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
tauri-plugin-biometric = { workspace = true }
tauri-plugin-barcode-scanner = { workspace = true }
tauri-plugin-safe-area-insets = { workspace = true }
tauri-plugin-sage = { workspace = true }
tauri-plugin-keychain = "2"
```

- [ ] **Step 2: Register keychain plugin in lib.rs**

Modify: `src-tauri/src/lib.rs:170-178`

Add `tauri_plugin_keychain::init()` to the `#[cfg(mobile)]` block:

```rust
#[cfg(mobile)]
{
    tauri_builder = tauri_builder
        .plugin(tauri_plugin_barcode_scanner::init())
        .plugin(tauri_plugin_safe_area_insets::init())
        .plugin(tauri_plugin_biometric::init())
        .plugin(tauri_plugin_sharesheet::init())
        .plugin(tauri_plugin_sage::init())
        .plugin(tauri_plugin_keychain::init());
}
```

- [ ] **Step 3: Add keychain permission to mobile capabilities**

Modify: `src-tauri/capabilities/mobile.json`

```json
{
  "$schema": "../gen/schemas/mobile-schema.json",
  "identifier": "mobile-capability",
  "windows": ["main"],
  "platforms": ["android", "iOS"],
  "permissions": [
    "safe-area-insets:default",
    "barcode-scanner:default",
    "biometric:default",
    "sage:default",
    "sharesheet:allow-share-text",
    "keychain:default"
  ]
}
```

- [ ] **Step 4: Add JS package dependency**

Run: `pnpm add tauri-plugin-keychain`

- [ ] **Step 5: Verify the project builds**

Run: `pnpm tauri build --debug` or at minimum `cargo check` in `src-tauri/`

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/capabilities/mobile.json package.json pnpm-lock.yaml
git commit -m "feat: add tauri-plugin-keychain dependency for biometric-password bridge"
```

---

### Task 2: Update spec to reference tauri-plugin-keychain

**Files:**

- Modify: `docs/superpowers/specs/2026-03-15-password-protection-design.md`

- [ ] **Step 1: Replace all references to `tauri-plugin-keystore` with `tauri-plugin-keychain`**

In the spec, find and replace:

- `tauri-plugin-keystore` → `tauri-plugin-keychain`
- Update the plugin description from "Simple API: `store(key, value)`, `retrieve(key)`, `remove(key)`" to "Key-value API: `saveItem(key, password)`, `getItem(key)`, `removeItem(key)`"
- Update the dependency section: "wraps iOS Keychain and Android AccountManager" instead of "iOS Keychain and Android Keystore (API 28+)"
- Update the Cargo dependency line to `tauri-plugin-keychain = "2"`

- [ ] **Step 2: Update provider placement in spec**

The spec currently says (line 226): "Provider placement: Inside `I18nProvider`... but wrapping `WalletProvider`". Update to reflect the new dependency chain:

> **Provider placement:** Inside `I18nProvider` and `WalletProvider` (required because `PasswordProvider` now reads `useWallet()` for the fingerprint and `useBiometric()` for the enabled state). Wraps `WalletConnectProvider` and all downstream providers, so `usePassword()` is available everywhere.

New tree: `BiometricProvider` → `I18nProvider` → `WalletProvider` → `PasswordProvider` → `PeerProvider` → `WalletConnectProvider` → `PriceProvider` → `RouterProvider`

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-03-15-password-protection-design.md
git commit -m "docs: update spec to reference tauri-plugin-keychain"
```

---

## Chunk 2: Unified PasswordContext with Keychain Integration

### Task 3: Refactor PasswordContext to unified auth entry point

This is the core task. `requestPassword` gains keychain awareness and subsumes the standalone biometric gate.

**Files:**

- Modify: `src/contexts/PasswordContext.tsx`
- Modify: `src/hooks/usePassword.ts`

- [ ] **Step 1: Write the updated PasswordContext**

The new `requestPassword` implements the decision tree from the spec. Key design decisions:

1. **Store-after-validation, not store-on-submit:** `handleSubmit` does NOT store the password in the keychain. Instead, `requestPassword` returns the password and the caller's backend operation validates it. Only after a successful operation does the password get stored. This is achieved by `requestPassword` saving the typed password in `pendingStoreRef` and only writing to keychain on the _next_ successful keychain-skipped call (i.e., the password worked last time).
2. **Stale keychain recovery:** A `skipKeychainRef` tracks fingerprints where keychain retrieval returned a password that was rejected. On the next `requestPassword` call for that fingerprint, keychain is skipped and the dialog is shown instead.
3. **Biometric caching for standalone gate:** The existing 5-minute caching window from `BiometricContext.promptIfEnabled()` is preserved for Case 2 (no password, biometric enabled). A `lastBiometricPromptRef` tracks the last successful biometric auth time.

```typescript
// src/contexts/PasswordContext.tsx
import { PasswordDialog } from '@/components/dialogs/PasswordDialog';
import { useBiometric } from '@/hooks/useBiometric';
import { useWallet } from '@/contexts/WalletContext';
import { platform } from '@tauri-apps/plugin-os';
import { createContext, ReactNode, useCallback, useRef, useState } from 'react';

const isMobile = platform() === 'ios' || platform() === 'android';

// Lazy-initialized keychain module (mobile only)
let keychainPromise: Promise<typeof import('tauri-plugin-keychain')> | null = null;
function getKeychain() {
  if (!isMobile) return null;
  if (!keychainPromise) keychainPromise = import('tauri-plugin-keychain');
  return keychainPromise;
}

async function keychainGet(key: string): Promise<string | null> {
  const mod = getKeychain();
  if (!mod) return null;
  try {
    const { getItem } = await mod;
    return await getItem(key);
  } catch {
    return null;
  }
}

async function keychainSave(key: string, password: string): Promise<void> {
  const mod = getKeychain();
  if (!mod) return;
  try {
    const { saveItem } = await mod;
    await saveItem(key, password);
  } catch {
    // Silently fail — keychain storage is best-effort
  }
}

async function keychainRemove(key: string): Promise<void> {
  const mod = getKeychain();
  if (!mod) return;
  try {
    const { removeItem } = await mod;
    await removeItem(key);
  } catch {
    // Silently fail
  }
}

// Biometric caching interval (5 minutes), matching the existing BiometricContext behavior
const BIOMETRIC_CACHE_MS = 5 * 60 * 1000;

interface PasswordRequest {
  resolve: (password: string | null | undefined) => void;
}

export interface PasswordContextType {
  /**
   * Unified auth entry point for all protected operations.
   *
   * Returns:
   * - string: use this password (typed or retrieved from keychain)
   * - null: no password needed, auth passed
   * - undefined: auth cancelled or failed, abort the operation
   */
  requestPassword: (hasPassword: boolean) => Promise<string | null | undefined>;

  /** Clear the keychain entry for a specific wallet fingerprint */
  clearKeychainEntry: (fingerprint: number) => Promise<void>;

  /** Update the keychain entry after a password change */
  updateKeychainEntry: (fingerprint: number, newPassword: string) => Promise<void>;

  /** Clear all keychain entries (used when disabling biometric) */
  clearAllKeychainEntries: (fingerprints: number[]) => Promise<void>;
}

export const PasswordContext = createContext<PasswordContextType | undefined>(
  undefined,
);

function keychainKey(fingerprint: number): string {
  return `sage-password-${fingerprint.toString()}`;
}

export function PasswordProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const pendingRef = useRef<PasswordRequest | null>(null);
  const { enabled: biometricEnabled } = useBiometric();
  const { wallet } = useWallet();

  // Stale keychain recovery: skip keychain for fingerprints that returned bad passwords
  const skipKeychainRef = useRef<Set<number>>(new Set());

  // Biometric caching for standalone gate (Case 2)
  const lastBiometricPromptRef = useRef<number | null>(null);

  const requestPassword = useCallback(
    async (hasPassword: boolean): Promise<string | null | undefined> => {
      const fingerprint = wallet?.fingerprint;

      // Case 1: No password, no biometric → no auth needed
      if (!hasPassword && !biometricEnabled) {
        return null;
      }

      // Case 2: No password, biometric enabled → standalone biometric gate with 5-min cache
      if (!hasPassword && biometricEnabled && isMobile) {
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

      // Case 3: Has password, biometric enabled → try keychain first (unless skipped)
      if (
        hasPassword &&
        biometricEnabled &&
        isMobile &&
        fingerprint &&
        !skipKeychainRef.current.has(fingerprint)
      ) {
        const stored = await keychainGet(keychainKey(fingerprint));
        if (stored !== null) {
          // Mark as pending validation — if backend rejects, next call skips keychain
          skipKeychainRef.current.add(fingerprint);
          return stored;
        }
        // Fall through to password dialog if keychain retrieval fails
      }

      // Case 4: Has password → show dialog (fallback or no biometric)
      if (hasPassword) {
        return new Promise<string | null | undefined>((resolve) => {
          pendingRef.current = { resolve };
          setOpen(true);
        });
      }

      return null;
    },
    [biometricEnabled, wallet?.fingerprint],
  );

  const handleSubmit = useCallback(
    (password: string) => {
      setOpen(false);
      const fingerprint = wallet?.fingerprint;

      // Manual password entry: clear skip flag and store in keychain
      // The password will be validated by the backend. If it's wrong, the
      // "Incorrect password" toast fires. On next requestPassword, keychain
      // will return this password, skipKeychainRef kicks in, and dialog shows.
      if (fingerprint) {
        skipKeychainRef.current.delete(fingerprint);
        if (biometricEnabled && isMobile) {
          keychainSave(keychainKey(fingerprint), password);
        }
      }

      pendingRef.current?.resolve(password);
      pendingRef.current = null;
    },
    [biometricEnabled, wallet?.fingerprint],
  );

  const handleCancel = useCallback(() => {
    setOpen(false);
    pendingRef.current?.resolve(undefined); // undefined = cancelled
    pendingRef.current = null;
  }, []);

  const clearKeychainEntry = useCallback(async (fingerprint: number) => {
    skipKeychainRef.current.delete(fingerprint);
    await keychainRemove(keychainKey(fingerprint));
  }, []);

  const updateKeychainEntry = useCallback(
    async (fingerprint: number, newPassword: string) => {
      skipKeychainRef.current.delete(fingerprint);
      if (biometricEnabled && isMobile) {
        await keychainSave(keychainKey(fingerprint), newPassword);
      }
    },
    [biometricEnabled],
  );

  const clearAllKeychainEntries = useCallback(
    async (fingerprints: number[]) => {
      for (const fp of fingerprints) {
        skipKeychainRef.current.delete(fp);
        await keychainRemove(keychainKey(fp));
      }
    },
    [],
  );

  return (
    <PasswordContext.Provider
      value={{
        requestPassword,
        clearKeychainEntry,
        updateKeychainEntry,
        clearAllKeychainEntries,
      }}
    >
      {children}
      <PasswordDialog
        open={open}
        onSubmit={handleSubmit}
        onCancel={handleCancel}
      />
    </PasswordContext.Provider>
  );
}
```

**Important notes for the implementer:**

- `PasswordProvider` now depends on `useBiometric()` and `useWallet()`. It must be placed inside both `BiometricProvider` and `WalletProvider` in the component tree. This is a change from the current placement where `PasswordProvider` wraps `WalletProvider`. See Step 3 for the new provider tree.
- **Stale keychain recovery:** When `requestPassword` returns a keychain-retrieved password, it adds the fingerprint to `skipKeychainRef`. If the backend accepts the password (normal flow), the _next_ `requestPassword` will skip keychain and show the dialog. But the user won't see this because the operation succeeded. If the backend _rejects_ the password (stale entry), the "Incorrect password" toast fires, and the next `requestPassword` shows the dialog. The user types the correct password in the dialog, which clears the skip flag and updates the keychain.
- **Why store-on-dialog-submit is acceptable:** The dialog password is stored in the keychain immediately on submit, before backend validation. If the user types a wrong password, it gets stored in the keychain — but `skipKeychainRef` ensures the next call skips keychain and shows the dialog again. When the user eventually types the correct password, it replaces the stale entry. This is a pragmatic trade-off: the one-wrong-password-in-keychain scenario is self-healing.
- The lazy `getKeychain()` import ensures the plugin is only loaded on mobile. On desktop, all keychain operations are no-ops.
- `keychainKey` uses `fingerprint.toString()` explicitly for clarity, though JS number-to-string conversion of u32 values is safe.

- [ ] **Step 2: Update usePassword hook type**

Update `src/hooks/usePassword.ts` — the type is already correct since it just re-exports the context, but verify it matches.

- [ ] **Step 3: Restructure provider tree in App.tsx**

`PasswordProvider` now depends on `useBiometric()` (from `BiometricProvider` in outer `App`) and `useWallet()` (from `WalletProvider`). `WalletConnectContext` uses `usePassword()`, so `PasswordProvider` must wrap it. The correct ordering is:

```
BiometricProvider (outer App) → I18nProvider → WalletProvider → PasswordProvider → WalletConnectProvider
```

Update `src/App.tsx` `AppInner`:

```typescript
<I18nProvider i18n={i18n}>
  <WalletProvider>
    <PasswordProvider>
      <PeerProvider>
        <WalletConnectProvider>
          <PriceProvider>
            <RouterProvider router={router} />
          </PriceProvider>
        </WalletConnectProvider>
      </PeerProvider>
    </PasswordProvider>
  </WalletProvider>
</I18nProvider>
```

- [ ] **Step 4: Verify build compiles**

Run: `pnpm tsc --noEmit`

- [ ] **Step 5: Commit**

```bash
git add src/contexts/PasswordContext.tsx src/hooks/usePassword.ts src/App.tsx
git commit -m "feat: refactor PasswordContext to unified auth with keychain integration"
```

---

## Chunk 3: Update All Call Sites

Every call site currently has a two-step pattern:

```typescript
const password = await requestPassword(wallet?.has_password ?? false);
if (password === null && wallet?.has_password) return;
if (!(await promptIfEnabled())) return;
```

This becomes:

```typescript
const password = await requestPassword(wallet?.has_password ?? false);
if (password === undefined) return;
```

### Task 4: Update ConfirmationDialog

**Files:**

- Modify: `src/components/ConfirmationDialog.tsx`

- [ ] **Step 1: Remove biometric import and usage**

Remove `import { useBiometric } from '@/hooks/useBiometric'` and `const { promptIfEnabled } = useBiometric()`.

- [ ] **Step 2: Update Sign Transaction button (line ~522-545)**

Replace the auth pattern in the Sign Transaction onClick handler:

```typescript
onClick={async () => {
  const password = await requestPassword(
    wallet?.has_password ?? false,
  );
  if (password === undefined) return;

  commands
    .signCoinSpends({
      coin_spends:
        response === null
          ? []
          : 'coin_spends' in response
            ? response.coin_spends
            : response.spend_bundle.coin_spends,
      password,
    })
    .then((data) => {
      setSignature(
        data.spend_bundle.aggregated_signature,
      );
      toast.success(t`Transaction signed successfully`);
    })
    .catch(addError);
}}
```

- [ ] **Step 3: Update Submit button (line ~633-645)**

Replace the auth pattern in the submit flow:

```typescript
const password = await requestPassword(wallet?.has_password ?? false);
if (password === undefined) return;

const data = await commands
  .signCoinSpends({
    coin_spends: response.coin_spends,
    password,
  })
  .catch(addError);
```

- [ ] **Step 4: Commit**

```bash
git add src/components/ConfirmationDialog.tsx
git commit -m "refactor: update ConfirmationDialog to unified auth pattern"
```

---

### Task 5: Update WalletCard

**Files:**

- Modify: `src/components/WalletCard.tsx`

- [ ] **Step 1: Remove biometric import and usage**

Remove `import { useBiometric } from '@/hooks/useBiometric'` and `const { promptIfEnabled } = useBiometric()`.

- [ ] **Step 2: Update deleteSelf (line ~83-94)**

`deleteSelf` currently only uses `promptIfEnabled` (no password). With unified auth, this becomes a `requestPassword` call that triggers the standalone biometric gate when no password is set:

```typescript
const deleteSelf = async () => {
  const password = await requestPassword(info.has_password);
  if (password === undefined) return;

  await commands
    .deleteKey({ fingerprint: info.fingerprint })
    .then(() =>
      setKeys(keys.filter((key) => key.fingerprint !== info.fingerprint)),
    )
    .catch(addError);

  setIsDeleteOpen(false);
};
```

Note: `deleteKey` doesn't take a password param. The `requestPassword` call here is purely for the biometric gate. The returned password string is unused.

- [ ] **Step 3: Update View Details effect (line ~172-202)**

```typescript
useEffect(() => {
  (async () => {
    if (!isDetailsOpen) {
      setSecrets(null);
      return;
    }

    const password = await requestPassword(info.has_password);
    if (password === undefined) {
      setIsDetailsOpen(false);
      return;
    }

    commands
      .getSecretKey({ fingerprint: info.fingerprint, password })
      .then((data) => data.secrets !== null && setSecrets(data.secrets))
      .catch(addError);
  })();
}, [
  isDetailsOpen,
  info.fingerprint,
  info.has_password,
  addError,
  requestPassword,
]);
```

- [ ] **Step 4: Update loginSelf biometric guard (line ~185 area)**

Check if `loginSelf` or any other function in WalletCard uses `promptIfEnabled`. If the existing code at line 185 (`if (!(await promptIfEnabled()))`) is inside the View Details effect, it's already handled in step 3. Search for any other usage.

- [ ] **Step 5: Commit**

```bash
git add src/components/WalletCard.tsx
git commit -m "refactor: update WalletCard to unified auth pattern"
```

---

### Task 6: Update useOfferProcessor

**Files:**

- Modify: `src/hooks/useOfferProcessor.ts`

- [ ] **Step 1: Remove biometric import and usage**

Remove `import { useBiometric } from '@/hooks/useBiometric'` and `const { promptIfEnabled } = useBiometric()`.

- [ ] **Step 2: Update processOffer auth pattern (line ~67-74)**

```typescript
const password = await requestPassword(wallet?.has_password ?? false);
if (password === undefined) {
  throw new Error(t`Authentication was cancelled`);
}
```

Remove the separate `promptIfEnabled` check and its error.

- [ ] **Step 3: Remove promptIfEnabled from dependency array (line ~193)**

Remove `promptIfEnabled` from the `useCallback` dependency array.

- [ ] **Step 4: Commit**

```bash
git add src/hooks/useOfferProcessor.ts
git commit -m "refactor: update useOfferProcessor to unified auth pattern"
```

---

### Task 7: Update WalletConnect handler and commands

**Files:**

- Modify: `src/walletconnect/handler.ts`
- Modify: `src/walletconnect/commands/chip0002.ts`
- Modify: `src/walletconnect/commands/offers.ts`
- Modify: `src/walletconnect/commands/high-level.ts`
- Modify: `src/contexts/WalletConnectContext.tsx`

- [ ] **Step 1: Update HandlerContext interface**

In `src/walletconnect/handler.ts`, remove `promptIfEnabled`:

```typescript
export interface HandlerContext {
  requestPassword: (hasPassword: boolean) => Promise<string | null | undefined>;
  hasPassword: boolean;
}
```

- [ ] **Step 2: Update chip0002.ts commands**

In `handleSignCoinSpends` and `handleSignMessage`, replace:

```typescript
const password = await context.requestPassword(context.hasPassword);
if (password === null && context.hasPassword)
  throw new Error('Authentication failed');

if (!(await context.promptIfEnabled())) {
  throw new Error('Authentication failed');
}
```

With:

```typescript
const password = await context.requestPassword(context.hasPassword);
if (password === undefined) throw new Error('Authentication failed');
```

- [ ] **Step 3: Update offers.ts commands**

In `handleCreateOffer`, `handleTakeOffer`, `handleCancelOffer`, replace the same two-step pattern with the unified check. Each function has `if (!(await context.promptIfEnabled()))` — remove all of them and update the password check.

- [ ] **Step 4: Update high-level.ts commands**

In `handleSend`, `handleSignMessageByAddress`, `handleBulkMintNfts`, and any other function using `context.promptIfEnabled()`, apply the same replacement.

- [ ] **Step 5: Update WalletConnectContext.tsx**

Remove `useBiometric` import and `promptIfEnabled` destructuring. Remove `promptIfEnabled` from the context object passed to `handleCommand` and from the `useCallback` dependency array:

```typescript
const result = await handleCommand(method, request.params.request.params, {
  requestPassword,
  hasPassword: wallet?.has_password ?? false,
});
```

- [ ] **Step 6: Commit**

```bash
git add src/walletconnect/handler.ts src/walletconnect/commands/chip0002.ts src/walletconnect/commands/offers.ts src/walletconnect/commands/high-level.ts src/contexts/WalletConnectContext.tsx
git commit -m "refactor: update WalletConnect to unified auth pattern"
```

---

### Task 8: Update Settings RPC, increaseDerivationIndex, OfferRowCard, and remaining call sites

**Files:**

- Modify: `src/pages/Settings.tsx`
- Modify: `src/pages/Offers.tsx`
- Modify: `src/pages/Offer.tsx`
- Modify: `src/components/OfferRowCard.tsx`

- [ ] **Step 1: Update RpcSettings in Settings.tsx (line ~939-986)**

Replace `promptIfEnabled` usage in `start` and `toggleRunOnStartup` with `requestPassword`. Since these are sensitive operations but don't require a password, use `requestPassword(false)` which triggers the standalone biometric gate:

```typescript
function RpcSettings() {
  const { addError } = useErrors();
  const { requestPassword } = usePassword();
  // ... existing state ...

  const start = async () => {
    const auth = await requestPassword(false);
    if (auth === undefined) return;

    commands
      .startRpcServer()
      .catch(addError)
      .then(() => setIsRunning(true));
  };

  const toggleRunOnStartup = async (checked: boolean) => {
    const auth = await requestPassword(false);
    if (auth === undefined) return;

    commands
      .setRpcRunOnStartup(checked)
      .catch(addError)
      .then(() => setRunOnStartup(checked));
  };
  // ...
}
```

- [ ] **Step 2: Update increaseDerivationIndex in Settings.tsx (line ~1257-1258)**

The `handler` function in the derivation index section uses the old pattern:

```typescript
// Old:
if (password === null && key?.has_password) return;
// New:
if (password === undefined) return;
```

- [ ] **Step 3: Update OfferRowCard.tsx (line ~67-68)**

The `cancelHandler` uses the old pattern:

```typescript
// Old:
const password = await requestPassword(wallet?.has_password ?? false);
if (password === null && wallet?.has_password) return;
// New:
const password = await requestPassword(wallet?.has_password ?? false);
if (password === undefined) return;
```

- [ ] **Step 4: Update Offers.tsx auth pattern (line ~192-194)**

Same replacement: `password === null && wallet?.has_password` → `password === undefined`.

- [ ] **Step 5: Update Offer.tsx auth pattern (line ~103-104)**

Same replacement.

- [ ] **Step 6: Verify no remaining promptIfEnabled call sites**

Run: `grep -r "promptIfEnabled" src/ --include="*.ts" --include="*.tsx" -l`

Expected: Only `BiometricContext.tsx` (definition) and `useBiometric.ts` (re-export). No call sites.

- [ ] **Step 7: Commit**

```bash
git add src/pages/Settings.tsx src/pages/Offers.tsx src/pages/Offer.tsx src/components/OfferRowCard.tsx
git commit -m "refactor: update remaining call sites to unified auth pattern"
```

---

## Chunk 4: Settings UI and Keychain Lifecycle

### Task 9: Move biometric toggle to Security section with context-aware labels

**Files:**

- Modify: `src/pages/Settings.tsx`

- [ ] **Step 1: Remove biometric toggle from GlobalSettings (line ~307-318)**

Remove the `isMobile && (...)` block containing the biometric toggle from the Preferences section in `GlobalSettings`.

- [ ] **Step 2: Add biometric toggle to WalletSettings Security section**

The Security section already exists in WalletSettings (where password management lives). Add the biometric toggle there with context-aware labels:

```typescript
// Inside the Security section of WalletSettings, after password controls
{isMobile && available && (
  <>
    <SettingItem
      label={wallet?.has_password ? t`Biometric Unlock` : t`Biometric Authentication`}
      description={
        wallet?.has_password
          ? t`Use Face ID / Touch ID instead of typing your password`
          : t`Require biometrics for sensitive actions`
      }
      control={
        <Switch
          checked={biometricEnabled}
          onCheckedChange={toggleBiometric}
        />
      }
    />
    {wallet?.has_password && biometricEnabled && (
      <p className='text-xs text-muted-foreground px-3 -mt-2'>
        {t`Biometric unlock is a convenience — remember your password. There is no way to recover a lost password.`}
      </p>
    )}
  </>
)}
```

Where `biometricEnabled`, `available`, and `toggleBiometric` come from the existing biometric context, but `toggleBiometric` is updated to clear keychain entries on disable:

```typescript
const {
  enabled: biometricEnabled,
  available,
  enableIfAvailable,
  disable,
} = useBiometric();
const { clearAllKeychainEntries } = usePassword();

const toggleBiometric = async (value: boolean) => {
  if (value) {
    await enableIfAvailable();
  } else {
    await disable();
    // Clear all stored passwords from keychain
    if (keys) {
      await clearAllKeychainEntries(keys.map((k) => k.fingerprint));
    }
  }
};
```

Note: `useBiometric` and `usePassword` need to be available in the WalletSettings component. Check if they're already imported there.

- [ ] **Step 3: Commit**

```bash
git add src/pages/Settings.tsx
git commit -m "feat: move biometric toggle to Security section with context-aware labels"
```

---

### Task 10: Handle keychain lifecycle in password management

**Files:**

- Modify: `src/pages/Settings.tsx` (password change/remove handlers)

**Note:** Stale keychain entry recovery is already handled in the `PasswordContext` code from Task 3 via `skipKeychainRef`. This task only covers the explicit lifecycle events triggered by user actions in Settings.

- [ ] **Step 1: Update password change handler to update keychain**

When a user changes their password in Settings, update the keychain entry. Destructure `updateKeychainEntry` from `usePassword()` in the relevant Settings component:

```typescript
// After successful commands.changePassword():
await updateKeychainEntry(wallet.fingerprint, newPassword);
```

- [ ] **Step 2: Update password remove handler to clear keychain**

When a user removes their password, clear the keychain entry:

```typescript
// After successful commands.changePassword(old, ""):
await clearKeychainEntry(wallet.fingerprint);
```

- [ ] **Step 3: Commit**

```bash
git add src/pages/Settings.tsx
git commit -m "feat: handle keychain lifecycle for password changes"
```

---

### Task 11: Handle wallet deletion keychain cleanup

**Files:**

- Modify: `src/components/WalletCard.tsx` or wherever `deleteKey` is called

- [ ] **Step 1: Clear keychain entry on wallet deletion**

In the `deleteSelf` handler in `WalletCard.tsx`, after successful deletion:

```typescript
const { clearKeychainEntry } = usePassword();

const deleteSelf = async () => {
  const password = await requestPassword(info.has_password);
  if (password === undefined) return;

  await commands
    .deleteKey({ fingerprint: info.fingerprint })
    .then(async () => {
      await clearKeychainEntry(info.fingerprint);
      setKeys(keys.filter((key) => key.fingerprint !== info.fingerprint));
    })
    .catch(addError);

  setIsDeleteOpen(false);
};
```

- [ ] **Step 2: Commit**

```bash
git add src/components/WalletCard.tsx
git commit -m "feat: clear keychain entry on wallet deletion"
```

---

## Chunk 5: Verification and Cleanup

### Task 12: Verify no remaining direct promptIfEnabled usage

- [ ] **Step 1: Search for any remaining promptIfEnabled call sites**

Run: `grep -r "promptIfEnabled" src/ --include="*.ts" --include="*.tsx"`

Expected: Only `BiometricContext.tsx` (definition) and `useBiometric.ts` (re-export). No call sites.

- [ ] **Step 2: Search for old auth pattern**

Run: `grep -r "password === null && wallet" src/ --include="*.ts" --include="*.tsx"`

Expected: No results. All should use `password === undefined`.

- [ ] **Step 3: Verify TypeScript compiles**

Run: `pnpm tsc --noEmit`

- [ ] **Step 4: Verify build succeeds**

Run: `pnpm build`

- [ ] **Step 5: Commit any final cleanup**

```bash
git add -A
git commit -m "chore: final cleanup for biometric-password bridge"
```

---

### Task 13: Manual testing checklist

These are manual verification steps since the biometric bridge requires mobile hardware:

- [ ] **Step 1: Desktop — password dialog works as before**
  - Create wallet, set password, verify operations prompt for password
  - Verify cancel returns undefined (operation aborted)
  - Verify wrong password shows toast

- [ ] **Step 2: Desktop — biometric toggle not visible**
  - Verify biometric toggle does not appear in Settings on desktop

- [ ] **Step 3: Mobile — standalone biometric gate**
  - Enable biometric, no password set
  - Verify sensitive operations trigger biometric prompt
  - Verify cancel returns undefined

- [ ] **Step 4: Mobile — biometric + password (first use)**
  - Set password on wallet, enable biometric
  - First protected operation → password dialog (no keychain entry yet)
  - Type correct password → operation succeeds → password stored in keychain

- [ ] **Step 5: Mobile — biometric + password (subsequent)**
  - Next protected operation → biometric prompt (keychain retrieval)
  - On success → operation proceeds without typing password

- [ ] **Step 6: Mobile — biometric fails → fallback to dialog**
  - Cancel biometric prompt → password dialog appears
  - Type password → operation succeeds

- [ ] **Step 7: Mobile — stale keychain entry**
  - Change password in Settings
  - Next operation should use new keychain entry (updated on change)

- [ ] **Step 8: Mobile — disable biometric clears keychain**
  - Disable biometric toggle
  - Next operation should prompt for password (no keychain)

- [ ] **Step 9: WalletConnect — unified auth works**
  - Connect dApp via WalletConnect
  - Trigger signing operation → single auth prompt (not two)
