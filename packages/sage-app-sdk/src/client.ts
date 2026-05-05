import { initSageRuntimeBridge } from './runtime';
import type { SageClient } from './types';

type SageGlobal = typeof globalThis & {
  __TAURI__?: unknown;
};

let themeBootstrapStarted = false;

export function isSageRuntimeAvailable(): boolean {
  return !!(globalThis as SageGlobal).__TAURI__;
}

function getClientFromWindow(): SageClient | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return window.__SAGE__;
}

function bootstrapTheme(client: SageClient) {
  if (themeBootstrapStarted) return;
  themeBootstrapStarted = true;

  void client.environment.theme.mountCssVars().catch((err) => {
    console.debug('[Sage SDK] Theme CSS vars not mounted:', err);
  });

  try {
    client.environment.theme.onChanged?.(() => {
      void client.environment.theme.mountCssVars().catch((err) => {
        console.debug('[Sage SDK] Theme CSS vars refresh failed:', err);
      });
    });
  } catch (err) {
    console.debug('[Sage SDK] Theme change listener not available:', err);
  }
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

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

export function formatSageError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;

  if (isObject(err)) {
    if (typeof err.message === 'string') return err.message;
    if (typeof err.reason === 'string') return err.reason;

    try {
      return JSON.stringify(err, null, 2);
    } catch {
      return 'Unknown Sage error';
    }
  }

  return String(err);
}
