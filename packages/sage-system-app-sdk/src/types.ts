import type * as Generated from './generated-types';

export * from '@sage-app/sdk';
export * from './generated-types';

import type { SageClient } from '@sage-app/sdk';

export type SageSystemBridgeVersion = 'v1';

export type SageSystemRuntimeManagerClient = {
  listRuntimes(): Promise<Generated.SageAppRuntimeRecordView[]>;
  focusRuntime(
    input: Generated.RuntimeTargetParams,
  ): Promise<Generated.SageAppRuntimeRecordView>;
  hideRuntime(
    input: Generated.RuntimeTargetParams,
  ): Promise<Generated.SageAppRuntimeRecordView>;
  killRuntime(
    input: Generated.RuntimeTargetParams,
  ): Promise<Generated.SystemKillRuntimeResult>;
  onRuntimesChanged(
    handler: (event: Generated.RuntimeManagerRuntimesChangedEvent) => void,
  ): () => void;
};

export type SageSystemCapabilitiesClient = {
  listUserDefinitions(): Promise<Generated.SageAppCapabilityDefinitionView[]>;
};

export type SageSystemAppPermissionsClient = {
  getReviewContext(
    input: Generated.AppPermissionsGetReviewContextParams,
  ): Promise<Generated.AppPermissionsReviewContext>;

  applyPermissions(
    input: Generated.AppPermissionsApplyPermissionsParams,
  ): Promise<Generated.AppPermissionsApplyPermissionsResult>;
};

export type SageSystemAppUpdateClient = {
  getReviewContext(
    input: Generated.AppUpdateGetReviewContextParams,
  ): Promise<Generated.AppUpdateReviewContext>;

  applyUpdate(
    input: Generated.AppUpdateApplyUpdateParams,
  ): Promise<Generated.AppUpdateApplyUpdateResult>;
};

export type SageSystemClient = SageClient & {
  runtimeManager: SageSystemRuntimeManagerClient;
  capabilities: SageSystemCapabilitiesClient;
  appPermissions: SageSystemAppPermissionsClient;
  appUpdate: SageSystemAppUpdateClient;
};
