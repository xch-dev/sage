import { platform } from '@tauri-apps/plugin-os';
import {
  openAppSettings,
  requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { useNavigate, useLocation } from 'react-router-dom';
import { useNavigationStore } from '@/state';
import { useCallback, useEffect } from 'react';
import { t } from '@lingui/core/macro';
import { useErrors } from '@/hooks/useErrors';
import { ScanImageError, scanQrFromImage } from '@/lib/scanImage';

export function useScannerOrClipboard(onScanResult: (text: string) => void) {
  const navigate = useNavigate();
  const location = useLocation();
  const { returnValues, setReturnValue } = useNavigationStore();
  const { addError } = useErrors();
  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    const returnValue = returnValues[location.pathname];
    if (!returnValue) return;

    if (returnValue.status === 'success' && returnValue?.data) {
      onScanResult(returnValue.data);
      setReturnValue(location.pathname, { status: 'completed' });
    }
  }, [returnValues, onScanResult, location.pathname, setReturnValue]);

  const handleScanImage = useCallback(
    async (file: File) => {
      try {
        const content = await scanQrFromImage(file);
        onScanResult(content);
      } catch (error) {
        const code = error instanceof ScanImageError ? error.code : 'unreadable';

        const reason =
          code === 'no-qr-found'
            ? t`No QR code found in that image.`
            : code === 'file-too-big'
              ? t`That image is too large to scan.`
              : t`That file could not be read as an image.`;

        addError({ kind: 'invalid', reason });
      }
    },
    [addError, onScanResult],
  );

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

  return { handleScanOrPaste, handleScanImage, isMobile };
}
