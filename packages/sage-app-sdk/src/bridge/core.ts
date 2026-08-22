import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import {
  RustBridgeErrorResponse,
  RustBridgeInvokeResult,
  RustBridgeSuccessResponse,
} from '../generated-types';
import { debugComms } from '../debug';

export type GenericBridgeRequest = {
  bridgeVersion?: string;
  id: string;
  method: string;
  params?: unknown;
};

export type GenericBridgeSuccessResponse = {
  bridgeVersion: string;
  id: string;
  ok: true;
  result: unknown;
};

export type GenericBridgeErrorResponse = {
  bridgeVersion: string;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

type ListenEvent<T = unknown> = {
  payload: T;
};

type Unlisten = () => void;

export type BridgeRuntimeWebviewHandle = {
  label: string;
  listen<T = unknown>(
    event: string,
    handler: (event: ListenEvent<T>) => void,
  ): Promise<Unlisten>;
};

type PendingBridgeRequest = {
  resolve: (value: unknown) => void;
  reject: (reason?: unknown) => void;
  timeoutId: number;
  method: string;
};

export type BridgeRuntimeCoreConfig = {
  version: string;
  invokeCommand: string;
  requestIdPrefix: string;
  timeoutMs?: number;
};

export type BridgeRuntimeCore = {
  webview: BridgeRuntimeWebviewHandle;
  pendingRequests: Map<string, PendingBridgeRequest>;
  callHost<T>(method: string, params?: unknown): Promise<T>;
  rejectAllPending(reason: string): void;
};

function tryGetCurrentWebview(): BridgeRuntimeWebviewHandle | null {
  try {
    return getCurrentWebview() as BridgeRuntimeWebviewHandle;
  } catch {
    return null;
  }
}

export function parseJsonOrNull(value: string | null | undefined): unknown {
  if (value == null) {
    return null;
  }

  try {
    return JSON.parse(value);
  } catch (err) {
    console.error('Failed to parse JSON payload from Rust bridge:', err, value);
    return null;
  }
}

export function toSdkBridgeSuccessResponse(
  version: string,
  response: RustBridgeSuccessResponse,
): GenericBridgeSuccessResponse {
  return {
    bridgeVersion: version,
    id: response.id,
    ok: true,
    result: parseJsonOrNull(response.resultJson),
  };
}

export function toSdkBridgeErrorResponse(
  version: string,
  response: RustBridgeErrorResponse,
): GenericBridgeErrorResponse {
  return {
    bridgeVersion: version,
    id: response.id,
    ok: false,
    error: response.error,
  };
}

export function createBridgeRuntimeCore(
  config: BridgeRuntimeCoreConfig,
): BridgeRuntimeCore | null {
  const maybeWebview = tryGetCurrentWebview();
  if (!maybeWebview) {
    return null;
  }

  const webview = maybeWebview;
  const pendingRequests = new Map<string, PendingBridgeRequest>();
  const timeoutMs = config.timeoutMs ?? 30000;

  function rejectAllPending(reason: string) {
    for (const [id, pending] of pendingRequests.entries()) {
      window.clearTimeout(pending.timeoutId);
      pending.reject(new Error(reason));
      pendingRequests.delete(id);
    }
  }

  async function callHost<T>(method: string, params?: unknown): Promise<T> {
    const id = `${config.requestIdPrefix}-${Date.now()}-${Math.random()
      .toString(36)
      .slice(2)}`;
    debugComms('request', { id, method, params });

    return await new Promise<T>((resolve, reject) => {
      const timeoutId = window.setTimeout(() => {
        const pending = pendingRequests.get(id);
        if (!pending) {
          return;
        }

        pendingRequests.delete(id);
        reject(new Error(`timeout for ${method}`));
      }, timeoutMs);

      pendingRequests.set(id, {
        resolve: (value) => resolve(value as T),
        reject,
        timeoutId,
        method,
      });

      void (async () => {
        try {
          const request: GenericBridgeRequest = {
            bridgeVersion: config.version,
            id,
            method,
            params,
          };

          const result = await invoke<RustBridgeInvokeResult>(
            config.invokeCommand,
            {
              request: {
                bridgeVersion: request.bridgeVersion ?? null,
                id: request.id,
                method: request.method,
                paramsJson:
                  request.params === undefined
                    ? null
                    : JSON.stringify(request.params),
              },
            },
          );
          debugComms('invoke-result', result);

          if (result.kind === 'success' || result.kind === 'error') {
            pendingRequests.delete(id);
            window.clearTimeout(timeoutId);
            if (result.kind === 'success') {
              const response = toSdkBridgeSuccessResponse(
                config.version,
                result,
              );
              resolve(response.result as T);
            } else {
              const response = toSdkBridgeErrorResponse(config.version, result);
              reject(new Error(response.error.message));
            }
          }
        } catch (error: unknown) {
          debugComms('request-error', { id, method, error });
          const pending = pendingRequests.get(id);
          if (!pending) {
            return;
          }

          pendingRequests.delete(id);
          window.clearTimeout(timeoutId);
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      })();
    });
  }

  return {
    webview,
    pendingRequests,
    callHost,
    rejectAllPending,
  };
}
