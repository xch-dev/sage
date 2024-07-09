import { invoke } from '@tauri-apps/api/core';

export async function generateMnemonic(use24Words: boolean): Promise<string> {
  return await invoke('generate_mnemonic', { use24Words });
}
