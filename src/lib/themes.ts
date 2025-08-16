export interface Theme {
  name: string;
  displayName: string;
  backgroundImage?: string;
  colors: {
    background: string;
    foreground: string;
    card: string;
    cardForeground: string;
    popover: string;
    popoverForeground: string;
    primary: string;
    primaryForeground: string;
    secondary: string;
    secondaryForeground: string;
    muted: string;
    mutedForeground: string;
    accent: string;
    accentForeground: string;
    destructive: string;
    destructiveForeground: string;
    border: string;
    input: string;
    ring: string;
    chart1: string;
    chart2: string;
    chart3: string;
    chart4: string;
    chart5: string;
  };
  fonts: {
    sans: string;
    serif: string;
    mono: string;
    heading: string;
    body: string;
  };
  corners: {
    none: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
    full: string;
  };
  shadows: {
    none: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
    inner: string;
    card: string;
    button: string;
    dropdown: string;
  };
}

// Cache for loaded themes
let themesCache: Theme[] | null = null;
let themesPromise: Promise<Theme[]> | null = null;

// Dynamically discover theme folders by scanning the themes directory
async function discoverThemeFolders(): Promise<string[]> {
  try {
    // Try to get directory listing (this works in some server configurations)
    const response = await fetch('/themes/');
    if (response.ok) {
      const text = await response.text();
      // Parse HTML directory listing to extract folder names
      const folderMatches = text.match(/href="([^"]+)\/"/g);
      if (folderMatches) {
        const folders = folderMatches
          .map(match => match.replace('href="', '').replace('/"', ''))
          .filter(folder => folder !== '..' && folder !== '.');
        
        // Filter folders that actually contain a theme.json file
        const validThemes = await Promise.all(
          folders.map(async (folder) => {
            try {
              const themeResponse = await fetch(`/themes/${folder}/theme.json`);
              return themeResponse.ok ? folder : null;
            } catch {
              return null;
            }
          })
        );
        
        return validThemes.filter((theme): theme is string => theme !== null);
      }
    }
  } catch {
    console.warn('Could not scan themes directory');
  }

  // Fallback to known themes if scanning fails
  return ['light', 'dark', 'win95', 'colorful', 'amiga', 'macintosh'];
}

/**
 * Loads a single theme from its JSON file
 */
async function loadTheme(themeName: string): Promise<Theme> {
  try {
    const response = await fetch(`/themes/${themeName}/theme.json`);
    if (!response.ok) {
      throw new Error(
        `Failed to load theme ${themeName}: ${response.statusText}`,
      );
    }

    const theme = (await response.json()) as Theme;

    // Process background image path to be relative to the theme folder
    if (theme.backgroundImage && !theme.backgroundImage.startsWith('/')) {
      theme.backgroundImage = `/themes/${themeName}/${theme.backgroundImage}`;
    }

    return theme;
  } catch (error) {
    console.error(`Error loading theme ${themeName}:`, error);
    throw error;
  }
}

/**
 * Loads all themes from the public/themes folder
 */
export async function loadThemes(): Promise<Theme[]> {
  if (themesCache) {
    return themesCache;
  }

  if (themesPromise) {
    return themesPromise;
  }

  themesPromise = discoverThemeFolders()
    .then((themeFolders) => Promise.all(
      themeFolders.map((themeName) => loadTheme(themeName)),
    ))
    .then((themes) => {
      themesCache = themes;
      return themes;
    })
    .catch((error) => {
      console.error('Error loading themes:', error);
      // Return a fallback theme if loading fails
      return [getFallbackTheme()];
    });

  return themesPromise;
}

/**
 * Provides a fallback theme in case loading fails
 */
function getFallbackTheme(): Theme {
  return {
    name: 'light',
    displayName: 'Light',
    colors: {
      background: '0 0% 100%',
      foreground: '0 0% 3.9%',
      card: '0 0% 98%',
      cardForeground: '0 0% 3.9%',
      popover: '0 0% 100%',
      popoverForeground: '0 0% 3.9%',
      primary: '0 0% 9%',
      primaryForeground: '0 0% 98%',
      secondary: '0 0% 96.1%',
      secondaryForeground: '0 0% 9%',
      muted: '0 0% 96.1%',
      mutedForeground: '0 0% 45.1%',
      accent: '0 0% 96.1%',
      accentForeground: '0 0% 9%',
      destructive: '0 84.2% 60.2%',
      destructiveForeground: '0 0% 98%',
      border: '0 0% 89.8%',
      input: '0 0% 96.1%',
      ring: '0 0% 3.9%',
      chart1: '12 76% 61%',
      chart2: '173 58% 39%',
      chart3: '197 37% 24%',
      chart4: '43 74% 66%',
      chart5: '27 87% 67%',
    },
    fonts: {
      sans: 'Inter, system-ui, sans-serif',
      serif: 'Georgia, serif',
      mono: 'Courier New, Monaco, Consolas, monospace',
      heading: 'Inter, system-ui, sans-serif',
      body: 'Inter, system-ui, sans-serif',
    },
    corners: {
      none: '0px',
      sm: '0.125rem',
      md: '0.375rem',
      lg: '0.5rem',
      xl: '0.75rem',
      full: '9999px',
    },
    shadows: {
      none: 'none',
      sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
      md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
      lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
      xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
      inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
      card: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
      button: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
      dropdown:
        '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
    },
  };
}

/**
 * Gets a theme by name from the loaded themes
 */
export async function getThemeByName(name: string): Promise<Theme | undefined> {
  const themes = await loadThemes();
  return themes.find((theme) => theme.name === name);
}

/**
 * Synchronous version of getThemeByName for backward compatibility
 * Note: This will return undefined if themes haven't been loaded yet
 */
export function getThemeByNameSync(name: string): Theme | undefined {
  if (!themesCache) {
    return undefined;
  }
  return themesCache.find((theme) => theme.name === name);
}

export function applyTheme(theme: Theme) {
  const root = document.documentElement;

  // Apply all color variables with !important to override CSS classes
  Object.entries(theme.colors).forEach(([key, value]) => {
    const cssVar = `--${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
    root.style.setProperty(cssVar, value, 'important');
  });

  // Apply font variables
  Object.entries(theme.fonts).forEach(([key, value]) => {
    const cssVar = `--font-${key}`;
    root.style.setProperty(cssVar, value, 'important');
  });

  // Apply corner variables
  Object.entries(theme.corners).forEach(([key, value]) => {
    const cssVar = `--corner-${key}`;
    root.style.setProperty(cssVar, value, 'important');
  });

  // Apply shadow variables
  Object.entries(theme.shadows).forEach(([key, value]) => {
    const cssVar = `--shadow-${key}`;
    root.style.setProperty(cssVar, value, 'important');
  });

  // Apply background image if present
  if (theme.backgroundImage) {
    root.style.setProperty(
      '--background-image',
      `url(${theme.backgroundImage})`,
      'important',
    );
    document.body.classList.add('has-background-image');
  } else {
    root.style.removeProperty('--background-image');
    document.body.classList.remove('has-background-image');
  }
}
