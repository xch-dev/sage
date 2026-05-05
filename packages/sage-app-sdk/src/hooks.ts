import { getSageClient, hasSageBridge } from './client';
import type { SageClient } from './types';

let promise: Promise<SageClient> | null = null;

export function useSageClient(): SageClient {
  if (typeof window !== 'undefined' && window.__SAGE__) {
    return window.__SAGE__;
  }

  if (!hasSageBridge()) {
    throw new Error('Sage bridge is not available');
  }

  promise ??= getSageClient();

  throw promise;
}
