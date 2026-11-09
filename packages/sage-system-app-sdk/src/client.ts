import type { SageSystemClient } from './types';
import {
  getSageSystemClient as getRuntimeSageSystemClient,
  initSageSystemRuntimeBridge,
} from './runtime';

type SageSystemGlobal = typeof globalThis & {
  __TAURI__?: unknown;
  __SAGE_SYSTEM__?: SageSystemClient;
};

export function isSageSystemRuntimeAvailable(): boolean {
  return !!(globalThis as SageSystemGlobal).__TAURI__;
}

function getClientFromWindow(): SageSystemClient | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return (window as SageSystemGlobal).__SAGE_SYSTEM__;
}

export function isSageSystemBridgeInitialized(): boolean {
  return !!getClientFromWindow();
}

export function hasSageSystemBridge(): boolean {
  return !!getClientFromWindow();
}

export async function getSageSystemClient(): Promise<SageSystemClient> {
  return await getRuntimeSageSystemClient();
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

export { initSageSystemRuntimeBridge };
