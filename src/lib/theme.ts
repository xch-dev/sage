import iconDark from '@/icon-dark.png';
import iconLight from '@/icon-light.png';
import { getColorLightness, makeColorTransparent } from './color-utils';
import { validateTheme, validateThemeJson } from './theme-schema-validation';
import { Theme } from './theme.type';
import { deepMerge } from './utils';

export async function loadUserTheme(themeJson: string): Promise<Theme | null> {
  try {
    // Validate in development only
    if (import.meta.env.DEV) {
      const validation = validateThemeJson(themeJson);
      if (!validation.success) {
        console.error(`User theme validation failed:`, validation.error);
        return null;
      }
    }

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

    // Validate in development only
    if (import.meta.env.DEV) {
      const validation = validateTheme(themeModule.default);
      if (!validation.success) {
        throw new Error(
          `Theme ${themeName} validation failed: ${validation.error}`,
        );
      }
    }

    let theme = themeModule.default as Theme;

    if (theme.inherits) {
      const inheritedTheme = await loadBuiltInTheme(
        theme.inherits,
        loadedThemes,
      );
      if (inheritedTheme) {
        theme = deepMerge(inheritedTheme, theme);
      }
    }

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
            theme.backgroundImage = (
              imageModule as { default: string }
            ).default;
          } else {
            // Fallback to a relative path if not found
            theme.backgroundImage = `../themes/${themeName}/${theme.backgroundImage}`;
          }
        }
      } catch (error) {
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

function applyCommonThemeProperties(theme: Theme, root: HTMLElement): void {
  // Set theme class for CSS selectors
  root.classList.add(`theme-${theme.name}`);

  // Set data attributes for theme styles
  const buttonStyles = theme.buttonStyles || [];
  root.setAttribute('data-theme-styles', buttonStyles.join(' '));

  // Apply background image if present
  if (theme.backgroundImage) {
    root.style.setProperty(
      '--background-image',
      `url(${theme.backgroundImage})`,
      'important',
    );

    // Apply background size (default: cover)
    const backgroundSize = theme.backgroundSize || 'cover';
    root.style.setProperty('--background-size', backgroundSize, 'important');

    // Apply background position (default: center)
    const backgroundPosition = theme.backgroundPosition || 'center';
    root.style.setProperty(
      '--background-position',
      backgroundPosition,
      'important',
    );

    // Apply background repeat (default: no-repeat)
    const backgroundRepeat = theme.backgroundRepeat || 'no-repeat';
    root.style.setProperty(
      '--background-repeat',
      backgroundRepeat,
      'important',
    );

    root.classList.add('has-background-image');
  } else {
    root.style.removeProperty('--background-image');
    root.style.removeProperty('--background-size');
    root.style.removeProperty('--background-position');
    root.style.removeProperty('--background-repeat');
    root.classList.remove('has-background-image');
  }
}

function applyThemeVariables(theme: Theme, root: HTMLElement): void {
  // Create mappings from theme properties to CSS variables
  const variableMappings = [
    {
      themeObj: theme.colors,
      transform: (key: string) =>
        `--${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`,
    },
    { themeObj: theme.fonts, transform: (key: string) => `--font-${key}` },
    { themeObj: theme.corners, transform: (key: string) => `--corner-${key}` },
    { themeObj: theme.shadows, transform: (key: string) => `--shadow-${key}` },
  ];

  variableMappings.forEach(({ themeObj, transform }) => {
    if (themeObj) {
      Object.entries(themeObj).forEach(([key, value]) => {
        if (value) {
          const cssVar = transform(key);
          root.style.setProperty(cssVar, value, 'important');
        }
      });
    }
  });
}

export function applyTheme(theme: Theme, root: HTMLElement) {
  // Remove any existing theme classes
  const existingThemeClasses = Array.from(root.classList).filter((cls) =>
    cls.startsWith('theme-'),
  );
  root.classList.remove(...existingThemeClasses);

  // Clear all CSS variables to reset to defaults
  [
    ...colorVariableNames,
    ...fontVariableNames,
    ...cornerVariableNames,
    ...shadowVariableNames,
    ...themeFeatureFlagVariableNames,
    ...navigationAndButtonVariableNames,
    ...backgroundImageAndTransparencyVariableNames,
    ...tableVariableNames,
    ...switchVariableNames,
    ...backdropFilterVariableNames,
    ...buttonVariableNames,
  ].forEach((cssVar) => {
    root.style.removeProperty(cssVar);
  });

  applyThemeVariables(theme, root);

  // Apply backdrop-filter variables if defined in colors object
  if (theme.colors) {
    const backdropFilterMap: Record<string, string> = {};
    [
      'cardBackdropFilter',
      'popoverBackdropFilter',
      'inputBackdropFilter',
    ].forEach((base) => {
      const cssVar = `--${base.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
      backdropFilterMap[`${base}`] = cssVar;
      backdropFilterMap[`${base}Webkit`] = `${cssVar}-webkit`;
    });

    Object.entries(backdropFilterMap).forEach(([themeKey, cssVar]) => {
      const value = theme.colors?.[themeKey as keyof typeof theme.colors];
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

    const transparentColorMap = [
      { value: backgroundValue, opacity: bodyOpacity, name: 'background' },
      { value: cardValue, opacity: cardOpacity, name: 'card' },
      { value: popoverValue, opacity: popoverOpacity, name: 'popover' },
    ];

    transparentColorMap.forEach(({ value, opacity, name }) => {
      if (value) {
        const transparentColor = makeColorTransparent(value, opacity);
        root.style.setProperty(
          `--${name}-transparent`,
          transparentColor,
          'important',
        );
      }
    });
  }

  if (theme.buttons) {
    const propertyToCssMap = {
      background: 'bg',
      color: 'color',
      border: 'border',
      borderStyle: 'border-style',
      borderWidth: 'border-width',
      borderColor: 'border-color',
      borderRadius: 'radius',
      boxShadow: 'shadow',
      backdropFilter: 'backdrop-filter',
      backdropFilterWebkit: 'backdrop-filter-webkit',
    };

    Object.entries(theme.buttons).forEach(([variant, config]) => {
      if (config) {
        // Apply base styles
        Object.entries(propertyToCssMap).forEach(([property, cssName]) => {
          const value = config[property as keyof typeof config];
          if (value && typeof value === 'string') {
            root.style.setProperty(
              `--btn-${variant}-${cssName}`,
              value,
              'important',
            );
          }
        });

        // Apply hover and active states using the same mapping
        ['hover', 'active'].forEach((state) => {
          const stateConfig = config[state as keyof typeof config];
          if (stateConfig && typeof stateConfig === 'object') {
            Object.entries(propertyToCssMap).forEach(
              ([property, baseCssName]) => {
                const value = (stateConfig as Record<string, unknown>)[
                  property
                ];
                if (value && typeof value === 'string') {
                  const cssName = `${state}-${baseCssName}`;
                  root.style.setProperty(
                    `--btn-${variant}-${cssName}`,
                    value,
                    'important',
                  );
                }
              },
            );

            // Handle transform property specifically for hover/active states
            const transform = (stateConfig as Record<string, unknown>)
              .transform;
            if (transform && typeof transform === 'string') {
              root.style.setProperty(
                `--btn-${variant}-${state}-transform`,
                transform,
                'important',
              );
            }
          }
        });
      }
    });
  }

  const buttonStyles = theme.buttonStyles || [];
  const buttonStyleMap = {
    gradient: 'gradient-buttons',
    shimmer: 'shimmer-effects',
    'pixel-art': 'pixel-art',
    '3d-effects': '3d-effects',
    'rounded-buttons': 'rounded-buttons',
  };

  // Set CSS variables for button style flags
  Object.entries(buttonStyleMap).forEach(([style, cssName]) => {
    root.style.setProperty(
      `--theme-has-${cssName}`,
      buttonStyles.includes(style) ? '1' : '0',
      'important',
    );
  });

  document.body.setAttribute('data-theme-styles', buttonStyles.join(' '));

  if (theme.tables) {
    const tableSections = [
      {
        obj: theme.tables,
        prefix: 'table',
        properties: ['background', 'border', 'borderRadius', 'boxShadow'],
      },
      {
        obj: theme.tables.header,
        prefix: 'table-header',
        properties: [
          'background',
          'color',
          'border',
          'fontWeight',
          'fontSize',
          'backdropFilter',
          'backdropFilterWebkit',
        ],
      },
      {
        obj: theme.tables.row,
        prefix: 'table-row',
        properties: [
          'background',
          'color',
          'border',
          'backdropFilter',
          'backdropFilterWebkit',
        ],
      },
      {
        obj: theme.tables.row?.hover,
        prefix: 'table-row-hover',
        properties: ['background', 'color'],
      },
      {
        obj: theme.tables.row?.selected,
        prefix: 'table-row-selected',
        properties: ['background', 'color'],
      },
      {
        obj: theme.tables.cell,
        prefix: 'table-cell',
        properties: ['padding', 'border', 'fontSize'],
      },
      {
        obj: theme.tables.footer,
        prefix: 'table-footer',
        properties: [
          'background',
          'color',
          'border',
          'backdropFilter',
          'backdropFilterWebkit',
        ],
      },
    ];

    tableSections.forEach(({ obj, prefix, properties }) => {
      if (obj) {
        properties.forEach((property) => {
          const value = (obj as Record<string, unknown>)[property];
          if (value && typeof value === 'string') {
            const cssVar = `--${prefix}-${property.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
            root.style.setProperty(cssVar, value, 'important');
          }
        });
      }
    });
  }

  if (theme.sidebar) {
    const sidebarProperties = [
      'background',
      'backdropFilter',
      'backdropFilterWebkit',
      'border',
    ];

    sidebarProperties.forEach((property) => {
      const value = (theme.sidebar as Record<string, unknown>)[property];
      if (value && typeof value === 'string') {
        const cssVar = `--sidebar-${property.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
        root.style.setProperty(cssVar, value, 'important');
      }
    });
  }

  // Apply common theme properties (background image, classes, etc.)
  applyCommonThemeProperties(theme, root);

  // Apply document-wide background image handling for main theme
  if (theme.backgroundImage) {
    document.body.classList.add('has-background-image');

    // Set background opacity variables
    const opacityDefaults = {
      body: 0.1,
      card: 0.75,
      popover: 0.9,
    };

    Object.entries(opacityDefaults).forEach(([key, defaultValue]) => {
      const opacity =
        theme.backgroundOpacity?.[
          key as keyof typeof theme.backgroundOpacity
        ] ?? defaultValue;
      root.style.setProperty(
        `--background-${key}-opacity`,
        opacity.toString(),
        'important',
      );
    });
  } else {
    document.body.classList.remove('has-background-image');
  }

  if (theme.switches) {
    const switchStates = ['checked', 'unchecked'] as const;

    switchStates.forEach((state) => {
      const switchConfig = theme.switches?.[state];
      if (switchConfig?.background) {
        root.style.setProperty(
          `--switch-${state}-bg`,
          switchConfig.background,
          'important',
        );
      }
    });
  }
}

export function applyThemeIsolated(theme: Theme, root: HTMLElement): void {
  applyThemeVariables(theme, root);
  applyCommonThemeProperties(theme, root);

  if (theme.backgroundImage) {
    const backgroundStyles = {
      backgroundImage: `url(${theme.backgroundImage})`,
      backgroundSize: theme.backgroundSize || 'cover',
      backgroundPosition: theme.backgroundPosition || 'center',
      backgroundRepeat: theme.backgroundRepeat || 'no-repeat',
    };

    Object.entries(backgroundStyles).forEach(([property, value]) => {
      root.style.setProperty(
        property.replace(/([A-Z])/g, '-$1').toLowerCase(),
        value,
      );
    });
  }

  // Set explicit background and text colors for complete isolation
  if (theme.colors?.card) {
    root.style.backgroundColor = theme.colors.card;
  }
  if (theme.colors?.cardForeground) {
    root.style.color = theme.colors.cardForeground;
  }
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

const colorVariableNames = [
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
];

const fontVariableNames = [
  '--font-sans',
  '--font-serif',
  '--font-mono',
  '--font-heading',
  '--font-body',
];

const cornerVariableNames = [
  '--corner-none',
  '--corner-sm',
  '--corner-md',
  '--corner-lg',
  '--corner-xl',
  '--corner-full',
];

const shadowVariableNames = [
  '--shadow-none',
  '--shadow-sm',
  '--shadow-md',
  '--shadow-lg',
  '--shadow-xl',
  '--shadow-inner',
  '--shadow-card',
  '--shadow-button',
  '--shadow-dropdown',
];

const themeFeatureFlagVariableNames = [
  '--theme-has-gradient-buttons',
  '--theme-has-shimmer-effects',
  '--theme-has-pixel-art',
  '--theme-has-3d-effects',
  '--theme-has-rounded-buttons',
];

const navigationAndButtonVariableNames = [
  '--outline-button-bg',
  '--nav-active-bg',
];

const backgroundImageAndTransparencyVariableNames = [
  '--background-image',
  '--background-size',
  '--background-position',
  '--background-repeat',
  '--background-transparent',
  '--card-transparent',
  '--popover-transparent',
  '--background-body-opacity',
  '--background-card-opacity',
  '--background-popover-opacity',
];

const tableVariableNames = [
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
];

const switchVariableNames = ['--switch-checked-bg', '--switch-unchecked-bg'];

const backdropFilterVariableNames = [
  '--card-backdrop-filter',
  '--card-backdrop-filter-webkit',
  '--popover-backdrop-filter',
  '--popover-backdrop-filter-webkit',
  '--input-backdrop-filter',
  '--input-backdrop-filter-webkit',
  '--table-header-backdrop-filter',
  '--table-header-backdrop-filter-webkit',
  '--table-row-backdrop-filter',
  '--table-row-backdrop-filter-webkit',
  '--table-footer-backdrop-filter',
  '--table-footer-backdrop-filter-webkit',
  '--sidebar-backdrop-filter',
  '--sidebar-backdrop-filter-webkit',
];

const buttonBaseVariableNames = [
  'bg',
  'color',
  'border',
  'border-style',
  'border-width',
  'border-color',
  'radius',
  'shadow',
  'backdrop-filter',
  'backdrop-filter-webkit',
  'hover-bg',
  'hover-color',
  'hover-transform',
  'hover-border-style',
  'hover-border-color',
  'hover-shadow',
  'active-bg',
  'active-color',
  'active-transform',
  'active-border-style',
  'active-border-color',
  'active-shadow',
];

// Generate all button variable combinations
const buttonVariableNames = [
  'default',
  'outline',
  'secondary',
  'destructive',
  'ghost',
  'link',
].flatMap((variant) =>
  buttonBaseVariableNames.map((baseName) => `--btn-${variant}-${baseName}`),
);
