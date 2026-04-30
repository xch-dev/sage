import {
  getSageSystemClient,
  type RuntimeManagerRuntimesChangedEvent,
  type RuntimeTargetParams,
  type SageAppRuntimeRecordView,
  type SystemKillRuntimeResult,
} from '@sage-system-app/sdk';

const client = await getSageSystemClient();

export type { SageAppRuntimeRecordView as RuntimeRecord };

export function onRuntimesChanged(
  handler: (event: RuntimeManagerRuntimesChangedEvent) => void,
): () => void {
  return client.runtimeManager.onRuntimesChanged(handler);
}

export async function listRuntimes(): Promise<SageAppRuntimeRecordView[]> {
  return await client.runtimeManager.listRuntimes();
}

export async function focusRuntime(appId: string): Promise<SageAppRuntimeRecordView> {
  return await client.runtimeManager.focusRuntime({
    appId,
  } satisfies RuntimeTargetParams);
}

export async function hideRuntime(appId: string): Promise<SageAppRuntimeRecordView> {
  return await client.runtimeManager.hideRuntime({
    appId,
  } satisfies RuntimeTargetParams);
}

export async function killRuntime(
  appId: string,
): Promise<SystemKillRuntimeResult> {
  return await client.runtimeManager.killRuntime({
    appId,
  } satisfies RuntimeTargetParams);
}
