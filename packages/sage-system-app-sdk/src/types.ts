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
  getActiveTaskbarRuntime(): Promise<Generated.SageAppRuntimeRecordView | null>;
  onRuntimesChanged(
    handler: (event: Generated.RuntimeManagerRuntimesChangedEvent) => void,
  ): () => void;
  onActiveTaskbarRuntimeChanged(
    handler: (event: Generated.RuntimeManagerActiveTaskbarRuntimeChangedEvent) => void,
  ): () => void;
  hideSelf(): Promise<void>;
  closeSelf(): Promise<void>;
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

export type SageSystemAppInstallClient = {
  previewUrl(
    input: Generated.AppInstallPreviewUrlParams,
  ): Promise<Generated.SageAppUrlPreview>;
  previewZip(
    input: Generated.AppInstallPreviewZipParams,
  ): Promise<Generated.SageAppPackageManifest>;
  installUrl(
    input: Generated.AppInstallInstallUrlParams,
  ): Promise<Generated.AppInstallInstallResult>;
  installZip(
    input: Generated.AppInstallInstallZipParams,
  ): Promise<Generated.AppInstallInstallResult>;
};

export type SageSystemAppUpdateClient = {
  getReviewContext(
    input: Generated.AppUpdateGetReviewContextParams,
  ): Promise<Generated.AppUpdateReviewContext>;
  applyUpdate(
    input: Generated.AppUpdateApplyUpdateParams,
  ): Promise<Generated.AppUpdateApplyUpdateResult>;
};

export type SageSystemFileSystemClient = {
  selectFile(
    input: Generated.FileSystemSelectFileParams,
  ): Promise<Generated.FileSystemSelectFileResult>;
};

export type SageSystemBridgeApprovalsClient = {
  listPending(): Promise<Generated.PendingBridgeApprovalView[]>;
  resolve(input: Generated.ResolveBridgeApprovalArgs): Promise<void>;
  onChanged(
    handler: (event: Generated.BridgeApprovalsChangedEvent) => void,
  ): () => void;
};

export type SageSystemDonationsClient = {
  getDetails(
    input: Generated.DonationGetDetailsParams,
  ): Promise<Generated.DonationDetails>;
};
export type SageSandboxClient = {
  getState(): Promise<Generated.SandboxStateView>;
  rerunTests(): Promise<Generated.SandboxStateView>;
  onStateChanged(
    handler: (state: Generated.SandboxStateView) => void,
  ): () => void;
};
export type SageWalletClient = {
  listWallets(): Promise<Generated.WalletListWalletsResult>;
};

export type SageSystemClient = SageClient & {
  runtimeManager: SageSystemRuntimeManagerClient;
  capabilities: SageSystemCapabilitiesClient;
  appPermissions: SageSystemAppPermissionsClient;
  appInstall: SageSystemAppInstallClient;
  appUpdate: SageSystemAppUpdateClient;
  fileSystem: SageSystemFileSystemClient;
  bridgeApprovals: SageSystemBridgeApprovalsClient;
  donations: SageSystemDonationsClient;
  sandbox: SageSandboxClient;
  wallet: SageWalletClient;
};
