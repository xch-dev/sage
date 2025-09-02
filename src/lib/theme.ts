import iconDark from '@/icon-dark.png';
import iconLight from '@/icon-light.png';
import { makeColorTransparent } from './color-utils';
import { validateTheme } from './theme-schema-validation';
import { Theme } from './theme.type';
import { deepMerge } from './utils';

export async function loadUserTheme(themeJson: string): Promise<Theme | null> {
  try {
    let theme = validateTheme(JSON.parse(themeJson));

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
        // we allow remote urls and local files for built in themes
        // local images get imported from the theme's folder
        if (
          !(
            theme.backgroundImage.startsWith('http://') ||
            theme.backgroundImage.startsWith('https://')
          ) &&
          !theme.backgroundImage.startsWith('/')
        ) {
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

  if (theme.backgroundImage) {
    root.style.setProperty(
      '--background-image',
      `url(${theme.backgroundImage})`,
      'important',
    );

    const backgroundSize = theme.backgroundSize || 'cover';
    root.style.setProperty('--background-size', backgroundSize, 'important');

    const backgroundPosition = theme.backgroundPosition || 'center';
    root.style.setProperty(
      '--background-position',
      backgroundPosition,
      'important',
    );

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
    ...backgroundImageVariableNames,
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

  // Set navigation active background with transparency
  if (theme.colors?.primary) {
    const navActiveBg = makeColorTransparent(theme.colors.primary, 0.1);
    root.style.setProperty('--nav-active-bg', navActiveBg, 'important');
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
          'padding',
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

    // Handle switch thumb background
    if (theme.switches.thumb?.background) {
      root.style.setProperty(
        '--switch-thumb-bg',
        theme.switches.thumb.background,
        'important',
      );
    }
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

const colorVariableNames = [
  '--background',
  '--background-transparent',
  '--foreground',
  '--card',
  '--card-foreground',
  '--card-transparent',
  '--popover',
  '--popover-transparent',
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

const backgroundImageVariableNames = [
  '--background-image',
  '--background-size',
  '--background-position',
  '--background-repeat',
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
  '--table-header-padding',
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

const switchVariableNames = ['--switch-checked-bg', '--switch-unchecked-bg', '--switch-thumb-bg'];

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
