import { invoke } from '@tauri-apps/api/core';

export interface Insets {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

export async function getInsets(): Promise<Insets> {
  return await invoke('plugin:safe-area-insets|get_insets');
}
