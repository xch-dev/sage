import type * as Generated from './generated-types';
import type { SageSystemBridgeVersion, SageSystemClient } from './types';
import {
  createBridgeRuntimeCore,
  getSageClient,
  parseJsonOrNull,
} from '@sage-app/sdk';

export const SAGE_SYSTEM_BRIDGE_VERSION: SageSystemBridgeVersion = 'v1';

type SageSystemListenEvent<T = unknown> = {
  payload: T;
};

type SageUnlisten = () => void;

type SageWebviewHandle = {
  label: string;
  listen<T = unknown>(
    event: string,
    handler: (event: SageSystemListenEvent<T>) => void,
  ): Promise<SageUnlisten>;
};

type SageSystemWindow = Window &
  typeof globalThis & {
    __SAGE_SYSTEM__?: SageSystemClient;
  };

type SystemRuntimeEventEnvelope<T = unknown> = {
  type: string;
  payload: T;
};

type RustLikeSystemBridgeSuccessResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: true;
  result?: unknown;
  resultJson?: string;
};

type RustLikeSystemBridgeErrorResponse = {
  bridgeVersion: SageSystemBridgeVersion;
  id: string;
  ok: false;
  error: {
    code: string;
    message: string;
  };
};

type RustLikeSystemBridgeResponse =
  | RustLikeSystemBridgeSuccessResponse
  | RustLikeSystemBridgeErrorResponse;

function getSageWindow(): SageSystemWindow {
  return window as SageSystemWindow;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function isSystemRuntimeEventEnvelope(
  value: unknown,
): value is SystemRuntimeEventEnvelope {
  return (
    isObject(value) && typeof value.type === 'string' && 'payload' in value
  );
}

function dispatchSystemRuntimeEvent(data: SystemRuntimeEventEnvelope) {
  window.dispatchEvent(
    new CustomEvent<SystemRuntimeEventEnvelope>('sage-system:event', {
      detail: data,
    }),
  );

  window.dispatchEvent(
    new CustomEvent<SystemRuntimeEventEnvelope>(
      `sage-system:event:${data.type}`,
      { detail: data },
    ),
  );
}

function onSystemRuntimeEventType<T>(
  type: string,
  handler: (event: T) => void,
): () => void {
  const listener = (event: Event) => {
    const custom = event as CustomEvent<SystemRuntimeEventEnvelope<T>>;
    handler(custom.detail.payload);
  };

  window.addEventListener(
    `sage-system:event:${type}`,
    listener as EventListener,
  );

  return () => {
    window.removeEventListener(
      `sage-system:event:${type}`,
      listener as EventListener,
    );
  };
}

function bridgeResponseResult(
  data: RustLikeSystemBridgeSuccessResponse,
): unknown {
  if ('result' in data) {
    return data.result;
  }

  return parseJsonOrNull(data.resultJson);
}

export function initSageSystemRuntimeBridge(): boolean {
  const w = getSageWindow();

  if (w.__SAGE_SYSTEM__) {
    return true;
  }

  const core = createBridgeRuntimeCore({
    version: SAGE_SYSTEM_BRIDGE_VERSION,
    invokeCommand: 'apps_invoke_system_bridge',
    requestIdPrefix: 'sage-system',
  });

  if (!core) {
    return false;
  }

  const webview = core.webview as SageWebviewHandle;
  const callHost = core.callHost;

  webview
    .listen<RustLikeSystemBridgeResponse>(
      'sage-system-bridge:response',
      (event: SageSystemListenEvent<RustLikeSystemBridgeResponse>) => {
        const data = event.payload;

        if (!data || data.bridgeVersion !== SAGE_SYSTEM_BRIDGE_VERSION) {
          console.warn(
            '[Sage System SDK] Dropped malformed bridge response:',
            data,
          );
          return;
        }

        const pending = core.pendingRequests.get(data.id);
        if (!pending) {
          console.warn(
            '[Sage System SDK] Response for unknown request id:',
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
            new Error(
              data.error?.message || 'Unknown Sage system bridge error',
            ),
          );
        }
      },
    )
    .catch((error: unknown) => {
      console.error('Failed to subscribe to system bridge response:', error);
    });

  webview
    .listen('sage-system-bridge:event', (event: SageSystemListenEvent) => {
      const data = event.payload;

      if (!isSystemRuntimeEventEnvelope(data)) {
        console.warn(
          '[Sage System SDK] Dropped malformed runtime event:',
          data,
        );
        return;
      }

      try {
        dispatchSystemRuntimeEvent(data);
      } catch (error: unknown) {
        console.error(
          'Failed to dispatch Sage system bridge runtime event:',
          error,
        );
      }
    })
    .catch((error: unknown) => {
      console.error('Failed to subscribe to system bridge event:', error);
    });

  void getSageClient()
    .then((userClient) => {
      w.__SAGE_SYSTEM__ = {
        ...userClient,

        runtimeManager: {
          async listRuntimes() {
            return await callHost<Generated.SageAppRuntimeRecordView[]>(
              'runtimeManager.listRuntimes',
            );
          },

          async focusRuntime(input: Generated.RuntimeTargetParams) {
            return await callHost<Generated.SageAppRuntimeRecordView>(
              'runtimeManager.focusRuntime',
              input,
            );
          },

          async hideRuntime(input: Generated.RuntimeTargetParams) {
            return await callHost<Generated.SageAppRuntimeRecordView>(
              'runtimeManager.hideRuntime',
              input,
            );
          },

          async killRuntime(input: Generated.RuntimeTargetParams) {
            return await callHost<Generated.SystemKillRuntimeResult>(
              'runtimeManager.killRuntime',
              input,
            );
          },
          async getActiveRuntime(): Promise<Generated.SageAppRuntimeRecordView | null> {
            return await callHost<Generated.SageAppRuntimeRecordView | null>('runtimeManager.getActiveRuntime');
          },
          async hideSelf() {
            return await callHost<void>('runtimeManager.hideSelf');
          },
          async closeSelf() {
            return await callHost<void>('runtimeManager.closeSelf');
          },

          onRuntimesChanged(handler) {
            return onSystemRuntimeEventType<Generated.RuntimeManagerRuntimesChangedEvent>(
              'runtimeManager.runtimesChanged',
              handler,
            );
          },
          onActiveTaskbarRuntimeChanged(handler) {
            return onSystemRuntimeEventType<Generated.RuntimeManagerActiveTaskbarRuntimeChangedEvent>(
              'runtimeManager.activeRuntimeChanged',
              handler,
            );
          }
        },

        appInstall: {
          async previewUrl(input: Generated.AppInstallPreviewUrlParams) {
            return await callHost<Generated.SageAppUrlPreview>(
              'appInstall.previewUrl',
              input,
            );
          },

          async previewZip(input: Generated.AppInstallPreviewZipParams) {
            return await callHost<Generated.SageAppPackageManifest>(
              'appInstall.previewZip',
              input,
            );
          },

          async installUrl(input: Generated.AppInstallInstallUrlParams) {
            return await callHost<Generated.AppInstallInstallResult>(
              'appInstall.installUrl',
              input,
            );
          },

          async installZip(input: Generated.AppInstallInstallZipParams) {
            return await callHost<Generated.AppInstallInstallResult>(
              'appInstall.installZip',
              input,
            );
          },
        },

        appUpdate: {
          async getReviewContext(
            input: Generated.AppUpdateGetReviewContextParams,
          ) {
            return await callHost<Generated.AppUpdateReviewContext>(
              'appUpdate.getReviewContext',
              input,
            );
          },

          async applyUpdate(input: Generated.AppUpdateApplyUpdateParams) {
            return await callHost<Generated.AppUpdateApplyUpdateResult>(
              'appUpdate.applyUpdate',
              input,
            );
          },
        },
        capabilities: {
          async listUserDefinitions() {
            return await callHost<Generated.SageAppCapabilityDefinitionView[]>(
              'capabilities.listUserDefinitions',
            );
          },
        },

        appPermissions: {
          async getReviewContext(
            input: Generated.AppPermissionsGetReviewContextParams,
          ) {
            return await callHost<Generated.AppPermissionsReviewContext>(
              'appPermissions.getReviewContext',
              input,
            );
          },

          async applyPermissions(
            input: Generated.AppPermissionsApplyPermissionsParams,
          ) {
            return await callHost<Generated.AppPermissionsApplyPermissionsResult>(
              'appPermissions.applyPermissions',
              input,
            );
          },
        },
        fileSystem: {
          async selectFile(input: Generated.FileSystemSelectFileParams) {
            return await callHost<Generated.FileSystemSelectFileResult>(
              'fileSystem.selectFile',
              input,
            );
          },
        },
        bridgeApprovals: {
          async listPending() {
            return await callHost<Generated.PendingBridgeApprovalView[]>(
              'bridgeApprovals.listPending',
            );
          },

          async resolve(input: Generated.ResolveBridgeApprovalArgs) {
            return await callHost<void>('bridgeApprovals.resolve', input);
          },

          onChanged(handler) {
            return onSystemRuntimeEventType<Generated.BridgeApprovalsChangedEvent>(
              'bridgeApprovals.changed',
              handler,
            );
          },
        },
      };
    })
    .catch((error: unknown) => {
      console.error('Failed to initialize Sage system client:', error);
    });

  return true;
}

export async function getSageSystemClient(): Promise<SageSystemClient> {
  initSageSystemRuntimeBridge();

  const w = getSageWindow();

  if (w.__SAGE_SYSTEM__) {
    return w.__SAGE_SYSTEM__;
  }

  const started = Date.now();

  while (!w.__SAGE_SYSTEM__) {
    if (Date.now() - started > 5000) {
      throw new Error('Sage system bridge failed to initialize');
    }

    await new Promise((resolve) => window.setTimeout(resolve, 10));
  }

  return w.__SAGE_SYSTEM__;
}
