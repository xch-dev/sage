import { platform } from '@tauri-apps/plugin-os';
import React, { createContext, useContext, useEffect, useState } from 'react';
import { getInsets, Insets } from 'tauri-plugin-safe-area-insets';

const defaultInsets: Insets = {
  top: 0,
  bottom: 0,
  left: 0,
  right: 0,
};

const SafeAreaContext = createContext<Insets>(defaultInsets);

export function SafeAreaProvider({ children }: { children: React.ReactNode }) {
  const [insets, setInsets] = useState<Insets>(defaultInsets);

  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    async function loadInsets() {
      try {
        const newInsets = await getInsets();
        setInsets(newInsets);
      } catch (error) {
        console.error('Failed to load insets:', error);
      }
    }

    if (isMobile) {
      loadInsets();
    }
  }, [isMobile]);

  return (
    <SafeAreaContext.Provider value={insets}>
      {children}
    </SafeAreaContext.Provider>
  );
}

export function useInsets() {
  const context = useContext(SafeAreaContext);
  if (context === undefined) {
    throw new Error('useSafeArea must be used within a SafeAreaProvider');
  }
  return context;
}
