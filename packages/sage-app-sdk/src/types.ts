import type * as Generated from './generated-types';

export * from './generated-types';

export type SageBridgeVersion = 'v1';

export type SageBridgeSendPayload = {
  kind: string;
  [key: string]: unknown;
};

export type SageBridgeSuccessResponse = {
  bridgeVersion: SageBridgeVersion;
  id: string;
  ok: true;
  result: unknown;
};

export type SageBridgeErrorResponse = {
  bridgeVersion: SageBridgeVersion;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

export type SageWalletClient = {
  sendTransaction(
    input: Generated.WalletSendTransactionParams,
  ): Promise<Generated.WalletSendTransactionResult>;
  signMessage(
    input: Generated.WalletSignMessageParams,
  ): Promise<Generated.WalletSignMessageResult>;
  signCoinSpends(
    input: Generated.WalletSignCoinSpendsParams,
  ): Promise<Generated.WalletSignCoinSpendsResult>;
  getAssetBalance(
    input: Generated.WalletGetAssetBalanceParams,
  ): Promise<Generated.WalletGetAssetBalanceResult>;
  getAssetCoins(
    input: Generated.WalletGetAssetCoinsParams,
  ): Promise<Generated.WalletGetAssetCoinsResult>;
  filterUnlockedCoins(
    input: Generated.WalletFilterUnlockedCoinsParams,
  ): Promise<Generated.WalletFilterUnlockedCoinsResult>;
  getPublicKeys(
    input?: Generated.WalletGetPublicKeysParams,
  ): Promise<Generated.WalletGetPublicKeysResult>;
  getKey(input: Generated.GetKey): Promise<Generated.GetKeyResponse>;
  getSecretKey(
    input: Generated.GetSecretKey,
  ): Promise<Generated.GetSecretKeyResponse>;

  getSyncStatus(): Promise<Generated.GetSyncStatusResponse>;
  getVersion(): Promise<Generated.GetVersionResponse>;
  getPendingTransactions(): Promise<Generated.GetPendingTransactionsResponse>;
  getXchUsdPrice(): Promise<Generated.GetXchUsdPriceResponse>;

  checkAddress(
    input: Generated.CheckAddress,
  ): Promise<Generated.CheckAddressResponse>;
  getDerivations(
    input: Generated.GetDerivations,
  ): Promise<Generated.GetDerivationsResponse>;
  getSpendableCoinCount(
    input: Generated.GetSpendableCoinCount,
  ): Promise<Generated.GetSpendableCoinCountResponse>;
  getCoinsByIds(
    input: Generated.GetCoinsByIds,
  ): Promise<Generated.GetCoinsByIdsResponse>;
  getCoins(input: Generated.GetCoins): Promise<Generated.GetCoinsResponse>;
  getTransaction(
    input: Generated.GetTransaction,
  ): Promise<Generated.GetTransactionResponse>;
  getTransactions(
    input: Generated.GetTransactions,
  ): Promise<Generated.GetTransactionsResponse>;

  sendXch(
    input: Generated.WalletSendXchParams,
  ): Promise<Generated.TransactionResponse>;
};

export type SageAppLifecycleClient = {
  onBeforeStop(
    handler: (event: Generated.BeforeStopEvent) => void | Promise<void>,
  ): () => void;
};

export type SageAppClient = {
  bridgePing(): Promise<Generated.BridgePingResult>;
  bridgeSend(input: SageBridgeSendPayload): Promise<Generated.BridgeSendResult>;
  getInfo(): Promise<Generated.AppGetInfoResult>;
  getCapabilities(): Promise<string[]>;
  /** @deprecated Use requestPermissionGrants instead. */
  requestCapabilityGrant(
    input: Generated.RequestCapabilityGrantParams,
  ): Promise<Generated.RequestCapabilityGrantResult>;
  /** @deprecated Use requestPermissionGrants instead. */
  requestNetworkWhitelistGrant(
    input: Generated.RequestNetworkWhitelistGrantParams,
  ): Promise<Generated.RequestNetworkWhitelistGrantResult>;
  requestPermissionGrants(
    input: Generated.RequestPermissionGrantsParams,
  ): Promise<Generated.RequestPermissionGrantsResult>;
  onGrantedCapabilitiesChange(
    handler: (event: Generated.GrantedCapabilitiesChangeEvent) => void,
  ): () => void;
  onGrantedNetworkWhitelistChange(
    handler: (event: Generated.GrantedNetworkWhitelistChangeEvent) => void,
  ): () => void;
  lifecycle: SageAppLifecycleClient;
};

export type SageEnvironmentThemeClient = {
  getCurrent(): Promise<Generated.EnvironmentThemeGetCurrentResult>;

  onChanged(
    handler: (event: Generated.EnvironmentThemeChangedEvent) => void,
  ): () => void;

  mountCssVars(): Promise<() => void>;
};

export type SageEnvironmentClient = {
  theme: SageEnvironmentThemeClient;
  getNetwork(): Promise<Generated.EnvironmentGetNetworkResult>;
};

export type SageClient = {
  initialAppInfo: Generated.AppGetInfoResult;
  app: SageAppClient;
  wallet: SageWalletClient;
  environment: SageEnvironmentClient;
};

export type {
  RuntimeAckResult,
  ReadyToStopParams,
  SetBeforeStopListenerParams,
} from './generated-types';
