import {
  getSageSystemClient,
  type SandboxStateView,
} from '@sage-system-app/sdk';

const client = await getSageSystemClient();

export type { SandboxStateView };

export async function getSandboxState(): Promise<SandboxStateView> {
  return await client.sandbox.getState();
}

export async function rerunSandboxTests(): Promise<SandboxStateView> {
  return await client.sandbox.rerunTests();
}

export function onSandboxStateChanged(
  handler: (event: SandboxStateView) => void,
): () => void {
  return client.sandbox.onStateChanged(handler);
}

export async function closeSelf(): Promise<void> {
  return await client.runtimeManager.closeSelf();
}
