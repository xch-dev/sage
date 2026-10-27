import { commands } from '@/bindings';
import { useEffect, useRef, useState } from 'react';
import { useTheme } from 'theme-o-rama';

// Must match the key theme-o-rama uses for the selected theme.
const THEME_STORAGE_KEY = 'theme';

/**
 * Mirrors the selected theme into the Rust config, so it survives
 * reinstalls and webview data resets that wipe localStorage.
 */
export function useThemePersistence() {
  const { currentTheme, setTheme, isLoading } = useTheme();
  const [restored, setRestored] = useState(false);
  const restoreStarted = useRef(false);
  const lastSaved = useRef<string | null>(null);

  // If localStorage lost the theme, restore it from the config once.
  useEffect(() => {
    if (isLoading || restoreStarted.current) {
      return;
    }

    restoreStarted.current = true;

    const restore = async () => {
      if (localStorage.getItem(THEME_STORAGE_KEY) !== null) {
        return;
      }

      const { theme } = await commands.getSelectedTheme({});

      if (theme) {
        lastSaved.current = theme;
        setTheme(theme);
      }
    };

    restore()
      .catch((err) => {
        console.error('Failed to restore theme from config:', err);
      })
      .finally(() => setRestored(true));
  }, [isLoading, setTheme]);

  // Save every later theme change to the config.
  useEffect(() => {
    if (!restored || !currentTheme) {
      return;
    }

    const theme = localStorage.getItem(THEME_STORAGE_KEY);

    if (theme === null || theme === lastSaved.current) {
      return;
    }

    lastSaved.current = theme;

    commands.setSelectedTheme({ theme }).catch((err) => {
      console.error('Failed to save theme to config:', err);
    });
  }, [restored, currentTheme]);
}
