import type { SageSystemClient } from './types';

declare global {
  interface Window {
    __SAGE_SYSTEM__?: SageSystemClient;
    __SAGE_SYSTEM_RUNTIME_BRIDGE_INITIALIZED__?: boolean;
  }
}

export {};
