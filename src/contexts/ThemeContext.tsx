import {
  applyTheme,
  getThemeByName,
  getThemeByNameSync,
  loadThemes,
  type Theme,
} from '@/lib/themes';
import React, { createContext, useContext, useEffect, useState } from 'react';

interface ThemeContextType {
  currentTheme: Theme | null;
  setTheme: (themeName: string) => void;
  availableThemes: Theme[];
  isLoading: boolean;
  error: string | null;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [currentTheme, setCurrentTheme] = useState<Theme | null>(null);
  const [availableThemes, setAvailableThemes] = useState<Theme[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const setTheme = async (themeName: string) => {
    try {
      const theme = await getThemeByName(themeName);
      if (theme) {
        setCurrentTheme(theme);
        applyTheme(theme);
        localStorage.setItem('theme', themeName);
      }
    } catch (err) {
      console.error('Error setting theme:', err);
      setError('Failed to set theme');
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

        // Load saved theme from localStorage
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme) {
          const theme = themes.find((t) => t.name === savedTheme);
          if (theme) {
            setCurrentTheme(theme);
            applyTheme(theme);
          } else {
            // Fallback to first theme if saved theme not found
            setCurrentTheme(themes[0]);
            applyTheme(themes[0]);
          }
        } else {
          // Default to light theme
          const lightTheme = themes.find((t) => t.name === 'light');
          if (lightTheme) {
            setCurrentTheme(lightTheme);
            applyTheme(lightTheme);
          } else {
            // Fallback to first theme
            setCurrentTheme(themes[0]);
            applyTheme(themes[0]);
          }
        }
      } catch (err) {
        console.error('Error loading themes:', err);
        setError('Failed to load themes');

        // Try to use fallback theme
        const fallbackTheme = getThemeByNameSync('light');
        if (fallbackTheme) {
          setCurrentTheme(fallbackTheme);
          applyTheme(fallbackTheme);
        }
      } finally {
        setIsLoading(false);
      }
    };

    initializeThemes();
  }, []);

  return (
    <ThemeContext.Provider
      value={{ currentTheme, setTheme, availableThemes, isLoading, error }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
