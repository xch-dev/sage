import { Theme } from 'theme-o-rama';

export function isUserTheme(theme: Theme): boolean {
  return theme.tags?.includes('user') === true;
}

export function isFeaturedTheme(theme: Theme): boolean {
  return theme.tags?.includes('featured') === true;
}

// Dynamically discover theme folders by scanning the themes directory
export async function discoverThemes(): Promise<Theme[]> {
  try {
    // Use dynamic imports to discover available themes
    const themeModules = import.meta.glob('../themes/*/theme.json', {
      eager: true,
    });

    // Extract theme JSON contents from the module paths
    const themeContents = Object.entries(themeModules)
      .map(([path, module]) => {
        // Path format: "../themes/themeName/theme.json"
        const match = path.match(/\.\.\/themes\/([^/]+)\/theme\.json$/);
        if (match) {
          return module as Theme;
        }
        return null;
      })
      .filter((theme): theme is Theme => theme !== null);

    return themeContents;
  } catch (error) {
    console.warn('Could not discover theme folders:', error);
    return [];
  }
}

export function resolveThemeImage(
  themeName: string,
  imagePath: string,
): string {
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
