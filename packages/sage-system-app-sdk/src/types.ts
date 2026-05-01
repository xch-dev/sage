export * from './generated-types';

import type {
  RuntimeManagerRuntimesChangedEvent,
  RuntimeTargetParams,
  SageAppRuntimeRecordView,
  SystemKillRuntimeResult,
} from './generated-types';

export type SageSystemBridgeVersion = 'v1';

export type SageSystemBridgeSuccessResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: true;
  result: unknown;
};

export type SageSystemBridgeErrorResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

export type SageSystemBridgeResponse =
  | SageSystemBridgeSuccessResponse
  | SageSystemBridgeErrorResponse;

export type SageSystemRuntimeManagerClient = {
  listRuntimes(): Promise<SageAppRuntimeRecordView[]>;
  focusRuntime(input: RuntimeTargetParams): Promise<SageAppRuntimeRecordView>;
  hideRuntime(input: RuntimeTargetParams): Promise<SageAppRuntimeRecordView>;
  killRuntime(input: RuntimeTargetParams): Promise<SystemKillRuntimeResult>;
  onRuntimesChanged(
    handler: (event: RuntimeManagerRuntimesChangedEvent) => void,
  ): () => void;
};

export type SageSystemClient = {
  runtimeManager: SageSystemRuntimeManagerClient;
};
