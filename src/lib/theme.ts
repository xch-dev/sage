import iconDark from '@/icon-dark.png';
import iconLight from '@/icon-light.png';
import { getColorLightness, makeColorTransparent } from './color-utils';
import { deepMerge } from './utils';

export async function loadUserTheme(themeJson: string): Promise<Theme | null> {
  try {
    let theme = JSON.parse(themeJson) as Theme;
    if (theme.inherits) {
      const inheritedTheme = await loadBuiltInTheme(
        theme.inherits,
        new Set<string>(),
      );
      if (inheritedTheme) {
        theme = deepMerge(inheritedTheme, theme);
      }
    }
    theme.isUserTheme = true;
    return theme;
  } catch (error) {
    console.error(`Error loading user theme:`, error);
    return null;
  }
}

export async function loadBuiltInTheme(
  themeName: string,
  loadedThemes: Set<string> = new Set<string>(),
): Promise<Theme | null> {
  try {
    // Check for circular inheritance
    if (loadedThemes.has(themeName)) {
      console.warn(
        `Circular theme inheritance detected: ${Array.from(loadedThemes).join(' -> ')} -> ${themeName}. Skipping inheritance.`,
      );
      return null;
    }

    loadedThemes.add(themeName);

    // Import theme as a module for hot reloading
    const themeModule = await import(`../themes/${themeName}/theme.json`);
    let theme = themeModule.default as Theme;

    // Validate that required properties are present
    if (!theme.name) {
      throw new Error(`Theme ${themeName} is missing required 'name' property`);
    }
    if (!theme.displayName) {
      throw new Error(
        `Theme ${themeName} is missing required 'displayName' property`,
      );
    }

    if (theme.inherits) {
      const inheritedTheme = await loadBuiltInTheme(
        theme.inherits,
        loadedThemes,
      );
      if (inheritedTheme) {
        theme = deepMerge(inheritedTheme, theme);
      }
    }

    // Process background image path
    if (theme.backgroundImage) {
      if (!theme.backgroundImage.startsWith('/')) {
        // Use static glob import to avoid dynamic import warnings for local files
        const imageModules = import.meta.glob(
          '../themes/*/*.{jpg,jpeg,png,gif,webp}',
          { eager: true },
        );
        const imagePath = `../themes/${themeName}/${theme.backgroundImage}`;
        const imageModule = imageModules[imagePath];

        if (imageModule) {
          theme.backgroundImage = (imageModule as { default: string }).default;
        } else {
          // Fallback to a relative path if not found
          theme.backgroundImage = `../themes/${themeName}/${theme.backgroundImage}`;
        }
      }
    }

    // only light and dark icons for now
    theme.icon_path = theme.most_like === 'dark' ? iconLight : iconDark;
    theme.isUserTheme = false;

    return theme;
  } catch (error) {
    console.error(`Error loading theme ${themeName}:`, error);
    return null;
  }
}

export function applyTheme(theme: Theme, root: HTMLElement, isPreview = false) {
  // Only manipulate classes if not a preview
  if (!isPreview) {
    // Remove any existing theme classes
    const existingThemeClasses = Array.from(root.classList).filter((cls) =>
      cls.startsWith('theme-'),
    );
    root.classList.remove(...existingThemeClasses);

    // Add theme-specific class
    root.classList.add(`theme-${theme.name}`);
  }

  // Clear all previously set CSS variables to reset to defaults
  const cssVarsToClear = [
    '--background',
    '--foreground',
    '--card',
    '--card-foreground',
    '--popover',
    '--popover-foreground',
    '--primary',
    '--primary-foreground',
    '--secondary',
    '--secondary-foreground',
    '--muted',
    '--muted-foreground',
    '--accent',
    '--accent-foreground',
    '--destructive',
    '--destructive-foreground',
    '--border',
    '--input',
    '--input-background',
    '--ring',
    '--chart-1',
    '--chart-2',
    '--chart-3',
    '--chart-4',
    '--chart-5',
    '--font-sans',
    '--font-serif',
    '--font-mono',
    '--font-heading',
    '--font-body',
    '--corner-none',
    '--corner-sm',
    '--corner-md',
    '--corner-lg',
    '--corner-xl',
    '--corner-full',
    '--shadow-none',
    '--shadow-sm',
    '--shadow-md',
    '--shadow-lg',
    '--shadow-xl',
    '--shadow-inner',
    '--shadow-card',
    '--shadow-button',
    '--shadow-dropdown',
    '--theme-has-gradient-buttons',
    '--theme-has-shimmer-effects',
    '--theme-has-pixel-art',
    '--theme-has-3d-effects',
    '--theme-has-rounded-buttons',
    '--outline-button-bg',
    '--table-background',
    '--table-border',
    '--table-border-radius',
    '--table-box-shadow',
    '--table-header-background',
    '--table-header-color',
    '--table-header-border',
    '--table-header-font-weight',
    '--table-header-font-size',
    '--table-row-background',
    '--table-row-color',
    '--table-row-border',
    '--table-row-hover-background',
    '--table-row-hover-color',
    '--table-row-selected-background',
    '--table-row-selected-color',
    '--table-cell-padding',
    '--table-cell-border',
    '--table-cell-font-size',
    '--table-footer-background',
    '--table-footer-color',
    '--table-footer-border',
    '--card-backdrop-filter',
    '--card-backdrop-filter-webkit',
    '--popover-backdrop-filter',
    '--popover-backdrop-filter-webkit',
    '--surface-backdrop-filter',
    '--surface-backdrop-filter-webkit',
    '--input-backdrop-filter',
    '--input-backdrop-filter-webkit',
    '--table-header-backdrop-filter',
    '--table-header-backdrop-filter-webkit',
    '--table-row-backdrop-filter',
    '--table-row-backdrop-filter-webkit',
    '--table-footer-backdrop-filter',
    '--table-footer-backdrop-filter-webkit',
    '--sidebar-background',
    '--sidebar-backdrop-filter',
    '--sidebar-backdrop-filter-webkit',
    '--sidebar-border',
    '--btn-default-backdrop-filter',
    '--btn-default-backdrop-filter-webkit',
    '--btn-outline-backdrop-filter',
    '--btn-outline-backdrop-filter-webkit',
    '--btn-secondary-backdrop-filter',
    '--btn-secondary-backdrop-filter-webkit',
    '--btn-destructive-backdrop-filter',
    '--btn-destructive-backdrop-filter-webkit',
    '--btn-ghost-backdrop-filter',
    '--btn-ghost-backdrop-filter-webkit',
    '--btn-link-backdrop-filter',
    '--btn-link-backdrop-filter-webkit',
  ];

  cssVarsToClear.forEach((cssVar) => {
    root.style.removeProperty(cssVar);
  });

  // Apply all color variables with !important to override CSS classes
  Object.entries(theme.colors || {}).forEach(([key, value]) => {
    if (value) {
      const cssVar = `--${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
      root.style.setProperty(cssVar, value || '', 'important');
    }
  });

  // Apply backdrop-filter variables if defined in colors object
  if (theme.colors) {
    const backdropFilterMap: Record<string, string> = {
      'cardBackdropFilter': '--card-backdrop-filter',
      'cardBackdropFilterWebkit': '--card-backdrop-filter-webkit',
      'popoverBackdropFilter': '--popover-backdrop-filter',
      'popoverBackdropFilterWebkit': '--popover-backdrop-filter-webkit',
      'surfaceBackdropFilter': '--surface-backdrop-filter',
      'surfaceBackdropFilterWebkit': '--surface-backdrop-filter-webkit',
      'inputBackdropFilter': '--input-backdrop-filter',
      'inputBackdropFilterWebkit': '--input-backdrop-filter-webkit',
    };

    Object.entries(backdropFilterMap).forEach(([themeKey, cssVar]) => {
      const value = (theme.colors as any)[themeKey];
      if (value) {
        root.style.setProperty(cssVar, value, 'important');
      }
    });
  }

  // Apply theme-specific input background if defined
  if (theme.colors?.inputBackground) {
    root.style.setProperty(
      '--input-background',
      theme.colors.inputBackground || '',
      'important',
    );
  } else if (theme.colors?.input) {
    // For other themes, use the regular input color
    root.style.setProperty(
      '--input-background',
      theme.colors.input || '',
      'important',
    );
  }
  // If neither is defined, CSS defaults will be used

  // Set dynamic outline button background based on theme
  const outlineButtonBg = getOutlineButtonBackground(theme);
  root.style.setProperty('--outline-button-bg', outlineButtonBg, 'important');

  // Set navigation active background with transparency
  if (theme.colors?.primary) {
    const navActiveBg = makeColorTransparent(theme.colors.primary, 0.1);
    root.style.setProperty('--nav-active-bg', navActiveBg, 'important');
  }

  // Set transparent versions of colors for background image support
  if (theme.backgroundImage) {
    const bodyOpacity = theme.backgroundOpacity?.body ?? 0.1;
    const cardOpacity = theme.backgroundOpacity?.card ?? 0.75;
    const popoverOpacity = theme.backgroundOpacity?.popover ?? 0.9;

    // Get the actual values from CSS variables since they might be inherited
    const backgroundValue =
      theme.colors?.background ||
      getComputedStyle(root).getPropertyValue('--background').trim();
    const cardValue =
      theme.colors?.card ||
      getComputedStyle(root).getPropertyValue('--card').trim();
    const popoverValue =
      theme.colors?.popover ||
      getComputedStyle(root).getPropertyValue('--popover').trim();

    if (backgroundValue) {
      const transparentBg = makeColorTransparent(backgroundValue, bodyOpacity);
      root.style.setProperty(
        '--background-transparent',
        transparentBg,
        'important',
      );
    }

    if (cardValue) {
      const transparentCard = makeColorTransparent(cardValue, cardOpacity);
      root.style.setProperty(
        '--card-transparent',
        transparentCard,
        'important',
      );
    }

    if (popoverValue) {
      const transparentPopover = makeColorTransparent(
        popoverValue,
        popoverOpacity,
      );
      root.style.setProperty(
        '--popover-transparent',
        transparentPopover,
        'important',
      );
    }
  }

  // Apply button-specific variables if defined
  if (theme.buttons) {
    Object.entries(theme.buttons).forEach(([variant, config]) => {
      if (config) {
        // Apply base button styles
        if (config.background) {
          root.style.setProperty(
            `--btn-${variant}-bg`,
            config.background,
            'important',
          );
        }
        if (config.color) {
          root.style.setProperty(
            `--btn-${variant}-color`,
            config.color,
            'important',
          );
        }
        if (config.border) {
          root.style.setProperty(
            `--btn-${variant}-border`,
            config.border,
            'important',
          );
        }
        if (config.borderStyle) {
          root.style.setProperty(
            `--btn-${variant}-border-style`,
            config.borderStyle,
            'important',
          );
        }
        if (config.borderWidth) {
          root.style.setProperty(
            `--btn-${variant}-border-width`,
            config.borderWidth,
            'important',
          );
        }
        if (config.borderColor) {
          root.style.setProperty(
            `--btn-${variant}-border-color`,
            config.borderColor,
            'important',
          );
        }
        if (config.borderRadius) {
          root.style.setProperty(
            `--btn-${variant}-radius`,
            config.borderRadius,
            'important',
          );
        }
        if (config.boxShadow) {
          root.style.setProperty(
            `--btn-${variant}-shadow`,
            config.boxShadow,
            'important',
          );
        }
        if (config.backdropFilter) {
          root.style.setProperty(
            `--btn-${variant}-backdrop-filter`,
            config.backdropFilter,
            'important',
          );
        }
        if (config.backdropFilterWebkit) {
          root.style.setProperty(
            `--btn-${variant}-backdrop-filter-webkit`,
            config.backdropFilterWebkit,
            'important',
          );
        }

        // Apply hover styles
        if (config.hover) {
          if (config.hover.background) {
            root.style.setProperty(
              `--btn-${variant}-hover-bg`,
              config.hover.background,
              'important',
            );
          }
          if (config.hover.color) {
            root.style.setProperty(
              `--btn-${variant}-hover-color`,
              config.hover.color,
              'important',
            );
          }
          if (config.hover.transform) {
            root.style.setProperty(
              `--btn-${variant}-hover-transform`,
              config.hover.transform,
              'important',
            );
          }
          if (config.hover.borderStyle) {
            root.style.setProperty(
              `--btn-${variant}-hover-border-style`,
              config.hover.borderStyle,
              'important',
            );
          }
          if (config.hover.borderColor) {
            root.style.setProperty(
              `--btn-${variant}-hover-border-color`,
              config.hover.borderColor,
              'important',
            );
          }
          if (config.hover.boxShadow) {
            root.style.setProperty(
              `--btn-${variant}-hover-shadow`,
              config.hover.boxShadow,
              'important',
            );
          }
        }

        // Apply active styles
        if (config.active) {
          if (config.active.background) {
            root.style.setProperty(
              `--btn-${variant}-active-bg`,
              config.active.background,
              'important',
            );
          }
          if (config.active.color) {
            root.style.setProperty(
              `--btn-${variant}-active-color`,
              config.active.color,
              'important',
            );
          }
          if (config.active.transform) {
            root.style.setProperty(
              `--btn-${variant}-active-transform`,
              config.active.transform,
              'important',
            );
          }
          if (config.active.borderStyle) {
            root.style.setProperty(
              `--btn-${variant}-active-border-style`,
              config.active.borderStyle,
              'important',
            );
          }
          if (config.active.borderColor) {
            root.style.setProperty(
              `--btn-${variant}-active-border-color`,
              config.active.borderColor,
              'important',
            );
          }
          if (config.active.boxShadow) {
            root.style.setProperty(
              `--btn-${variant}-active-shadow`,
              config.active.boxShadow,
              'important',
            );
          }
        }
      }
    });
  }

  // Apply button style flags for dynamic CSS
  const buttonStyles = theme.buttonStyles || [];

  // Set CSS variables for button style flags
  root.style.setProperty(
    '--theme-has-gradient-buttons',
    buttonStyles.includes('gradient') ? '1' : '0',
    'important',
  );
  root.style.setProperty(
    '--theme-has-shimmer-effects',
    buttonStyles.includes('shimmer') ? '1' : '0',
    'important',
  );
  root.style.setProperty(
    '--theme-has-pixel-art',
    buttonStyles.includes('pixel-art') ? '1' : '0',
    'important',
  );
  root.style.setProperty(
    '--theme-has-3d-effects',
    buttonStyles.includes('3d-effects') ? '1' : '0',
    'important',
  );
  root.style.setProperty(
    '--theme-has-rounded-buttons',
    buttonStyles.includes('rounded-buttons') ? '1' : '0',
    'important',
  );

  // Set data attribute for CSS selectors
  if (isPreview) {
    root.setAttribute('data-theme-styles', buttonStyles.join(' '));
  } else {
    document.body.setAttribute('data-theme-styles', buttonStyles.join(' '));
  }

  // Apply sidebar-specific variables if defined
  if (theme.sidebar) {
    if (theme.sidebar.background) {
      root.style.setProperty(
        '--sidebar-background',
        theme.sidebar.background,
        'important',
      );
    }
    if (theme.sidebar.backdropFilter) {
      root.style.setProperty(
        '--sidebar-backdrop-filter',
        theme.sidebar.backdropFilter,
        'important',
      );
    }
    if (theme.sidebar.backdropFilterWebkit) {
      root.style.setProperty(
        '--sidebar-backdrop-filter-webkit',
        theme.sidebar.backdropFilterWebkit,
        'important',
      );
    }
    if (theme.sidebar.border) {
      root.style.setProperty(
        '--sidebar-border',
        theme.sidebar.border,
        'important',
      );
    }
  }

  // Apply table-specific variables if defined
  if (theme.tables) {
    // Apply base table styles
    if (theme.tables.background) {
      root.style.setProperty(
        '--table-background',
        theme.tables.background,
        'important',
      );
    }
    if (theme.tables.border) {
      root.style.setProperty(
        '--table-border',
        theme.tables.border,
        'important',
      );
    }
    if (theme.tables.borderRadius) {
      root.style.setProperty(
        '--table-border-radius',
        theme.tables.borderRadius,
        'important',
      );
    }
    if (theme.tables.boxShadow) {
      root.style.setProperty(
        '--table-box-shadow',
        theme.tables.boxShadow,
        'important',
      );
    }

    // Apply header styles
    if (theme.tables.header) {
      if (theme.tables.header.background) {
        root.style.setProperty(
          '--table-header-background',
          theme.tables.header.background,
          'important',
        );
      }
      if (theme.tables.header.color) {
        root.style.setProperty(
          '--table-header-color',
          theme.tables.header.color,
          'important',
        );
      }
      if (theme.tables.header.border) {
        root.style.setProperty(
          '--table-header-border',
          theme.tables.header.border,
          'important',
        );
      }
      if (theme.tables.header.fontWeight) {
        root.style.setProperty(
          '--table-header-font-weight',
          theme.tables.header.fontWeight,
          'important',
        );
      }
      if (theme.tables.header.fontSize) {
        root.style.setProperty(
          '--table-header-font-size',
          theme.tables.header.fontSize,
          'important',
        );
      }
    }

    // Apply row styles
    if (theme.tables.row) {
      if (theme.tables.row.background) {
        root.style.setProperty(
          '--table-row-background',
          theme.tables.row.background,
          'important',
        );
      }
      if (theme.tables.row.color) {
        root.style.setProperty(
          '--table-row-color',
          theme.tables.row.color,
          'important',
        );
      }
      if (theme.tables.row.border) {
        root.style.setProperty(
          '--table-row-border',
          theme.tables.row.border,
          'important',
        );
      }
      if (theme.tables.row.hover) {
        if (theme.tables.row.hover.background) {
          root.style.setProperty(
            '--table-row-hover-background',
            theme.tables.row.hover.background,
            'important',
          );
        }
        if (theme.tables.row.hover.color) {
          root.style.setProperty(
            '--table-row-hover-color',
            theme.tables.row.hover.color,
            'important',
          );
        }
      }
      if (theme.tables.row.selected) {
        if (theme.tables.row.selected.background) {
          root.style.setProperty(
            '--table-row-selected-background',
            theme.tables.row.selected.background,
            'important',
          );
        }
        if (theme.tables.row.selected.color) {
          root.style.setProperty(
            '--table-row-selected-color',
            theme.tables.row.selected.color,
            'important',
          );
        }
      }
      if (theme.tables.row.backdropFilter) {
        root.style.setProperty(
          '--table-row-backdrop-filter',
          theme.tables.row.backdropFilter,
          'important',
        );
      }
      if (theme.tables.row.backdropFilterWebkit) {
        root.style.setProperty(
          '--table-row-backdrop-filter-webkit',
          theme.tables.row.backdropFilterWebkit,
          'important',
        );
      }
    }

    // Apply cell styles
    if (theme.tables.cell) {
      if (theme.tables.cell.padding) {
        root.style.setProperty(
          '--table-cell-padding',
          theme.tables.cell.padding,
          'important',
        );
      }
      if (theme.tables.cell.border) {
        root.style.setProperty(
          '--table-cell-border',
          theme.tables.cell.border,
          'important',
        );
      }
      if (theme.tables.cell.fontSize) {
        root.style.setProperty(
          '--table-cell-font-size',
          theme.tables.cell.fontSize,
          'important',
        );
      }
    }

    // Apply footer styles
    if (theme.tables.footer) {
      if (theme.tables.footer.background) {
        root.style.setProperty(
          '--table-footer-background',
          theme.tables.footer.background,
          'important',
        );
      }
      if (theme.tables.footer.color) {
        root.style.setProperty(
          '--table-footer-color',
          theme.tables.footer.color,
          'important',
        );
      }
      if (theme.tables.footer.border) {
        root.style.setProperty(
          '--table-footer-border',
          theme.tables.footer.border,
          'important',
        );
      }
      if (theme.tables.footer.backdropFilter) {
        root.style.setProperty(
          '--table-footer-backdrop-filter',
          theme.tables.footer.backdropFilter,
          'important',
        );
      }
      if (theme.tables.footer.backdropFilterWebkit) {
        root.style.setProperty(
          '--table-footer-backdrop-filter-webkit',
          theme.tables.footer.backdropFilterWebkit,
          'important',
        );
      }
    }

    // Apply header backdrop filters
    if (theme.tables.header) {
      if (theme.tables.header.backdropFilter) {
        root.style.setProperty(
          '--table-header-backdrop-filter',
          theme.tables.header.backdropFilter,
          'important',
        );
      }
      if (theme.tables.header.backdropFilterWebkit) {
        root.style.setProperty(
          '--table-header-backdrop-filter-webkit',
          theme.tables.header.backdropFilterWebkit,
          'important',
        );
      }
    }
  }

  // Apply font variables
  Object.entries(theme.fonts || {}).forEach(([key, value]) => {
    if (value) {
      const cssVar = `--font-${key}`;
      root.style.setProperty(cssVar, value, 'important');
    }
  });

  // Apply corner variables
  Object.entries(theme.corners || {}).forEach(([key, value]) => {
    if (value) {
      const cssVar = `--corner-${key}`;
      root.style.setProperty(cssVar, value, 'important');
    }
  });

  // Apply shadow variables
  Object.entries(theme.shadows || {}).forEach(([key, value]) => {
    if (value) {
      const cssVar = `--shadow-${key}`;
      root.style.setProperty(cssVar, value, 'important');
    }
  });

  // Apply background image if present
  if (theme.backgroundImage) {
    root.style.setProperty(
      '--background-image',
      `url(${theme.backgroundImage})`,
      'important',
    );

    // For document-wide themes, add class to body
    // For preview themes, add class to the element and apply background
    if (isPreview) {
      root.classList.add('has-background-image');
      // Apply background image directly to preview element
      root.style.backgroundImage = `url(${theme.backgroundImage})`;
      root.style.backgroundSize = 'cover';
      root.style.backgroundPosition = 'center';
    } else {
      document.body.classList.add('has-background-image');
    }

    // Set background opacity variables
    const bodyOpacity = theme.backgroundOpacity?.body ?? 0.1;
    const cardOpacity = theme.backgroundOpacity?.card ?? 0.75;
    const popoverOpacity = theme.backgroundOpacity?.popover ?? 0.9;

    root.style.setProperty(
      '--background-body-opacity',
      bodyOpacity.toString(),
      'important',
    );
    root.style.setProperty(
      '--background-card-opacity',
      cardOpacity.toString(),
      'important',
    );
    root.style.setProperty(
      '--background-popover-opacity',
      popoverOpacity.toString(),
      'important',
    );
  } else {
    root.style.removeProperty('--background-image');
    if (isPreview) {
      root.classList.remove('has-background-image');
      root.style.removeProperty('background-image');
      root.style.removeProperty('background-size');
      root.style.removeProperty('background-position');
    } else {
      document.body.classList.remove('has-background-image');
    }
  }
}

/**
 * Extracts theme properties into a styles object for component use
 * This is a subset of applyTheme logic focused on component styling
 */
export function getThemeStyles(theme: Theme): Record<string, string> {
  const styles: Record<string, string> = {};

  // Apply background color - use card color, or background color as fallback
  if (theme.colors?.card) {
    styles.backgroundColor = theme.colors.card;
  } else if (theme.colors?.background) {
    styles.backgroundColor = theme.colors.background;
  }

  // Apply text color - use card foreground, or foreground as fallback
  if (theme.colors?.cardForeground) {
    styles.color = theme.colors.cardForeground;
  } else if (theme.colors?.foreground) {
    styles.color = theme.colors.foreground;
  }

  // Apply border
  if (theme.colors?.border) {
    styles.border = `1px solid ${theme.colors.border}`;
  }

  // Apply border radius
  if (theme.corners?.lg) {
    styles.borderRadius = theme.corners.lg;
  }

  // Apply box shadow
  if (theme.shadows?.card) {
    styles.boxShadow = theme.shadows.card;
  }

  // Apply font family
  if (theme.fonts?.body) {
    styles.fontFamily = theme.fonts.body;
  }

  // Apply background image
  if (theme.backgroundImage) {
    styles.backgroundImage = `url(${theme.backgroundImage})`;
    styles.backgroundSize = 'cover';
    styles.backgroundPosition = 'center';
    styles.backgroundRepeat = 'no-repeat';
  }

  return styles;
}

/**
 * Gets button styles for a specific variant from a theme
 */
export function getButtonStyles(
  theme: Theme,
  variant:
    | 'default'
    | 'outline'
    | 'secondary'
    | 'destructive'
    | 'ghost'
    | 'link' = 'default',
): Record<string, string> {
  const styles: Record<string, string> = {};
  const buttonConfig = theme.buttons?.[variant];

  if (buttonConfig) {
    if (buttonConfig.background) {
      styles.backgroundColor = buttonConfig.background;
    }
    if (buttonConfig.color) {
      styles.color = buttonConfig.color;
    }
    if (buttonConfig.border) {
      styles.border = buttonConfig.border;
    }
    if (buttonConfig.borderRadius) {
      styles.borderRadius = buttonConfig.borderRadius;
    }
    if (buttonConfig.boxShadow) {
      styles.boxShadow = buttonConfig.boxShadow;
    }
  } else {
    // Fallback to theme colors if no specific button config
    if (theme.colors?.primary) {
      styles.backgroundColor = theme.colors.primary;
    }
    if (theme.colors?.primaryForeground) {
      styles.color = theme.colors.primaryForeground;
    }
    if (theme.colors?.border) {
      styles.border = `1px solid ${theme.colors.border}`;
    }
    if (theme.corners?.md) {
      styles.borderRadius = theme.corners.md;
    }
    if (theme.shadows?.button) {
      styles.boxShadow = theme.shadows.button;
    }
  }

  return styles;
}

/**
 * Gets heading styles from a theme
 */
export function getHeadingStyles(theme: Theme): Record<string, string> {
  const styles: Record<string, string> = {};

  if (theme.fonts?.heading) {
    styles.fontFamily = theme.fonts.heading;
  }

  return styles;
}

export function getMutedTextStyles(theme: Theme): Record<string, string> {
  const styles: Record<string, string> = {};

  if (theme.colors?.mutedForeground) {
    styles.color = theme.colors.mutedForeground;
  }
  if (theme.fonts?.body) {
    styles.fontFamily = theme.fonts.body;
  }

  return styles;
}

export function getTextStyles(theme: Theme): Record<string, string> {
  const styles: Record<string, string> = {};

  if (theme.colors?.secondary) {
    styles.color = theme.colors.secondary;
  }
  if (theme.fonts?.body) {
    styles.fontFamily = theme.fonts.body;
  }

  return styles;
}

function getOutlineButtonBackground(theme: Theme): string {
  // If theme has no colors defined, use CSS defaults (light theme)
  if (!theme.colors?.background) {
    return 'transparent';
  }

  // Get background lightness using our color utility
  const lightness = getColorLightness(theme.colors.background);

  // If background is very dark (< 20% lightness), use card color for subtle background
  // If background is light (> 50% lightness), use transparent
  if (lightness < 20) {
    return theme.colors.card ? theme.colors.card || '' : 'transparent';
  } else if (lightness > 50) {
    return 'transparent';
  } else {
    // For mid-range themes, use a slightly lighter version of the background
    return theme.colors.secondary
      ? theme.colors.secondary || ''
      : 'transparent';
  }
}

export interface Theme {
  name: string;
  displayName: string;
  most_like?: 'light' | 'dark';
  icon_path?: string;
  backgroundImage?: string;
  isUserTheme?: boolean;
  backgroundOpacity?: {
    body?: number;
    card?: number;
    popover?: number;
  };
  inherits?: string;
  colors?: {
    background?: string;
    foreground?: string;
    card?: string;
    cardForeground?: string;
    popover?: string;
    popoverForeground?: string;
    primary?: string;
    primaryForeground?: string;
    secondary?: string;
    secondaryForeground?: string;
    muted?: string;
    mutedForeground?: string;
    accent?: string;
    accentForeground?: string;
    destructive?: string;
    destructiveForeground?: string;
    border?: string;
    input?: string;
    inputBackground?: string;
    ring?: string;
    chart1?: string;
    chart2?: string;
    chart3?: string;
    chart4?: string;
    chart5?: string;
  };
  fonts?: {
    sans?: string;
    serif?: string;
    mono?: string;
    heading?: string;
    body?: string;
  };
  corners?: {
    none?: string;
    sm?: string;
    md?: string;
    lg?: string;
    xl?: string;
    full?: string;
  };
  shadows?: {
    none?: string;
    sm?: string;
    md?: string;
    lg?: string;
    xl?: string;
    inner?: string;
    card?: string;
    button?: string;
    dropdown?: string;
  };
  // Optional theme-specific sidebar configuration
  sidebar?: {
    background?: string;
    backdropFilter?: string;
    backdropFilterWebkit?: string;
    border?: string;
  };
  // Optional theme-specific table configurations
  tables?: {
    background?: string;
    border?: string;
    borderRadius?: string;
    boxShadow?: string;
    header?: {
      background?: string;
      color?: string;
      border?: string;
      fontWeight?: string;
      fontSize?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
    };
    row?: {
      background?: string;
      color?: string;
      border?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
      };
      selected?: {
        background?: string;
        color?: string;
      };
    };
    cell?: {
      padding?: string;
      border?: string;
      fontSize?: string;
    };
    footer?: {
      background?: string;
      color?: string;
      border?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
    };
  };
  // Optional theme-specific button configurations
  buttons?: {
    default?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    outline?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    secondary?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    destructive?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    ghost?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    link?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
  };
  // Button style flags for dynamic CSS application
  buttonStyles?: string[];
  // Optional theme-specific switch configurations
  switches?: {
    checked?: {
      background?: string;
    };
    unchecked?: {
      background?: string;
    };
  };
}
