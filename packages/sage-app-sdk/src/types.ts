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

export type SageBridgeResponse =
  | SageBridgeSuccessResponse
  | SageBridgeErrorResponse;

export type SageBridgeRuntimeEvent =
  | Generated.GrantedCapabilitiesChangeEvent
  | Generated.GrantedNetworkWhitelistChangeEvent;

export type SageWalletClient = {
  getKeys(): Promise<Generated.GetKeysResponse>;
  getKey(input: Generated.GetKey): Promise<Generated.GetKeyResponse>;
  getSecretKey(
    input: Generated.GetSecretKey,
  ): Promise<Generated.GetSecretKeyResponse>;

  getSyncStatus(): Promise<Generated.GetSyncStatusResponse>;
  getVersion(): Promise<Generated.GetVersionResponse>;
  getPendingTransactions(): Promise<Generated.GetPendingTransactionsResponse>;

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
    handler: (
      detail: Omit<Generated.SageLifecycleBeforeStopDetail, 'requestId'>,
    ) => void | Promise<void>,
  ): () => void;
};

export type SageAppClient = {
  bridgePing(): Promise<Generated.BridgePingResult>;
  bridgeSend(input: SageBridgeSendPayload): Promise<Generated.BridgeSendResult>;
  getInfo(): Promise<Generated.AppGetInfoResult>;
  getCapabilities(): Promise<string[]>;
  requestCapabilityGrant(
    input: Generated.RequestCapabilityGrantParams,
  ): Promise<Generated.RequestCapabilityGrantResult>;
  requestNetworkWhitelistGrant(
    input: Generated.RequestNetworkWhitelistGrantParams,
  ): Promise<Generated.RequestNetworkWhitelistGrantResult>;
  onGrantedCapabilitiesChange(
    handler: (event: Generated.GrantedCapabilitiesChangeEvent) => void,
  ): () => void;
  onGrantedNetworkWhitelistChange(
    handler: (event: Generated.GrantedNetworkWhitelistChangeEvent) => void,
  ): () => void;
  lifecycle: SageAppLifecycleClient;
};

export type SageClient = {
  initialAppInfo: Generated.AppGetInfoResult;
  app: SageAppClient;
  wallet: SageWalletClient;
};

export type {
  RuntimeAckResult,
  ReadyToStopParams,
  SetBeforeStopListenerParams,
  SageLifecycleBeforeStopDetail,
} from './generated-types';
