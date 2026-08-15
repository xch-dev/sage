import type { SandboxStateView } from './sandboxApi';

export type SandboxTab = 'effective' | 'previous' | 'current';

export type SandboxGateState = NonNullable<SandboxStateView['effective']>;

export function getLiveSandboxState(state: SandboxStateView | null) {
  return state?.currentRun?.state ?? null;
}

export function getBaselineSandboxState(state: SandboxStateView | null) {
  return state?.baseline ?? null;
}

export function getEffectiveSandboxState(state: SandboxStateView | null) {
  return state?.effective ?? null;
}

export function isCurrentSandboxRunActive(state: SandboxStateView | null) {
  return state?.currentRun?.state?.overallCriticalStatus === 'running';
}

export function selectedSandboxState(
  state: SandboxStateView | null,
  tab: SandboxTab,
): SandboxGateState | null {
  switch (tab) {
    case 'current':
      return getLiveSandboxState(state);
    case 'previous':
      return getBaselineSandboxState(state);
    case 'effective':
      return getEffectiveSandboxState(state);
  }
}

export function formatCapabilityLabel(value: string): string {
  return value
    .replace(/^sandbox\./, '')
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

export function listSandboxCapabilities(state: SandboxGateState | null) {
  if (!state) return [];

  return Object.entries(state)
    .filter(
      ([key]) =>
        key !== 'overallCriticalStatus' &&
        key !== 'startedAt' &&
        key !== 'finishedAt',
    )
    .sort(([a], [b]) =>
      formatCapabilityLabel(a).localeCompare(formatCapabilityLabel(b)),
    );
}
