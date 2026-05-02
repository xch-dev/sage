import type {
  RuntimeTargetParams,
  SageAppRuntimeRecordView,
  SageSystemBridgeVersion,
  SageSystemClient,
  SageSystemRuntimeManagerClient,
  SystemKillRuntimeResult,
  RuntimeManagerRuntimesChangedEvent,
} from './types';
import {
  createBridgeRuntimeCore,
  getSageClient,
  parseJsonOrNull,
} from '@sage-app/sdk';

export const SAGE_SYSTEM_BRIDGE_VERSION: SageSystemBridgeVersion = 'v1';

type SageSystemListenEvent<T = unknown> = {
  payload: T;
};

type SageUnlisten = () => void;

type SageWebviewHandle = {
  label: string;
  listen<T = unknown>(
    event: string,
    handler: (event: SageSystemListenEvent<T>) => void,
  ): Promise<SageUnlisten>;
};

type SageSystemWindow = Window &
  typeof globalThis & {
    __SAGE_SYSTEM_RUNTIME__?: SageSystemRuntimeManagerClient;
    __SAGE_SYSTEM__?: SageSystemClient;
    __SAGE_SYSTEM_RUNTIME_BRIDGE_INITIALIZED__?: boolean;
  };

type SystemRuntimeEventEnvelope<T = unknown> = {
  type: string;
  payload: T;
};

type RustLikeSystemBridgeSuccessResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: true;
  result?: unknown;
  resultJson?: string;
};

type RustLikeSystemBridgeErrorResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

type RustLikeSystemBridgeResponse =
  | RustLikeSystemBridgeSuccessResponse
  | RustLikeSystemBridgeErrorResponse;

function getSageWindow(): SageSystemWindow {
  return window as SageSystemWindow;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function isSystemRuntimeEventEnvelope(
  value: unknown,
): value is SystemRuntimeEventEnvelope {
  return (
    isObject(value) && typeof value.type === 'string' && 'payload' in value
  );
}

function dispatchSystemRuntimeEvent(data: SystemRuntimeEventEnvelope) {
  window.dispatchEvent(
    new CustomEvent<SystemRuntimeEventEnvelope>('sage-system:event', {
      detail: data,
    }),
  );

  window.dispatchEvent(
    new CustomEvent<SystemRuntimeEventEnvelope>(
      `sage-system:event:${data.type}`,
      { detail: data },
    ),
  );
}

function onSystemRuntimeEventType<T>(
  type: string,
  handler: (event: T) => void,
): () => void {
  const listener = (event: Event) => {
    const custom = event as CustomEvent<SystemRuntimeEventEnvelope<T>>;
    handler(custom.detail.payload);
  };

  window.addEventListener(
    `sage-system:event:${type}`,
    listener as EventListener,
  );

  return () => {
    window.removeEventListener(
      `sage-system:event:${type}`,
      listener as EventListener,
    );
  };
}

function bridgeResponseResult(
  data: RustLikeSystemBridgeSuccessResponse,
): unknown {
  if ('result' in data) {
    return data.result;
  }

  return parseJsonOrNull(data.resultJson);
}

export function initSageSystemRuntimeBridge(): boolean {
  const w = getSageWindow();

  if (w.__SAGE_SYSTEM_RUNTIME_BRIDGE_INITIALIZED__) {
    return true;
  }

  const core = createBridgeRuntimeCore({
    version: SAGE_SYSTEM_BRIDGE_VERSION,
    invokeCommand: 'apps_invoke_system_bridge',
    requestIdPrefix: 'sage-system',
  });

  if (!core) {
    return false;
  }

  const webview = core.webview as SageWebviewHandle;
  const callHost = core.callHost;

  w.__SAGE_SYSTEM_RUNTIME_BRIDGE_INITIALIZED__ = true;

  webview
    .listen<RustLikeSystemBridgeResponse>(
      'sage-system-bridge:response',
      (event: SageSystemListenEvent<RustLikeSystemBridgeResponse>) => {
        const data = event.payload;

        if (!data || data.bridgeVersion !== SAGE_SYSTEM_BRIDGE_VERSION) {
          console.warn(
            '[Sage System SDK] Dropped malformed bridge response:',
            data,
          );
          return;
        }

        const pending = core.pendingRequests.get(data.id);
        if (!pending) {
          console.warn(
            '[Sage System SDK] Response for unknown request id:',
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
            new Error(
              data.error?.message || 'Unknown Sage system bridge error',
            ),
          );
        }
      },
    )
    .catch((error: unknown) => {
      console.error('Failed to subscribe to system bridge response:', error);
    });

  webview
    .listen('sage-system-bridge:event', (event: SageSystemListenEvent) => {
      const data = event.payload;

      if (!isSystemRuntimeEventEnvelope(data)) {
        console.warn(
          '[Sage System SDK] Dropped malformed runtime event:',
          data,
        );
        return;
      }

      try {
        dispatchSystemRuntimeEvent(data);
      } catch (error: unknown) {
        console.error(
          'Failed to dispatch Sage system bridge runtime event:',
          error,
        );
      }
    })
    .catch((error: unknown) => {
      console.error('Failed to subscribe to system bridge event:', error);
    });

  w.__SAGE_SYSTEM_RUNTIME__ = {
    async listRuntimes() {
      return await callHost<SageAppRuntimeRecordView[]>(
        'runtimeManager.listRuntimes',
      );
    },

    async focusRuntime(input: RuntimeTargetParams) {
      return await callHost<SageAppRuntimeRecordView>(
        'runtimeManager.focusRuntime',
        input,
      );
    },

    async hideRuntime(input: RuntimeTargetParams) {
      return await callHost<SageAppRuntimeRecordView>(
        'runtimeManager.hideRuntime',
        input,
      );
    },

    async killRuntime(input: RuntimeTargetParams) {
      return await callHost<SystemKillRuntimeResult>(
        'runtimeManager.killRuntime',
        input,
      );
    },

    onRuntimesChanged(handler) {
      return onSystemRuntimeEventType<RuntimeManagerRuntimesChangedEvent>(
        'runtimeManager.runtimesChanged',
        handler,
      );
    },
  };

  return true;
}

export async function getSageSystemClient(): Promise<SageSystemClient> {
  if (!initSageSystemRuntimeBridge()) {
    throw new Error('Sage system bridge failed to initialize');
  }

  const w = getSageWindow();

  if (!w.__SAGE_SYSTEM_RUNTIME__) {
    throw new Error('Sage system runtime client is not initialized');
  }

  if (w.__SAGE_SYSTEM__) {
    return w.__SAGE_SYSTEM__;
  }

  const userClient = await getSageClient();

  w.__SAGE_SYSTEM__ = {
    ...userClient,
    runtimeManager: w.__SAGE_SYSTEM_RUNTIME__,
  };

  return w.__SAGE_SYSTEM__;
}
