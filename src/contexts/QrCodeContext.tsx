import {
  openAppSettings,
  requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import React, { createContext, useContext, useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

interface QRContextType {
  handleScan: () => Promise<void>;
  scannedValue: string | null;
}

const QRContext = createContext<QRContextType | null>(null);

export function QRProvider({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();
  const location = useLocation();
  const [scannedValue, setScannedValue] = React.useState<string | null>(null);

  useEffect(() => {
    if (location.state?.scannedUri) {
      setScannedValue(location.state.scannedUri);
      navigate(location.pathname, { replace: true, state: {} });
    }
  }, [location.state?.scannedUri, navigate, location.pathname]);

  const handleScan = async () => {
    const permissionState = await requestPermissions();
    if (permissionState === 'denied') {
      await openAppSettings();
    } else if (permissionState === 'granted') {
      navigate('/scan', {
        state: { returnTo: location.pathname },
      });
    }
  };

  return (
    <QRContext.Provider value={{ handleScan, scannedValue }}>
      {children}
    </QRContext.Provider>
  );
}

export function useQR() {
  const context = useContext(QRContext);
  if (!context) {
    throw new Error('useQR must be used within a QRProvider');
  }
  return context;
}
