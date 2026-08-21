import type { AppLaunchGateResult } from '@/bindings';
import { formatCapabilityLabel } from '@/lib/apps/sandbox';

export interface SandboxLaunchDecision {
  allowed: boolean;
  warning: boolean;
  title: string;
  description: string;
}

export function formatSandboxLaunchDecision(
  gate: AppLaunchGateResult | null | undefined,
): SandboxLaunchDecision {
  if (!gate) {
    return {
      allowed: false,
      warning: false,
      title: 'Sandbox tests are still running',
      description:
        'Apps are allowed to launch only when all required sandbox capabilities have passed.',
    };
  }

  if (gate.allowed) {
    if (gate.kind === 'sageUntested') {
      return {
        allowed: true,
        warning: true,
        title: 'Compatibility not verified',
        description:
          gate.message ??
          'This app has not been tested with this version of Sage.',
      };
    }

    return {
      allowed: true,
      warning: false,
      title: 'Sandbox checks passed',
      description: 'This app is allowed to launch.',
    };
  }

  if (gate.kind === 'sandboxPending') {
    return {
      allowed: false,
      warning: false,
      title: 'Sandbox tests are still running',
      description:
        gate.message ??
        (gate.capability
          ? `Sandbox tests are still running for ${formatCapabilityLabel(gate.capability)}.`
          : 'Apps are allowed to launch only when all required sandbox capabilities have passed.'),
    };
  }

  const capabilityLabel = gate.capability
    ? formatCapabilityLabel(gate.capability)
    : null;

  return {
    allowed: false,
    warning: false,
    title:
      gate.kind === 'requiresNewerSage'
        ? 'Requires newer Sage'
        : gate.kind === 'invalidSageVersion'
          ? 'Invalid version requirement'
          : capabilityLabel
            ? `${capabilityLabel} failed`
            : 'Sandbox test failed',
    description:
      gate.message ??
      (capabilityLabel
        ? `This app cannot be launched because ${capabilityLabel} failed.`
        : 'This app cannot be launched because a required sandbox capability failed.'),
  };
}
