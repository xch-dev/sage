import type {
  SandboxCapabilityResult,
  SandboxState,
  SandboxStateView,
  SageAppView,
  SystemSageAppView,
  UserSageAppView,
} from '@/bindings';

type AppLike = SageAppView | UserSageAppView | SystemSageAppView;

export type SandboxCapability =
  | 'storage_isolation_from_sage'
  | 'storage_persistence_normal'
  | 'storage_non_persistence_incognito'
  | 'storage_clear_cycle'
  | 'network_allowlist_enforced';

export interface AppLaunchGateResult {
  allowed: boolean;
  kind: 'allowed' | 'running' | 'failed';
  capability: SandboxCapability | null;
  message: string | null;
}

export function formatCapabilityLabel(capability: SandboxCapability): string {
  switch (capability) {
    case 'storage_isolation_from_sage':
      return 'storage isolation from Sage';
    case 'storage_persistence_normal':
      return 'persistent storage behavior';
    case 'storage_non_persistence_incognito':
      return 'incognito storage behavior';
    case 'storage_clear_cycle':
      return 'storage clear cycle behavior';
    case 'network_allowlist_enforced':
      return 'network allowlist enforcement';
  }
}

export function getSandboxCapabilityResult(
  sandbox: SandboxState,
  capability: SandboxCapability,
): SandboxCapabilityResult {
  switch (capability) {
    case 'storage_isolation_from_sage':
      return sandbox.storageIsolationFromSage;
    case 'storage_persistence_normal':
      return sandbox.storagePersistenceNormal;
    case 'storage_non_persistence_incognito':
      return sandbox.storageNonPersistenceIncognito;
    case 'storage_clear_cycle':
      return sandbox.storageClearCycle;
    case 'network_allowlist_enforced':
      return sandbox.networkAllowlistEnforced;
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
    ['storage_clear_cycle', sandbox.storageClearCycle],
    ['network_allowlist_enforced', sandbox.networkAllowlistEnforced],
  ];
}

export function getRequiredSandboxCapabilities(
  app: AppLike,
): SandboxCapability[] {
  const required: SandboxCapability[] = ['storage_isolation_from_sage'];

  if (
    app.common.grantedPermissions.capabilities.includes('storage.persistent_webview')
  ) {
    required.push('storage_persistence_normal');
  } else {
    required.push('storage_non_persistence_incognito');
  }

  if ((app.common.grantedPermissions.network.whitelist?.length ?? 0) > 0) {
    required.push('network_allowlist_enforced');
  }

  return required;
}

export function evaluateAppLaunchGate(
  app: AppLike,
  sandbox: SandboxState,
): AppLaunchGateResult {
  const isolation = sandbox.storageIsolationFromSage;

  if (isolation.status === 'pending' || isolation.status === 'running') {
    return {
      allowed: false,
      kind: 'running',
      capability: 'storage_isolation_from_sage',
      message: `Sandbox tests are still running for ${formatCapabilityLabel(
        'storage_isolation_from_sage',
      )}.`,
    };
  }

  if (isolation.status === 'failed') {
    return {
      allowed: false,
      kind: 'failed',
      capability: 'storage_isolation_from_sage',
      message:
        isolation.details ??
        `Sandbox test failed for ${formatCapabilityLabel(
          'storage_isolation_from_sage',
        )}.`,
    };
  }

  const required = getRequiredSandboxCapabilities(app);

  for (const capability of required) {
    const result = getSandboxCapabilityResult(sandbox, capability);

    if (result.status === 'pending' || result.status === 'running') {
      return {
        allowed: false,
        kind: 'running',
        capability,
        message: `Sandbox tests are still running for ${formatCapabilityLabel(capability)}.`,
      };
    }

    if (result.status === 'failed') {
      return {
        allowed: false,
        kind: 'failed',
        capability,
        message:
          result.details ??
          `Sandbox test failed for ${formatCapabilityLabel(capability)}.`,
      };
    }
  }

  return {
    allowed: true,
    kind: 'allowed',
    capability: null,
    message: null,
  };
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
