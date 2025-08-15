import { applyTheme, getThemeByName, themes, type Theme } from '@/lib/themes';
import React, { createContext, useContext, useEffect, useState } from 'react';

interface ThemeContextType {
  currentTheme: Theme;
  setTheme: (themeName: string) => void;
  availableThemes: Theme[];
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [currentTheme, setCurrentTheme] = useState<Theme>(themes[0]); // Default to light theme

  const setTheme = (themeName: string) => {
    const theme = getThemeByName(themeName);
    if (theme) {
      setCurrentTheme(theme);
      applyTheme(theme);
      localStorage.setItem('theme', themeName);
    }
  };

  useEffect(() => {
    // Load saved theme from localStorage
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      const theme = getThemeByName(savedTheme);
      if (theme) {
        setCurrentTheme(theme);
        applyTheme(theme);
      }
    } else {
      // Default to light theme
      const defaultTheme = getThemeByName('light');
      if (defaultTheme) {
        setCurrentTheme(defaultTheme);
        applyTheme(defaultTheme);
      }
    }
  }, []);

  return (
    <ThemeContext.Provider
      value={{ currentTheme, setTheme, availableThemes: themes }}
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
