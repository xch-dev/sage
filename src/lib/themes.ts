import { Theme } from 'theme-o-rama';
import { commands } from '../bindings';

export function hasTag(theme: Theme, tag: string): boolean {
  return theme.tags?.includes(tag) === true;
}

// Dynamically discover theme folders by scanning the themes directory
export async function discoverThemes(): Promise<Theme[]> {
  try {
    // Use dynamic imports to discover available themes
    const themeModules = import.meta.glob('../themes/*/theme.json', {
      eager: true,
    });

    // Extract theme JSON contents from the module paths
    const appThemes = Object.entries(themeModules)
      .map(([path, module]) => {
        // Path format: "../themes/themeName/theme.json"
        const match = path.match(/\.\.\/themes\/([^/]+)\/theme\.json$/);
        if (match) {
          return module as Theme;
        }
        return null;
      })
      .filter((theme): theme is Theme => theme !== null);

    const userThemes = await getUserThemes();

    return [...appThemes, ...userThemes];
  } catch (error) {
    console.warn('Could not discover theme folders:', error);
    return [];
  }
}

export async function resolveThemeImage(
  themeName: string,
  imagePath: string,
): Promise<string> {
  // Check for sentinel value to return uploaded background image
  if (imagePath === '{NEED_DATA_URL_BACKGROUND_IMAGE}') {
    return localStorage.getItem('background-image') ?? '';
  }

  // Use static glob import to avoid dynamic import warnings for local files
  const imageModules = import.meta.glob(
    '../themes/*/*.{jpg,jpeg,png,gif,webp}',
    { eager: true },
  );
  const resolvedPath = `../themes/${themeName}/${imagePath}`;
  const imageModule = imageModules[resolvedPath];

  if (imageModule) {
    return (imageModule as { default: string }).default;
  }

  return `../themes/${themeName}/${imagePath}`;
}

async function getUserThemes(): Promise<Theme[]> {
  const response = await commands.getUserThemes({});
  return response.themes
    .map((theme) => {
      try {
        const t = JSON.parse(theme) as Theme;
        t.tags = ['user'];
        return t;
      } catch {
        return null;
      }
    })
    .filter((theme): theme is Theme => theme !== null);
}
