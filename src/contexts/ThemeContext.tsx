import { applyTheme } from '@/lib/theme';
import { Theme } from '@/lib/theme.type';
import { getThemeByName, invalidateThemeCache, loadThemes } from '@/lib/themes';
import React, { createContext, useContext, useEffect, useState } from 'react';
import { useLocalStorage } from 'usehooks-ts';

interface ThemeContextType {
  currentTheme: Theme | null;
  setTheme: (themeName: string) => void;
  availableThemes: Theme[];
  isLoading: boolean;
  error: string | null;
  lastUsedNonCoreTheme: string | null;
  reloadThemes: () => Promise<void>;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [currentTheme, setCurrentTheme] = useState<Theme | null>(null);
  const [availableThemes, setAvailableThemes] = useState<Theme[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSettingTheme, setIsSettingTheme] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savedTheme, setSavedTheme] = useLocalStorage<string | null>(
    'theme',
    null,
  );
  const [dark] = useLocalStorage<boolean>('dark', false); // pre-themes dark mode setting
  const [lastUsedNonCoreTheme, setLastUsedNonCoreTheme] = useLocalStorage<
    string | null
  >('last-used-non-core-theme', null);

  const setTheme = async (themeName: string) => {
    if (isSettingTheme) return; // Prevent concurrent calls

    setIsSettingTheme(true);
    try {
      const theme = await getThemeByName(themeName);
      if (theme) {
        setCurrentTheme(theme);
        applyTheme(theme, document.documentElement);
        setSavedTheme(themeName);
        // Save as last used non-core theme if it's not light or dark
        if (themeName !== 'light' && themeName !== 'dark') {
          setLastUsedNonCoreTheme(themeName);
        }
        setError(null); // Clear any previous errors
      } else {
        setError(`Theme "${themeName}" not found`);
      }
    } catch (err) {
      console.error('Error setting theme:', err);
      setError('Failed to set theme');
    } finally {
      setIsSettingTheme(false);
    }
  };

  const reloadThemes = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Invalidate cache to force fresh load
      invalidateThemeCache();

      const themes = await loadThemes();
      setAvailableThemes(themes);
      if (themes.length === 0) {
        setCurrentTheme(null);
        return;
      }
      const themeToLoad = savedTheme || 'light';
      const theme = getTheme(themeToLoad, themes);
      if (theme) {
        setCurrentTheme(theme);
        applyTheme(theme, document.documentElement);
      }
    } catch (err) {
      console.error('Error reloading themes:', err);
      setError('Failed to reload themes');
      setCurrentTheme(null);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    const initializeThemes = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Load all themes
        const themes = await loadThemes();
        setAvailableThemes(themes);

        // If no themes loaded, just use CSS defaults
        if (themes.length === 0) {
          setCurrentTheme(null);
          return;
        }

        // Check for legacy dark setting and migrate if needed
        if (dark && !savedTheme) {
          setSavedTheme('dark');
        }

        // Load saved theme from localStorage or use fallback
        const themeToLoad = savedTheme || 'light';
        const theme = getTheme(themeToLoad, themes);
        if (theme) {
          setCurrentTheme(theme);
          applyTheme(theme, document.documentElement);
        }
      } catch (err) {
        console.error('Error loading themes:', err);
        setError('Failed to load themes');
        // Don't set a fallback theme - let CSS defaults handle it
        setCurrentTheme(null);
      } finally {
        setIsLoading(false);
      }
    };

    initializeThemes();
  }, [savedTheme, dark, setSavedTheme]);

  return (
    <ThemeContext.Provider
      value={{
        currentTheme,
        setTheme,
        availableThemes,
        isLoading,
        error,
        lastUsedNonCoreTheme,
        reloadThemes,
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

function getTheme(themeName: string, themes: Theme[]) {
  let theme = themes.find((t) => t.name === themeName);
  if (theme) {
    return theme;
  }
  theme = themes.find((t) => t.name === 'light');
  if (theme) {
    return theme;
  }
  if (themes.length > 0) {
    return themes[0];
  }
  return null;
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
