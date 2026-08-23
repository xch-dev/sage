import { invoke } from '@tauri-apps/api/core';

export async function isNdefAvailable(): Promise<boolean> {
  return await invoke<{ available: boolean }>(
    'plugin:sage|is_ndef_available',
  ).then((r) => r.available);
}

export async function getNdefPayloads(): Promise<number[][]> {
  return await invoke<{ payloads: number[][] }>(
    'plugin:sage|get_ndef_payloads',
  ).then((r) => r.payloads);
}

export interface WebviewBounds {
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function setWebviewBounds(bounds: WebviewBounds): Promise<void> {
  await invoke('plugin:sage|set_webview_bounds', { request: bounds });
}

export async function snapshotWebview(
  label: string,
  width = 360,
): Promise<string> {
  return await invoke<{ dataUrl: string }>('plugin:sage|snapshot_webview', {
    request: { label, width },
  }).then((response) => response.dataUrl);
}
