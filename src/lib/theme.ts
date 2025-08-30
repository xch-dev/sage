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
      try {
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
      catch (error) {
        console.warn(`Error loading theme ${themeName}:`, error);
        theme.backgroundImage = undefined;
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
  // For preview mode, apply complete isolation
  if (isPreview) {
    return applyThemeIsolated(theme, root);
  }

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
    // Core color variables
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

    // Font variables
    '--font-sans',
    '--font-serif',
    '--font-mono',
    '--font-heading',
    '--font-body',

    // Corner radius variables
    '--corner-none',
    '--corner-sm',
    '--corner-md',
    '--corner-lg',
    '--corner-xl',
    '--corner-full',

    // Shadow variables
    '--shadow-none',
    '--shadow-sm',
    '--shadow-md',
    '--shadow-lg',
    '--shadow-xl',
    '--shadow-inner',
    '--shadow-card',
    '--shadow-button',
    '--shadow-dropdown',

    // Theme feature flags
    '--theme-has-gradient-buttons',
    '--theme-has-shimmer-effects',
    '--theme-has-pixel-art',
    '--theme-has-3d-effects',
    '--theme-has-rounded-buttons',

    // Navigation and button variables
    '--outline-button-bg',
    '--nav-active-bg',

    // Background image and transparency variables
    '--background-image',
    '--background-transparent',
    '--card-transparent',
    '--popover-transparent',
    '--background-body-opacity',
    '--background-card-opacity',
    '--background-popover-opacity',

    // Table variables
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

    // Button variant variables (all possible combinations)
    '--btn-default-bg',
    '--btn-default-color',
    '--btn-default-border',
    '--btn-default-border-style',
    '--btn-default-border-width',
    '--btn-default-border-color',
    '--btn-default-radius',
    '--btn-default-shadow',
    '--btn-default-hover-bg',
    '--btn-default-hover-color',
    '--btn-default-hover-transform',
    '--btn-default-hover-border-style',
    '--btn-default-hover-border-color',
    '--btn-default-hover-shadow',
    '--btn-default-active-bg',
    '--btn-default-active-color',
    '--btn-default-active-transform',
    '--btn-default-active-border-style',
    '--btn-default-active-border-color',
    '--btn-default-active-shadow',

    '--btn-outline-bg',
    '--btn-outline-color',
    '--btn-outline-border',
    '--btn-outline-border-style',
    '--btn-outline-border-width',
    '--btn-outline-border-color',
    '--btn-outline-radius',
    '--btn-outline-shadow',
    '--btn-outline-hover-bg',
    '--btn-outline-hover-color',
    '--btn-outline-hover-transform',
    '--btn-outline-hover-border-style',
    '--btn-outline-hover-border-color',
    '--btn-outline-hover-shadow',
    '--btn-outline-active-bg',
    '--btn-outline-active-color',
    '--btn-outline-active-transform',
    '--btn-outline-active-border-style',
    '--btn-outline-active-border-color',
    '--btn-outline-active-shadow',

    '--btn-secondary-bg',
    '--btn-secondary-color',
    '--btn-secondary-border',
    '--btn-secondary-border-style',
    '--btn-secondary-border-width',
    '--btn-secondary-border-color',
    '--btn-secondary-radius',
    '--btn-secondary-shadow',
    '--btn-secondary-hover-bg',
    '--btn-secondary-hover-color',
    '--btn-secondary-hover-transform',
    '--btn-secondary-hover-border-style',
    '--btn-secondary-hover-border-color',
    '--btn-secondary-hover-shadow',
    '--btn-secondary-active-bg',
    '--btn-secondary-active-color',
    '--btn-secondary-active-transform',
    '--btn-secondary-active-border-style',
    '--btn-secondary-active-border-color',
    '--btn-secondary-active-shadow',

    '--btn-destructive-bg',
    '--btn-destructive-color',
    '--btn-destructive-border',
    '--btn-destructive-border-style',
    '--btn-destructive-border-width',
    '--btn-destructive-border-color',
    '--btn-destructive-radius',
    '--btn-destructive-shadow',
    '--btn-destructive-hover-bg',
    '--btn-destructive-hover-color',
    '--btn-destructive-hover-transform',
    '--btn-destructive-hover-border-style',
    '--btn-destructive-hover-border-color',
    '--btn-destructive-hover-shadow',
    '--btn-destructive-active-bg',
    '--btn-destructive-active-color',
    '--btn-destructive-active-transform',
    '--btn-destructive-active-border-style',
    '--btn-destructive-active-border-color',
    '--btn-destructive-active-shadow',

    '--btn-ghost-bg',
    '--btn-ghost-color',
    '--btn-ghost-border',
    '--btn-ghost-border-style',
    '--btn-ghost-border-width',
    '--btn-ghost-border-color',
    '--btn-ghost-radius',
    '--btn-ghost-shadow',
    '--btn-ghost-hover-bg',
    '--btn-ghost-hover-color',
    '--btn-ghost-hover-transform',
    '--btn-ghost-hover-border-style',
    '--btn-ghost-hover-border-color',
    '--btn-ghost-hover-shadow',
    '--btn-ghost-active-bg',
    '--btn-ghost-active-color',
    '--btn-ghost-active-transform',
    '--btn-ghost-active-border-style',
    '--btn-ghost-active-border-color',
    '--btn-ghost-active-shadow',

    '--btn-link-bg',
    '--btn-link-color',
    '--btn-link-border',
    '--btn-link-border-style',
    '--btn-link-border-width',
    '--btn-link-border-color',
    '--btn-link-radius',
    '--btn-link-shadow',
    '--btn-link-hover-bg',
    '--btn-link-hover-color',
    '--btn-link-hover-transform',
    '--btn-link-hover-border-style',
    '--btn-link-hover-border-color',
    '--btn-link-hover-shadow',
    '--btn-link-active-bg',
    '--btn-link-active-color',
    '--btn-link-active-transform',
    '--btn-link-active-border-style',
    '--btn-link-active-border-color',
    '--btn-link-active-shadow',

    // Switch variables
    '--switch-checked-bg',
    '--switch-unchecked-bg',
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

  // Apply switch-specific variables if defined
  if (theme.switches) {
    if (theme.switches.checked?.background) {
      root.style.setProperty(
        '--switch-checked-bg',
        theme.switches.checked.background,
        'important',
      );
    }
    if (theme.switches.unchecked?.background) {
      root.style.setProperty(
        '--switch-unchecked-bg',
        theme.switches.unchecked.background,
        'important',
      );
    }
  }
}

/**
 * Applies theme with complete isolation from ambient theme
 * Sets all theme values as CSS variables with fallbacks, ensuring no inheritance
 */
function applyThemeIsolated(theme: Theme, root: HTMLElement): void {
  // Set all CSS variables with explicit values, no inheritance
  const allVars: Record<string, string> = {};

  // Core colors with fallbacks
  allVars['--background'] = theme.colors?.background || 'hsl(0 0% 100%)';
  allVars['--foreground'] = theme.colors?.foreground || 'hsl(0 0% 3.9%)';
  allVars['--card'] = theme.colors?.card || theme.colors?.background || 'hsl(0 0% 98%)';
  allVars['--card-foreground'] = theme.colors?.cardForeground || theme.colors?.foreground || 'hsl(0 0% 3.9%)';
  allVars['--popover'] = theme.colors?.popover || theme.colors?.card || 'hsl(0 0% 100%)';
  allVars['--popover-foreground'] = theme.colors?.popoverForeground || theme.colors?.foreground || 'hsl(0 0% 3.9%)';
  allVars['--primary'] = theme.colors?.primary || 'hsl(0 0% 9%)';
  allVars['--primary-foreground'] = theme.colors?.primaryForeground || 'hsl(0 0% 98%)';
  allVars['--secondary'] = theme.colors?.secondary || 'hsl(0 0% 96.1%)';
  allVars['--secondary-foreground'] = theme.colors?.secondaryForeground || 'hsl(0 0% 9%)';
  allVars['--muted'] = theme.colors?.muted || theme.colors?.secondary || 'hsl(0 0% 96.1%)';
  allVars['--muted-foreground'] = theme.colors?.mutedForeground || 'hsl(0 0% 45.1%)';
  allVars['--accent'] = theme.colors?.accent || theme.colors?.secondary || 'hsl(0 0% 96.1%)';
  allVars['--accent-foreground'] = theme.colors?.accentForeground || theme.colors?.foreground || 'hsl(0 0% 9%)';
  allVars['--destructive'] = theme.colors?.destructive || 'hsl(0 84.2% 60.2%)';
  allVars['--destructive-foreground'] = theme.colors?.destructiveForeground || 'hsl(0 0% 98%)';
  allVars['--border'] = theme.colors?.border || 'hsl(0 0% 89.8%)';
  allVars['--input'] = theme.colors?.input || theme.colors?.border || 'hsl(0 0% 89.8%)';
  allVars['--input-background'] = theme.colors?.inputBackground || theme.colors?.background || 'hsl(0 0% 100%)';
  allVars['--ring'] = theme.colors?.ring || theme.colors?.primary || 'hsl(0 0% 3.9%)';

  // Chart colors
  allVars['--chart-1'] = theme.colors?.chart1 || 'hsl(12 76% 61%)';
  allVars['--chart-2'] = theme.colors?.chart2 || 'hsl(173 58% 39%)';
  allVars['--chart-3'] = theme.colors?.chart3 || 'hsl(197 37% 24%)';
  allVars['--chart-4'] = theme.colors?.chart4 || 'hsl(43 74% 66%)';
  allVars['--chart-5'] = theme.colors?.chart5 || 'hsl(27 87% 67%)';

  // Fonts
  allVars['--font-sans'] = theme.fonts?.sans || 'Inter, system-ui, sans-serif';
  allVars['--font-serif'] = theme.fonts?.serif || 'Georgia, serif';
  allVars['--font-mono'] = theme.fonts?.mono || 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace';
  allVars['--font-heading'] = theme.fonts?.heading || theme.fonts?.sans || 'Inter, system-ui, sans-serif';
  allVars['--font-body'] = theme.fonts?.body || theme.fonts?.sans || 'Inter, system-ui, sans-serif';

  // Corners
  allVars['--corner-none'] = theme.corners?.none || '0px';
  allVars['--corner-sm'] = theme.corners?.sm || '0.125rem';
  allVars['--corner-md'] = theme.corners?.md || '0.375rem';
  allVars['--corner-lg'] = theme.corners?.lg || '0.5rem';
  allVars['--corner-xl'] = theme.corners?.xl || '0.75rem';
  allVars['--corner-full'] = theme.corners?.full || '9999px';

  // Shadows
  allVars['--shadow-none'] = theme.shadows?.none || 'none';
  allVars['--shadow-sm'] = theme.shadows?.sm || '0 1px 2px 0 rgb(0 0 0 / 0.05)';
  allVars['--shadow-md'] = theme.shadows?.md || '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)';
  allVars['--shadow-lg'] = theme.shadows?.lg || '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)';
  allVars['--shadow-xl'] = theme.shadows?.xl || '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)';
  allVars['--shadow-inner'] = theme.shadows?.inner || 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)';
  allVars['--shadow-card'] = theme.shadows?.card || '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)';
  allVars['--shadow-button'] = theme.shadows?.button || '0 1px 2px 0 rgb(0 0 0 / 0.05)';
  allVars['--shadow-dropdown'] = theme.shadows?.dropdown || '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)';

  // Apply all variables
  Object.entries(allVars).forEach(([key, value]) => {
    root.style.setProperty(key, value, 'important');
  });

  // Set theme class for CSS selectors
  root.classList.add(`theme-${theme.name}`);

  // Set data attributes for theme styles
  const buttonStyles = theme.buttonStyles || [];
  root.setAttribute('data-theme-styles', buttonStyles.join(' '));

  // Apply background image if present
  if (theme.backgroundImage) {
    root.style.setProperty('--background-image', `url(${theme.backgroundImage})`, 'important');
    root.classList.add('has-background-image');

    // Apply background directly for preview
    root.style.backgroundImage = `url(${theme.backgroundImage})`;
    root.style.backgroundSize = 'cover';
    root.style.backgroundPosition = 'center';
    root.style.backgroundRepeat = 'no-repeat';
  }

  // Set the actual background color to ensure complete isolation
  root.style.backgroundColor = allVars['--card'];
  root.style.color = allVars['--card-foreground'];
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
    };
    row?: {
      background?: string;
      color?: string;
      border?: string;
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
