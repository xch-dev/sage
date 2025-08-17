import { type Theme, loadTheme } from './theme';

// Cache for loaded themes
let themesCache: Theme[] | null = null;
let themesPromise: Promise<Theme[]> | null = null;

// Dynamically discover theme folders by scanning the themes directory
async function discoverThemeFolders(): Promise<string[]> {
  try {
    // Use dynamic imports to discover available themes
    const themeModules = import.meta.glob('../themes/*/theme.json', {
      eager: false,
    });

    // Extract theme names from the module paths
    const themeNames = Object.keys(themeModules)
      .map((path) => {
        // Path format: "../themes/themeName/theme.json"
        const match = path.match(/\.\.\/themes\/([^/]+)\/theme\.json$/);
        return match ? match[1] : null;
      })
      .filter((name): name is string => name !== null);

    // Sort theme names alphabetically
    return themeNames.sort();
  } catch (error) {
    console.warn('Could not discover theme folders:', error);
    // Fallback to known themes if discovery fails
    return [];
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
    .then((themeFolders) =>
      Promise.all(themeFolders.map((themeName) => loadTheme(themeName))),
    )
    .then((themes) => {
      // Filter out null themes (themes that failed to load)
      const validThemes = themes.filter((theme): theme is Theme => theme !== null);
      themesCache = validThemes;
      return validThemes;
    })
    .catch((error) => {
      console.error('Error loading themes:', error);
      // Return a fallback theme if loading fails
      return [];
    });

  return themesPromise;
}

/**
 * Gets a theme by name from the loaded themes
 */
export async function getThemeByName(name: string): Promise<Theme | undefined> {
  const themes = await loadThemes();
  return themes.find((theme) => theme.name === name);
}

