export * from '@sage-app/sdk';
export * from './generated-types';

import type { SageClient } from '@sage-app/sdk';
import type {
  RuntimeManagerRuntimesChangedEvent,
  RuntimeTargetParams,
  SageAppRuntimeRecordView,
  SystemKillRuntimeResult,
} from './generated-types';

export type SageSystemBridgeVersion = 'v1';

export type SageSystemRuntimeManagerClient = {
  listRuntimes(): Promise<SageAppRuntimeRecordView[]>;
  focusRuntime(input: RuntimeTargetParams): Promise<SageAppRuntimeRecordView>;
  hideRuntime(input: RuntimeTargetParams): Promise<SageAppRuntimeRecordView>;
  killRuntime(input: RuntimeTargetParams): Promise<SystemKillRuntimeResult>;
  onRuntimesChanged(
    handler: (event: RuntimeManagerRuntimesChangedEvent) => void,
  ): () => void;
};

export type SageSystemClient = SageClient & {
  runtimeManager: SageSystemRuntimeManagerClient;
};
