import { useCallback } from 'react';
import { platform } from '@tauri-apps/plugin-os';
import {
  openAppSettings,
  requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { useNavigate } from 'react-router-dom';
import { useNavigationStore } from '@/state';

export function usePasteScanner(
  returnPath: string,
  onChange?: (value: string) => void,
) {
  const navigate = useNavigate();
  const { returnValues, setReturnValue } = useNavigationStore();
  const isMobile = platform() === 'ios' || platform() === 'android';

  const handlePasteScan = useCallback(async () => {
    if (isMobile) {
      const permissionState = await requestPermissions();
      if (permissionState === 'denied') {
        await openAppSettings();
      } else if (permissionState === 'granted') {
        navigate('/scan', {
          state: { returnTo: returnPath },
        });
      }
    } else {
      try {
        const clipboardText = await readText();
        if (clipboardText && onChange) {
          onChange(clipboardText);
        }
      } catch (error) {
        console.error('Failed to paste from clipboard:', error);
      }
    }
  }, [isMobile, navigate, returnPath, onChange]);

  // Check for scanned value
  if (
    returnValues[returnPath]?.status === 'success' &&
    returnValues[returnPath]?.data &&
    onChange
  ) {
    onChange(returnValues[returnPath].data);
    setReturnValue(returnPath, { status: 'completed' });
  }

  return {
    handlePasteScan,
    isMobile,
  };
}
