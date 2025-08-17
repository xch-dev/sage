export async function loadTheme(themeName: string): Promise<Theme> {
  try {
    // Import theme as a module for hot reloading
    const themeModule = await import(`../themes/${themeName}/theme.json`);
    const theme = themeModule.default as Theme;

    // Process background image path to be relative to the theme folder
    if (theme.backgroundImage && !theme.backgroundImage.startsWith('/')) {
      // Use static glob import to avoid dynamic import warnings
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

    return theme;
  } catch (error) {
    console.error(`Error loading theme ${themeName}:`, error);
    throw error;
  }
}

export function applyTheme(theme: Theme) {
  const root = document.documentElement;

  // Remove any existing theme classes
  const existingThemeClasses = Array.from(document.body.classList).filter(
    (cls) => cls.startsWith('theme-'),
  );
  document.body.classList.remove(...existingThemeClasses);

  // Add theme-specific class
  document.body.classList.add(`theme-${theme.name}`);

  // Clear all previously set CSS variables to reset to defaults
  const cssVarsToClear = [
    '--background', '--foreground', '--card', '--card-foreground',
    '--popover', '--popover-foreground', '--primary', '--primary-foreground',
    '--secondary', '--secondary-foreground', '--muted', '--muted-foreground',
    '--accent', '--accent-foreground', '--destructive', '--destructive-foreground',
    '--border', '--input', '--input-background', '--ring',
    '--chart-1', '--chart-2', '--chart-3', '--chart-4', '--chart-5',
    '--font-sans', '--font-serif', '--font-mono', '--font-heading', '--font-body',
    '--corner-none', '--corner-sm', '--corner-md', '--corner-lg', '--corner-xl', '--corner-full',
    '--shadow-none', '--shadow-sm', '--shadow-md', '--shadow-lg', '--shadow-xl', '--shadow-inner', '--shadow-card', '--shadow-button', '--shadow-dropdown'
  ];

  cssVarsToClear.forEach(cssVar => {
    root.style.removeProperty(cssVar);
  });

  // Apply all color variables with !important to override CSS classes
  Object.entries(theme.colors).forEach(([key, value]) => {
    const cssVar = `--${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
    root.style.setProperty(cssVar, value, 'important');
  });

  // Apply theme-specific input background if defined
  if (theme.colors.inputBackground) {
    root.style.setProperty(
      '--input-background',
      theme.colors.inputBackground,
      'important',
    );
  } else if (theme.colors.input) {
    // For other themes, use the regular input color
    root.style.setProperty(
      '--input-background',
      theme.colors.input,
      'important',
    );
  }
  // If neither is defined, CSS defaults will be used

  // Set dynamic outline button background based on theme
  const outlineButtonBg = getOutlineButtonBackground(theme);
  root.style.setProperty('--outline-button-bg', outlineButtonBg, 'important');

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

  // Set data attribute on body for CSS selectors
  document.body.setAttribute('data-theme-styles', buttonStyles.join(' '));

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

/**
 * Determines the appropriate background color for outline buttons based on theme
 */
function getOutlineButtonBackground(theme: Theme): string {
  // If theme has no colors defined, use CSS defaults (light theme)
  if (!theme.colors.background) {
    return 'transparent';
  }

  // Parse background lightness to determine if theme is light or dark
  const backgroundHsl = theme.colors.background;
  const lightnessMatch = backgroundHsl.match(/(\d+(?:\.\d+)?)%$/);
  const lightness = lightnessMatch ? parseFloat(lightnessMatch[1]) : 50;

  // If background is very dark (< 20% lightness), use card color for subtle background
  // If background is light (> 50% lightness), use transparent
  if (lightness < 20) {
    return `hsl(${theme.colors.card})`;
  } else if (lightness > 50) {
    return 'transparent';
  } else {
    // For mid-range themes, use a slightly lighter version of the background
    return `hsl(${theme.colors.secondary})`;
  }
}

export interface Theme {
  name: string;
  displayName: string;
  backgroundImage?: string;
  inherits?: string;
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
    inputBackground?: string;
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
}
