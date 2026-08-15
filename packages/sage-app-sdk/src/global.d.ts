import type { SageAppInfo, SageClient } from './types';

declare global {
  interface Window {
    __SAGE__?: SageClient;
    __SAGE_APP_INFO__?: SageAppInfo;
    __SAGE_RUNTIME_BRIDGE_INITIALIZED__?: boolean;
  }
}

export {};
