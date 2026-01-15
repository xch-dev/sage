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

export async function testTangem(): Promise<string> {
  return await invoke<{ output: string }>('plugin:sage|test_tangem').then(
    (r) => r.output,
  );
}
