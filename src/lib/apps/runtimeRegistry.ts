import { Webview } from '@tauri-apps/api/webview';
import {
  commands,
  type CreateInstalledRuntimeArgs,
  type SageAppRuntimeRecordView,
  type SystemSageAppView,
  type UserSageAppView,
  type SageAppView,
  type RuntimeTargetParams,
} from '@/bindings';

export type { SageAppRuntimeRecordView };

type AppLike = SageAppView | UserSageAppView | SystemSageAppView;

function runtimeTarget(appId: string): RuntimeTargetParams {
  return { appId };
}

export async function getRuntimeWebviewByLabel(
  webviewLabel: string,
): Promise<Webview | null> {
  return await Webview.getByLabel(webviewLabel).catch(() => null);
}

export async function focusRuntime(appId: string): Promise<void> {
  await commands.appsFocusRuntime(runtimeTarget(appId));
}

export async function hideRuntime(appId: string): Promise<void> {
  await commands.appsHideRuntime(runtimeTarget(appId));
}

export async function killRuntime(appId: string): Promise<void> {
  await commands.appsKillRuntime(runtimeTarget(appId));
}

export async function closeAppRuntime(
  appId: string,
  options?: { timeoutMs?: number },
): Promise<void> {
  void options;
  await killRuntime(appId);
}

export async function ensureInlineRuntime(
  app: AppLike,
): Promise<SageAppRuntimeRecordView> {
  const args: CreateInstalledRuntimeArgs = {
    appId: app.common.identity.id,
  };

  return await commands.appsCreateInlineRuntime(args);
}
