import { initSageRuntimeBridge } from './runtime';
import type { SageClient } from './types';
import { bootstrapTheme } from './theme/bootstrap';
import { formatSageError } from './client/errors';

type SageGlobal = typeof globalThis & {
  __TAURI__?: unknown;
};

export { formatSageError };

export function isSageRuntimeAvailable(): boolean {
  return !!(globalThis as SageGlobal).__TAURI__;
}

function getClientFromWindow(): SageClient | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return window.__SAGE__;
}

export function isSageBridgeInitialized(): boolean {
  return !!getClientFromWindow();
}

export function hasSageBridge(): boolean {
  return !!getClientFromWindow();
}

export async function getSageClient(): Promise<SageClient> {
  let client = getClientFromWindow();

  if (!client) {
    initSageRuntimeBridge();
    client = getClientFromWindow();
  }

  if (!client) {
    throw new Error('Sage bridge is unavailable in this runtime.');
  }

  bootstrapTheme(client);

  return client;
}
