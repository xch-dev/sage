import type * as Generated from './generated-types';
import type {
  SageBridgeResponse,
  SageBridgeRuntimeEvent,
  SageBridgeSendPayload,
  SageBridgeVersion,
  SageClient,
} from './types';
import { createBridgeRuntimeCore } from './bridge-runtime-core';

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

type BeforeStopPublicDetail = Omit<
  Generated.SageLifecycleBeforeStopDetail,
  'requestId'
>;

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

function isBridgeRuntimeEvent(value: unknown): value is SageBridgeRuntimeEvent {
  if (!isObject(value)) {
    return false;
  }

  return (
    value.type === 'grantedCapabilitiesChange' ||
    value.type === 'grantedNetworkWhitelistChange'
  );
}

function isGrantedCapabilitiesChangeEvent(
  value: unknown,
): value is Generated.GrantedCapabilitiesChangeEvent {
  return (
    isObject(value) &&
    value.type === 'grantedCapabilitiesChange'
  );
}

function isGrantedNetworkWhitelistChangeEvent(
  value: unknown,
): value is Generated.GrantedNetworkWhitelistChangeEvent {
  return (
    isObject(value) &&
    value.type === 'grantedNetworkWhitelistChange'
  );
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
    (detail: BeforeStopPublicDetail) => void | Promise<void>
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

      if (!isBridgeRuntimeEvent(data)) {
        console.warn('[Sage SDK] Dropped unknown runtime event:', data);
        return;
      }

      try {
        if (isGrantedCapabilitiesChangeEvent(data)) {
          window.dispatchEvent(
            new CustomEvent<Generated.GrantedCapabilitiesChangeEvent>(
              'app:granted-capabilities-change',
              { detail: data },
            ),
          );
          return;
        }

        if (isGrantedNetworkWhitelistChangeEvent(data)) {
          window.dispatchEvent(
            new CustomEvent<Generated.GrantedNetworkWhitelistChangeEvent>(
              'app:granted-network-whitelist-change',
              { detail: data },
            ),
          );
        }
      } catch (error: unknown) {
        console.error('Failed to dispatch Sage bridge runtime event:', error);
      }
    })
    .catch((error: unknown) => {
      console.error('Failed to subscribe to sage-bridge:event:', error);
    });

  webview
    .listen<SageBridgeResponse>(
      'sage-bridge:response',
      (event: SageListenEvent<SageBridgeResponse>) => {
        const data = event.payload;

        if (
          !data ||
          data.bridgeVersion !== SAGE_BRIDGE_VERSION
        ) {
          return;
        }

        const pending = core.pendingRequests.get(data.id);
        if (!pending) {
          return;
        }

        core.pendingRequests.delete(data.id);
        window.clearTimeout(pending.timeoutId);

        if (data.ok) {
          pending.resolve(data.result as unknown);
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

  webview
    .listen<Generated.SageLifecycleBeforeStopDetail>(
      'sage-lifecycle:before-stop',
      (event: SageListenEvent<Generated.SageLifecycleBeforeStopDetail>) => {
        const detail = event.payload;
        rejectAllPending('Sage runtime is stopping');

        if (!detail?.requestId || beforeStopHandlers.size === 0) {
          return;
        }

        const publicDetail: BeforeStopPublicDetail = {
          reason: detail.reason,
          appId: detail.appId,
          runtimeId: detail.runtimeId,
        };

        const handlers = Array.from(beforeStopHandlers);

        void Promise.allSettled(
          handlers.map((handler) => Promise.resolve(handler(publicDetail))),
        ).finally(() => {
          void callHost<Generated.RuntimeAckResult>(
            'app.lifecycle.readyToStop',
            {
              requestId: detail.requestId,
            } satisfies Generated.ReadyToStopParams,
          ).catch((error: unknown) => {
            console.error('Failed to acknowledge before-stop:', error);
          });
        });
      },
    )
    .catch((error: unknown) => {
      console.error(
        'Failed to subscribe to sage-lifecycle:before-stop:',
        error,
      );
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
        const listener = (event: Event) => {
          const custom =
            event as CustomEvent<Generated.GrantedCapabilitiesChangeEvent>;
          handler(custom.detail);
        };

        window.addEventListener(
          'app:granted-capabilities-change',
          listener as EventListener,
        );

        return () => {
          window.removeEventListener(
            'app:granted-capabilities-change',
            listener as EventListener,
          );
        };
      },

      onGrantedNetworkWhitelistChange(handler) {
        const listener = (event: Event) => {
          const custom =
            event as CustomEvent<Generated.GrantedNetworkWhitelistChangeEvent>;
          handler(custom.detail);
        };

        window.addEventListener(
          'app:granted-network-whitelist-change',
          listener as EventListener,
        );

        return () => {
          window.removeEventListener(
            'app:granted-network-whitelist-change',
            listener as EventListener,
          );
        };
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
  };

  return true;
}
