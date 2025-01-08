import { platform } from '@tauri-apps/plugin-os';
import {
  openAppSettings,
  requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { useNavigate, useLocation } from 'react-router-dom';
import { useNavigationStore } from '@/state';
import { useEffect } from 'react';

export function useScannerOrClipboard(onScanResult: (text: string) => void) {
  const navigate = useNavigate();
  const location = useLocation();
  const { returnValues, setReturnValue } = useNavigationStore();
  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    const returnValue = returnValues[location.pathname];
    if (!returnValue) return;

    if (returnValue.status === 'success' && returnValue?.data) {
      onScanResult(returnValue.data);
      setReturnValue(location.pathname, { status: 'completed' });
    }
  }, [returnValues, onScanResult, location.pathname, setReturnValue]);

  const handleScanOrPaste = async () => {
    if (isMobile) {
      const permissionState = await requestPermissions();
      if (permissionState === 'denied') {
        await openAppSettings();
      } else if (permissionState === 'granted') {
        navigate('/scan', {
          state: {
            returnTo: location.pathname,
          },
        });
      }
    } else {
      try {
        const clipboardText = await readText();
        if (clipboardText) {
          onScanResult(clipboardText);
        }
      } catch (error) {
        console.error('Failed to paste from clipboard:', error);
      }
    }
  };

  return { handleScanOrPaste, isMobile };
}
