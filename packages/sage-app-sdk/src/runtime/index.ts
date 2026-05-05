import type { SageBridgeVersion } from '../types';
import { createBridgeRuntimeCore, parseJsonOrNull } from '../bridge/core';
import type { SageClient } from '../types';
import { createSageClient } from './create-client';
import { dispatchRuntimeEvent, isRuntimeEventEnvelope } from './events';

export const SAGE_BRIDGE_VERSION: SageBridgeVersion = 'v1';

type SageListenEvent<T = unknown> = {
  payload: T;
};

type SageUnlisten = () => void;

type SageWebviewHandle = {
  label: string;
  listen<T = unknown>(
    event: string,
    handler: (event: SageListenEvent<T>) => void,
  ): Promise<SageUnlisten>;
};

type SageWindow = Window &
  typeof globalThis & {
    __SAGE__?: SageClient;
    __SAGE_RUNTIME_BRIDGE_INITIALIZED__?: boolean;
  };

type RustLikeBridgeSuccessResponse = {
  bridgeVersion: SageBridgeVersion;
  id: string;
  ok: true;
  result?: unknown;
  resultJson?: string;
};

type RustLikeBridgeErrorResponse = {
  bridgeVersion: SageBridgeVersion;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

type RustLikeBridgeResponse =
  | RustLikeBridgeSuccessResponse
  | RustLikeBridgeErrorResponse;

function getSageWindow(): SageWindow {
  return window as SageWindow;
}

function bridgeResponseResult(data: RustLikeBridgeSuccessResponse): unknown {
  if ('result' in data) {
    return data.result;
  }

  return parseJsonOrNull(data.resultJson);
}

export function initSageRuntimeBridge(): boolean {
  const w = getSageWindow();

  if (w.__SAGE__) {
    return true;
  }

  if (w.__SAGE_RUNTIME_BRIDGE_INITIALIZED__) {
    return true;
  }

  const core = createBridgeRuntimeCore({
    version: SAGE_BRIDGE_VERSION,
    invokeCommand: 'apps_invoke_bridge',
    requestIdPrefix: 'sage',
  });

  if (!core) {
    return false;
  }

  const webview = core.webview as SageWebviewHandle;

  w.__SAGE_RUNTIME_BRIDGE_INITIALIZED__ = true;

  webview
    .listen('sage-bridge:event', (event: SageListenEvent) => {
      const data = event.payload;

      if (!isRuntimeEventEnvelope(data)) {
        console.warn('[Sage SDK] Dropped malformed runtime event:', data);
        return;
      }

      try {
        dispatchRuntimeEvent(data);
      } catch (error: unknown) {
        console.error('Failed to dispatch Sage bridge runtime event:', error);
      }
    })
    .catch((error: unknown) => {
      console.error('Failed to subscribe to sage-bridge:event:', error);
    });

  webview
    .listen<RustLikeBridgeResponse>(
      'sage-bridge:response',
      (event: SageListenEvent<RustLikeBridgeResponse>) => {
        const data = event.payload;

        if (!data || data.bridgeVersion !== SAGE_BRIDGE_VERSION) {
          console.warn('[Sage SDK] Dropped malformed bridge response:', data);
          return;
        }

        const pending = core.pendingRequests.get(data.id);
        if (!pending) {
          console.warn(
            '[Sage SDK] Response for unknown request id:',
            data.id,
            data,
          );
          return;
        }

        core.pendingRequests.delete(data.id);
        window.clearTimeout(pending.timeoutId);

        if (data.ok) {
          pending.resolve(bridgeResponseResult(data));
        } else {
          pending.reject(
            new Error(data.error?.message || 'Unknown Sage bridge error'),
          );
        }
      },
    )
    .catch((error: unknown) => {
      console.error('Failed to subscribe to sage-bridge:response:', error);
    });

  w.__SAGE__ = createSageClient(core);

  return true;
}
