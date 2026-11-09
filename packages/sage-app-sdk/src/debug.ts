type SageDebugWindow = Window &
  typeof globalThis & {
    __SAGE_APPS_COMMS_DEBUG__?: boolean;
  };

export function sageAppsCommsDebugEnabled(): boolean {
  if (typeof window === 'undefined') return false;

  return (window as SageDebugWindow).__SAGE_APPS_COMMS_DEBUG__ === true;
}

export function debugComms(label: string, payload?: unknown) {
  if (!sageAppsCommsDebugEnabled()) return;

  if (payload === undefined) {
    console.debug(`[Sage Comms] ${label}`);
    return;
  }

  console.debug(`[Sage Comms] ${label}`, payload);
}
