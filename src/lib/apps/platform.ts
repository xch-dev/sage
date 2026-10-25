import { platform } from '@tauri-apps/plugin-os';

const currentPlatform = platform();

export const supportsSageApps =
  currentPlatform !== 'android' && currentPlatform !== 'ios';
