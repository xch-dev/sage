import type {
  SandboxCapabilityResult,
  SandboxState,
  SandboxStateView,
} from '@/bindings';

export type SandboxCapability =
  | 'storage_isolation_from_sage'
  | 'storage_persistence_normal'
  | 'storage_non_persistence_incognito'
  | 'network_allowlist_enforced';

export function formatCapabilityLabel(capability: SandboxCapability): string {
  switch (capability) {
    case 'storage_isolation_from_sage':
      return 'storage isolation from Sage';
    case 'storage_persistence_normal':
      return 'persistent storage behavior';
    case 'storage_non_persistence_incognito':
      return 'incognito storage behavior';
    case 'network_allowlist_enforced':
      return 'network allowlist enforcement';
  }
}

export function listSandboxCapabilities(
  sandbox: SandboxState,
): [SandboxCapability, SandboxCapabilityResult][] {
  return [
    ['storage_isolation_from_sage', sandbox.storageIsolationFromSage],
    ['storage_persistence_normal', sandbox.storagePersistenceNormal],
    [
      'storage_non_persistence_incognito',
      sandbox.storageNonPersistenceIncognito,
    ],
    ['network_allowlist_enforced', sandbox.networkAllowlistEnforced],
  ];
}

export function getLiveSandboxState(
  sandboxView: SandboxStateView | null | undefined,
): SandboxState | null {
  return sandboxView?.currentRun?.state ?? null;
}

export function getEffectiveSandboxState(
  sandboxView: SandboxStateView | null | undefined,
): SandboxState | null {
  return sandboxView?.effective ?? null;
}

export function getBaselineSandboxState(
  sandboxView: SandboxStateView | null | undefined,
): SandboxState | null {
  return sandboxView?.baseline ?? null;
}
