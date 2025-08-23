import { commands } from '../bindings';
import { type Theme, loadBuiltInTheme, loadUserTheme } from './theme';

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
    return [];
  }
}

export async function loadThemes(): Promise<Theme[]> {
  if (themesCache) {
    return themesCache;
  }

  if (themesPromise) {
    return themesPromise;
  }

  themesPromise = discoverThemeFolders()
    .then((themeFolders) =>
      Promise.all(themeFolders.map((themeName) => loadBuiltInTheme(themeName))),
    )
    .then((themes) => {
      // Filter out null themes (themes that failed to load)
      const validThemes = themes.filter(
        (theme): theme is Theme => theme !== null,
      );
      themesCache = validThemes;
      return validThemes;
    })
    .then(async () => {
      const userThemes = await getUserThemes();
      themesCache = [...(themesCache || []), ...userThemes];
      return themesCache;
    })
    .catch((error) => {
      console.error('Error loading themes:', error);
      return [];
    });

  return themesPromise;
}

async function getUserThemes(): Promise<Theme[]> {
  const response = await commands.getUserThemes({});
  const themePromises = response.themes.map(
    async (theme) => await loadUserTheme(theme),
  );
  const themes = await Promise.all(themePromises);
  return themes.filter((theme): theme is Theme => theme !== null);
}

export async function getThemeByName(name: string): Promise<Theme | undefined> {
  const themes = await loadThemes();
  return themes.find((theme) => theme.name === name);
}
