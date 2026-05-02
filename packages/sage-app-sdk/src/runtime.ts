import type * as Generated from './generated-types';
import type {
  SageBridgeSendPayload,
  SageBridgeVersion,
  SageClient,
} from './types';
import {
  createBridgeRuntimeCore,
  parseJsonOrNull,
} from './bridge-runtime-core';
import { applySageThemeCssVars, clearSageThemeCssVars } from './theme';

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
    __SAGE_APP_INFO__?: Generated.AppGetInfoResult;
    __SAGE_RUNTIME_BRIDGE_INITIALIZED__?: boolean;
  };

type RuntimeEventEnvelope<T = unknown> = {
  type: string;
  payload: T;
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

function buildFallbackAppInfo(): Generated.AppGetInfoResult {
  return {
    id: 'unknown',
    name: 'Unknown App',
    version: '0.0.0',
    requestedPermissions: {
      network: {
        whitelist: {
          required: [],
          optional: [],
        },
      },
      capabilities: {
        required: [],
        optional: [],
      },
    },
    capabilities: [],
    network: [],
  };
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function isRuntimeEventEnvelope(value: unknown): value is RuntimeEventEnvelope {
  return (
    isObject(value) && typeof value.type === 'string' && 'payload' in value
  );
}

function dispatchRuntimeEvent(data: RuntimeEventEnvelope) {
  window.dispatchEvent(
    new CustomEvent<RuntimeEventEnvelope>('sage:event', {
      detail: data,
    }),
  );

  window.dispatchEvent(
    new CustomEvent<RuntimeEventEnvelope>(`sage:event:${data.type}`, {
      detail: data,
    }),
  );
}

function onRuntimeEventType<T>(
  type: string,

  handler: (event: T) => void,
): () => void {
  const listener = (event: Event) => {
    const custom = event as CustomEvent<RuntimeEventEnvelope<T>>;

    handler(custom.detail.payload);
  };

  window.addEventListener(`sage:event:${type}`, listener as EventListener);

  return () => {
    window.removeEventListener(`sage:event:${type}`, listener as EventListener);
  };
}

function bridgeResponseResult(data: RustLikeBridgeSuccessResponse): unknown {
  if ('result' in data) {
    return data.result;
  }

  return parseJsonOrNull(data.resultJson);
}

function handleBeforeStopEvent(
  event: Generated.BeforeStopEvent,
  beforeStopHandlers: Set<
    (event: Generated.BeforeStopEvent) => void | Promise<void>
  >,
  callHost: <T>(method: string, params?: unknown) => Promise<T>,
  rejectAllPending: (reason: string) => void,
) {
  rejectAllPending('Sage runtime is stopping');

  if (!event?.requestId || beforeStopHandlers.size === 0) {
    return;
  }

  const handlers = Array.from(beforeStopHandlers);

  void Promise.allSettled(
    handlers.map((handler) => Promise.resolve(handler(event))),
  ).finally(() => {
    void callHost<Generated.RuntimeAckResult>('app.lifecycle.readyToStop', {
      requestId: event.requestId,
    } satisfies Generated.ReadyToStopParams).catch((error: unknown) => {
      console.error('Failed to acknowledge before-stop:', error);
    });
  });
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
  const callHost = core.callHost;
  const rejectAllPending = core.rejectAllPending;

  w.__SAGE_RUNTIME_BRIDGE_INITIALIZED__ = true;

  const beforeStopHandlers = new Set<
    (event: Generated.BeforeStopEvent) => void | Promise<void>
  >();
  let beforeStopRegistered = false;

  async function syncBeforeStopRegistration() {
    const shouldBeRegistered = beforeStopHandlers.size > 0;
    if (beforeStopRegistered === shouldBeRegistered) {
      return;
    }

    beforeStopRegistered = shouldBeRegistered;

    try {
      await callHost<Generated.RuntimeAckResult>(
        'app.lifecycle.setBeforeStopListener',
        {
          active: shouldBeRegistered,
        } satisfies Generated.SetBeforeStopListenerParams,
      );
    } catch (error) {
      console.error('Failed to sync before-stop listener registration:', error);
    }
  }

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

  onRuntimeEventType<Generated.BeforeStopEvent>(
    'lifecycle.beforeStop',
    (detail) => {
      handleBeforeStopEvent(
        detail,
        beforeStopHandlers,
        callHost,
        rejectAllPending,
      );
    },
  );

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

  w.__SAGE__ = {
    initialAppInfo: w.__SAGE_APP_INFO__ ?? buildFallbackAppInfo(),
    app: {
      async bridgePing() {
        return await callHost<Generated.BridgePingResult>('bridge.ping');
      },

      async bridgeSend(input: SageBridgeSendPayload) {
        return await callHost<Generated.BridgeSendResult>('bridge.send', input);
      },

      async getInfo() {
        return await callHost<Generated.AppGetInfoResult>('app.getInfo');
      },

      async getCapabilities() {
        return await callHost<string[]>('app.getCapabilities');
      },

      async requestCapabilityGrant(
        input: Generated.RequestCapabilityGrantParams,
      ) {
        return await callHost<Generated.RequestCapabilityGrantResult>(
          'app.requestCapabilityGrant',
          input,
        );
      },

      async requestNetworkWhitelistGrant(
        input: Generated.RequestNetworkWhitelistGrantParams,
      ) {
        return await callHost<Generated.RequestNetworkWhitelistGrantResult>(
          'app.requestNetworkWhitelistGrant',
          input,
        );
      },

      onGrantedCapabilitiesChange(handler) {
        return onRuntimeEventType<Generated.GrantedCapabilitiesChangeEvent>(
          'grantedCapabilitiesChange',
          handler,
        );
      },

      onGrantedNetworkWhitelistChange(handler) {
        return onRuntimeEventType<Generated.GrantedNetworkWhitelistChangeEvent>(
          'grantedNetworkWhitelistChange',
          handler,
        );
      },

      lifecycle: {
        onBeforeStop(handler) {
          beforeStopHandlers.add(handler);
          void syncBeforeStopRegistration();

          return () => {
            beforeStopHandlers.delete(handler);
            void syncBeforeStopRegistration();
          };
        },
      },
    },

    wallet: {
      async getKeys() {
        return await callHost<Generated.GetKeysResponse>('wallet.getKeys');
      },

      async getKey(input: Generated.GetKey) {
        return await callHost<Generated.GetKeyResponse>('wallet.getKey', input);
      },

      async getSecretKey(input: Generated.GetSecretKey) {
        return await callHost<Generated.GetSecretKeyResponse>(
          'wallet.getSecretKey',
          input,
        );
      },

      async getSyncStatus() {
        return await callHost<Generated.GetSyncStatusResponse>(
          'wallet.getSyncStatus',
        );
      },

      async getVersion() {
        return await callHost<Generated.GetVersionResponse>(
          'wallet.getVersion',
        );
      },

      async getPendingTransactions() {
        return await callHost<Generated.GetPendingTransactionsResponse>(
          'wallet.getPendingTransactions',
        );
      },

      async checkAddress(input: Generated.CheckAddress) {
        return await callHost<Generated.CheckAddressResponse>(
          'wallet.checkAddress',
          input,
        );
      },

      async getDerivations(input: Generated.GetDerivations) {
        return await callHost<Generated.GetDerivationsResponse>(
          'wallet.getDerivations',
          input,
        );
      },

      async getSpendableCoinCount(input: Generated.GetSpendableCoinCount) {
        return await callHost<Generated.GetSpendableCoinCountResponse>(
          'wallet.getSpendableCoinCount',
          input,
        );
      },

      async getCoinsByIds(input: Generated.GetCoinsByIds) {
        return await callHost<Generated.GetCoinsByIdsResponse>(
          'wallet.getCoinsByIds',
          input,
        );
      },

      async getCoins(input: Generated.GetCoins) {
        return await callHost<Generated.GetCoinsResponse>(
          'wallet.getCoins',
          input,
        );
      },

      async getTransaction(input: Generated.GetTransaction) {
        return await callHost<Generated.GetTransactionResponse>(
          'wallet.getTransaction',
          input,
        );
      },

      async getTransactions(input: Generated.GetTransactions) {
        return await callHost<Generated.GetTransactionsResponse>(
          'wallet.getTransactions',
          input,
        );
      },

      async sendXch(input: Generated.WalletSendXchParams) {
        return await callHost<Generated.TransactionResponse>(
          'wallet.sendXch',
          input,
        );
      },
    },
    environment: {
      theme: {
        async getCurrent() {
          return await callHost<Generated.EnvironmentThemeGetCurrentResult>(
            'environment.theme.getCurrent',
          );
        },

        onChanged(handler) {
          return onRuntimeEventType<Generated.EnvironmentThemeChangedEvent>(
            'environment.theme.changed',
            handler,
          );
        },

        async mountCssVars() {
          const current =
            await callHost<Generated.EnvironmentThemeGetCurrentResult>(
              'environment.theme.getCurrent',
            );

          applySageThemeCssVars(current.theme);

          const unlisten =
            onRuntimeEventType<Generated.EnvironmentThemeChangedEvent>(
              'environment.theme.changed',
              (event) => {
                applySageThemeCssVars(event.theme);
              },
            );

          return () => {
            unlisten();
            clearSageThemeCssVars();
          };
        },
      },
    },
  };

  return true;
}
