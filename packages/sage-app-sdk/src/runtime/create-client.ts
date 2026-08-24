import type * as Generated from '../generated-types';
import type { BridgeRuntimeCore } from '../bridge/core';
import type { SageBridgeSendPayload, SageClient } from '../types';
import { applySageThemeCssVars, clearSageThemeCssVars } from '../theme';
import { onRuntimeEventType } from './events';
import { handleBeforeStopEvent } from './lifecycle';

type SageWindow = Window &
  typeof globalThis & {
    __SAGE_APP_INFO__?: Generated.AppGetInfoResult;
  };

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
        whitelistByNetwork: {},
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

export function createSageClient(core: BridgeRuntimeCore): SageClient {
  const w = getSageWindow();
  const callHost = core.callHost;
  const rejectAllPending = core.rejectAllPending;

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

  return {
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

      async requestPermissionGrants(
        input: Generated.RequestPermissionGrantsParams,
      ) {
        return await callHost<Generated.RequestPermissionGrantsResult>(
          'app.requestPermissionGrants',
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
      async sendTransaction(input: Generated.WalletSendTransactionParams) {
        return await callHost<Generated.WalletSendTransactionResult>(
          'wallet.sendTransaction',
          input,
        );
      },

      async signMessage(input: Generated.WalletSignMessageParams) {
        return await callHost<Generated.WalletSignMessageResult>(
          'wallet.signMessage',
          input,
        );
      },

      async signCoinSpends(input: Generated.WalletSignCoinSpendsParams) {
        return await callHost<Generated.WalletSignCoinSpendsResult>(
          'wallet.signCoinSpends',
          input,
        );
      },

      async getAssetBalance(input: Generated.WalletGetAssetBalanceParams) {
        return await callHost<Generated.WalletGetAssetBalanceResult>(
          'wallet.getAssetBalance',
          input,
        );
      },

      async getAssetCoins(input: Generated.WalletGetAssetCoinsParams) {
        return await callHost<Generated.WalletGetAssetCoinsResult>(
          'wallet.getAssetCoins',
          input,
        );
      },

      async filterUnlockedCoins(
        input: Generated.WalletFilterUnlockedCoinsParams,
      ) {
        return await callHost<Generated.WalletFilterUnlockedCoinsResult>(
          'wallet.filterUnlockedCoins',
          input,
        );
      },

      async getPublicKeys(input?: Generated.WalletGetPublicKeysParams) {
        return await callHost<Generated.WalletGetPublicKeysResult>(
          'wallet.getPublicKeys',
          input,
        );
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
      async getXchUsdPrice() {
        return await callHost<Generated.GetXchUsdPriceResponse>(
          'wallet.getXchUsdPrice',
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
      getNetwork() {
        return callHost<Generated.EnvironmentGetNetworkResult>(
          'environment.getNetwork',
        );
      },
    },
  };
}
