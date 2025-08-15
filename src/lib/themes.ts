export interface Theme {
  name: string;
  displayName: string;
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
  };
}

export const themes: Theme[] = [
  {
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
      input: '0 0% 89.8%',
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
      mono: 'JetBrains Mono, Consolas, Monaco, monospace',
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
    },
  },
  {
    name: 'dark',
    displayName: 'Dark',
    colors: {
      background: '0 0% 3.9%',
      foreground: '0 0% 98%',
      card: '0 0% 8%',
      cardForeground: '0 0% 98%',
      popover: '0 0% 8%',
      popoverForeground: '0 0% 98%',
      primary: '0 0% 98%',
      primaryForeground: '0 0% 9%',
      secondary: '0 0% 14.9%',
      secondaryForeground: '0 0% 98%',
      muted: '0 0% 14.9%',
      mutedForeground: '0 0% 63.9%',
      accent: '0 0% 14.9%',
      accentForeground: '0 0% 98%',
      destructive: '0 62.8% 30.6%',
      destructiveForeground: '0 0% 98%',
      border: '0 0% 14.9%',
      input: '0 0% 14.9%',
      ring: '0 0% 83.1%',
      chart1: '220 70% 50%',
      chart2: '160 60% 45%',
      chart3: '30 80% 55%',
      chart4: '280 65% 60%',
      chart5: '340 75% 55%',
    },
    fonts: {
      sans: 'Inter, system-ui, sans-serif',
      serif: 'Georgia, serif',
      mono: 'JetBrains Mono, Consolas, Monaco, monospace',
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
      sm: '0 1px 2px 0 rgb(0 0 0 / 0.1)',
      md: '0 4px 6px -1px rgb(0 0 0 / 0.2), 0 2px 4px -2px rgb(0 0 0 / 0.2)',
      lg: '0 10px 15px -3px rgb(0 0 0 / 0.3), 0 4px 6px -4px rgb(0 0 0 / 0.2)',
      xl: '0 20px 25px -5px rgb(0 0 0 / 0.4), 0 8px 10px -6px rgb(0 0 0 / 0.2)',
      inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.1)',
      card: '0 2px 8px 0 rgb(0 0 0 / 0.2), 0 1px 3px -1px rgb(0 0 0 / 0.1)',
      button: '0 1px 2px 0 rgb(0 0 0 / 0.1)',
    },
  },
  {
    name: 'win95',
    displayName: 'Windows 95',
    colors: {
      background: '0 0% 75%',
      foreground: '0 0% 0%',
      card: '0 0% 78%',
      cardForeground: '0 0% 0%',
      popover: '0 0% 75%',
      popoverForeground: '0 0% 0%',
      primary: '240 100% 25%',
      primaryForeground: '0 0% 100%',
      secondary: '0 0% 85%',
      secondaryForeground: '0 0% 0%',
      muted: '0 0% 85%',
      mutedForeground: '0 0% 25%',
      accent: '60 100% 50%',
      accentForeground: '0 0% 0%',
      destructive: '0 100% 50%',
      destructiveForeground: '0 0% 100%',
      border: '0 0% 50%',
      input: '0 0% 100%',
      ring: '240 100% 25%',
      chart1: '240 100% 25%',
      chart2: '0 100% 50%',
      chart3: '60 100% 50%',
      chart4: '300 100% 50%',
      chart5: '120 100% 50%',
    },
    fonts: {
      sans: 'MS Sans Serif, Tahoma, Arial, sans-serif',
      serif: 'Times New Roman, serif',
      mono: 'Courier New, monospace',
      heading: 'MS Sans Serif, Tahoma, Arial, sans-serif',
      body: 'MS Sans Serif, Tahoma, Arial, sans-serif',
    },
    corners: {
      none: '0px',
      sm: '0px',
      md: '0px',
      lg: '0px',
      xl: '0px',
      full: '0px',
    },
    shadows: {
      none: 'none',
      sm: 'inset 2px 2px 0 0 rgb(128 128 128), inset -2px -2px 0 0 rgb(255 255 255)',
      md: 'inset 2px 2px 0 0 rgb(128 128 128), inset -2px -2px 0 0 rgb(255 255 255)',
      lg: 'inset 3px 3px 0 0 rgb(128 128 128), inset -3px -3px 0 0 rgb(255 255 255)',
      xl: 'inset 3px 3px 0 0 rgb(128 128 128), inset -3px -3px 0 0 rgb(255 255 255)',
      inner: 'inset 2px 2px 0 0 rgb(128 128 128), inset -2px -2px 0 0 rgb(255 255 255)',
      card: 'inset 2px 2px 0 0 rgb(128 128 128), inset -2px -2px 0 0 rgb(255 255 255)',
      button: 'inset -2px -2px 0 0 rgb(128 128 128), inset 2px 2px 0 0 rgb(255 255 255)',
    },
  },
  {
    name: 'nes',
    displayName: 'Nintendo NES',
    colors: {
      background: '0 0% 8%',
      foreground: '0 0% 100%',
      card: '0 0% 12%',
      cardForeground: '0 0% 100%',
      popover: '0 0% 12%',
      popoverForeground: '0 0% 100%',
      primary: '0 100% 50%',
      primaryForeground: '0 0% 100%',
      secondary: '0 0% 20%',
      secondaryForeground: '0 0% 100%',
      muted: '0 0% 15%',
      mutedForeground: '0 0% 70%',
      accent: '120 100% 50%',
      accentForeground: '0 0% 0%',
      destructive: '0 100% 50%',
      destructiveForeground: '0 0% 100%',
      border: '0 0% 30%',
      input: '0 0% 5%',
      ring: '0 100% 50%',
      chart1: '0 100% 50%',
      chart2: '240 100% 50%',
      chart3: '120 100% 50%',
      chart4: '60 100% 50%',
      chart5: '300 100% 50%',
    },
    fonts: {
      sans: '"Perfect DOS VGA 437", "MS Gothic", "Terminal", "System", Silkscreen, VT323, "Courier New", Consolas, Monaco, "Lucida Console", "Fixedsys", monospace',
      serif: '"Perfect DOS VGA 437", "MS Gothic", "Terminal", "System", Silkscreen, VT323, "Courier New", Consolas, Monaco, "Lucida Console", "Fixedsys", monospace',
      mono: '"Perfect DOS VGA 437", "MS Gothic", "Terminal", "System", Silkscreen, VT323, "Courier New", Consolas, Monaco, "Lucida Console", "Fixedsys", monospace',
      heading: '"Perfect DOS VGA 437", "MS Gothic", "Terminal", "System", Silkscreen, VT323, "Courier New", Consolas, Monaco, "Lucida Console", "Fixedsys", monospace',
      body: '"Perfect DOS VGA 437", "MS Gothic", "Terminal", "System", Silkscreen, VT323, "Courier New", Consolas, Monaco, "Lucida Console", "Fixedsys", monospace',
    },
    corners: {
      none: '0px',
      sm: '0px',
      md: '0px',
      lg: '0px',
      xl: '0px',
      full: '0px',
    },
    shadows: {
      none: 'none',
      sm: '2px 2px 0 0 rgb(0 0 0)',
      md: '2px 2px 0 0 rgb(0 0 0)',
      lg: '2px 2px 0 0 rgb(0 0 0)',
      xl: '2px 2px 0 0 rgb(0 0 0)',
      inner: 'inset 2px 2px 0 0 rgb(0 0 0)',
      card: 'none',
      button: '2px 2px 0 0 rgb(0 0 0)',
    },
  },
  {
    name: 'amiga',
    displayName: 'Amiga Workbench',
    colors: {
      background: '209 100% 78%',
      foreground: '0 0% 0%',
      card: '209 100% 85%',
      cardForeground: '0 0% 0%',
      popover: '209 100% 85%',
      popoverForeground: '0 0% 0%',
      primary: '25 100% 50%',
      primaryForeground: '0 0% 0%',
      secondary: '0 0% 85%',
      secondaryForeground: '0 0% 0%',
      muted: '0 0% 90%',
      mutedForeground: '0 0% 30%',
      accent: '25 100% 50%',
      accentForeground: '0 0% 0%',
      destructive: '0 100% 50%',
      destructiveForeground: '0 0% 100%',
      border: '0 0% 0%',
      input: '0 0% 100%',
      ring: '25 100% 50%',
      chart1: '209 100% 50%',
      chart2: '25 100% 50%',
      chart3: '0 100% 50%',
      chart4: '300 100% 50%',
      chart5: '60 100% 50%',
    },
    fonts: {
      sans: 'Topaz, Monaco, Courier, monospace',
      serif: 'Times, Times New Roman, serif',
      mono: 'Topaz, Monaco, Courier, monospace',
      heading: 'Topaz, Monaco, Courier, monospace',
      body: 'Topaz, Monaco, Courier, monospace',
    },
    corners: {
      none: '0px',
      sm: '0px',
      md: '0px',
      lg: '0px',
      xl: '0px',
      full: '0px',
    },
    shadows: {
      none: 'none',
      sm: 'inset -1px -1px 0 0 rgb(85 85 85), inset 1px 1px 0 0 rgb(255 255 255)',
      md: 'inset -2px -2px 0 0 rgb(85 85 85), inset 2px 2px 0 0 rgb(255 255 255)',
      lg: 'inset -2px -2px 0 0 rgb(85 85 85), inset 2px 2px 0 0 rgb(255 255 255)',
      xl: 'inset -3px -3px 0 0 rgb(85 85 85), inset 3px 3px 0 0 rgb(255 255 255)',
      inner: 'inset 1px 1px 0 0 rgb(85 85 85), inset -1px -1px 0 0 rgb(255 255 255)',
      card: 'inset -1px -1px 0 0 rgb(85 85 85), inset 1px 1px 0 0 rgb(255 255 255)',
      button: 'inset -1px -1px 0 0 rgb(85 85 85), inset 1px 1px 0 0 rgb(255 255 255)',
    },
  },
  {
    name: 'macintosh',
    displayName: 'Classic Mac',
    colors: {
      background: '0 0% 100%',
      foreground: '0 0% 0%',
      card: '0 0% 100%',
      cardForeground: '0 0% 0%',
      popover: '0 0% 100%',
      popoverForeground: '0 0% 0%',
      primary: '0 0% 0%',
      primaryForeground: '0 0% 100%',
      secondary: '0 0% 87%',
      secondaryForeground: '0 0% 0%',
      muted: '0 0% 93%',
      mutedForeground: '0 0% 20%',
      accent: '0 0% 87%',
      accentForeground: '0 0% 0%',
      destructive: '0 0% 0%',
      destructiveForeground: '0 0% 100%',
      border: '0 0% 0%',
      input: '0 0% 100%',
      ring: '0 0% 0%',
      chart1: '0 0% 0%',
      chart2: '0 0% 20%',
      chart3: '0 0% 40%',
      chart4: '0 0% 60%',
      chart5: '0 0% 80%',
    },
    fonts: {
      sans: 'Chicago, Monaco, Courier, monospace',
      serif: 'Times, Times New Roman, serif',
      mono: 'Monaco, Courier, monospace',
      heading: 'Chicago, Monaco, Courier, monospace',
      body: 'Chicago, Monaco, Courier, monospace',
    },
    corners: {
      none: '0px',
      sm: '0px',
      md: '8px',
      lg: '8px',
      xl: '8px',
      full: '8px',
    },
    shadows: {
      none: 'none',
      sm: '2px 2px 0 0 rgb(0 0 0)',
      md: '3px 3px 0 0 rgb(0 0 0)',
      lg: '4px 4px 0 0 rgb(0 0 0)',
      xl: '5px 5px 0 0 rgb(0 0 0)',
      inner: 'inset 1px 1px 0 0 rgb(0 0 0)',
      card: '2px 2px 0 0 rgb(0 0 0)',
      button: '2px 2px 0 0 rgb(0 0 0)',
    },
  },
];

export function getThemeByName(name: string): Theme | undefined {
  return themes.find(theme => theme.name === name);
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
}
