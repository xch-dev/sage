import React, { createContext, useContext, useEffect, useState } from 'react';
import { themes, type Theme, getThemeByName, applyTheme } from '@/lib/themes';

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
      
      // Update dark mode state to match theme
      const isDarkTheme = themeName === 'dark';
      localStorage.setItem('dark', isDarkTheme.toString());
      
      // Force a re-render by updating the dark mode state
      window.dispatchEvent(new CustomEvent('themeChanged', { detail: { theme: themeName } }));
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
      // If no theme is saved, check dark mode preference
      const isDarkMode = localStorage.getItem('dark') === 'true';
      const defaultTheme = isDarkMode ? getThemeByName('dark') : getThemeByName('light');
      if (defaultTheme) {
        setCurrentTheme(defaultTheme);
        applyTheme(defaultTheme);
      }
    }
  }, []);

  return (
    <ThemeContext.Provider value={{ currentTheme, setTheme, availableThemes: themes }}>
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
