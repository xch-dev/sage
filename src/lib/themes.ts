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
}

export const themes: Theme[] = [
  {
    name: 'light',
    displayName: 'Light',
    colors: {
      background: '0 0% 100%',
      foreground: '0 0% 3.9%',
      card: '0 0% 100%',
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
  },
  {
    name: 'dark',
    displayName: 'Dark',
    colors: {
      background: '0 0% 3.9%',
      foreground: '0 0% 98%',
      card: '0 0% 3.9%',
      cardForeground: '0 0% 98%',
      popover: '0 0% 3.9%',
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
  },
  {
    name: 'warm',
    displayName: 'Warm Gray',
    colors: {
      background: '30 25% 96%',
      foreground: '30 10% 3%',
      card: '0 0% 100%',
      cardForeground: '30 10% 3%',
      popover: '0 0% 100%',
      popoverForeground: '30 10% 3%',
      primary: '30 10% 15%',
      primaryForeground: '30 25% 96%',
      secondary: '30 25% 90%',
      secondaryForeground: '30 10% 15%',
      muted: '30 25% 90%',
      mutedForeground: '30 10% 45%',
      accent: '30 25% 90%',
      accentForeground: '30 10% 15%',
      destructive: '0 84% 60%',
      destructiveForeground: '30 25% 96%',
      border: '30 25% 85%',
      input: '30 25% 85%',
      ring: '30 10% 15%',
      chart1: '30 10% 25%',
      chart2: '30 10% 35%',
      chart3: '30 10% 45%',
      chart4: '30 10% 55%',
      chart5: '30 10% 65%',
    },
  },
  {
    name: 'cool',
    displayName: 'Cool Gray',
    colors: {
      background: '220 25% 96%',
      foreground: '220 10% 3%',
      card: '0 0% 100%',
      cardForeground: '220 10% 3%',
      popover: '0 0% 100%',
      popoverForeground: '220 10% 3%',
      primary: '220 10% 15%',
      primaryForeground: '220 25% 96%',
      secondary: '220 25% 90%',
      secondaryForeground: '220 10% 15%',
      muted: '220 25% 90%',
      mutedForeground: '220 10% 45%',
      accent: '220 25% 90%',
      accentForeground: '220 10% 15%',
      destructive: '0 84% 60%',
      destructiveForeground: '220 25% 96%',
      border: '220 25% 85%',
      input: '220 25% 85%',
      ring: '220 10% 15%',
      chart1: '220 10% 25%',
      chart2: '220 10% 35%',
      chart3: '220 10% 45%',
      chart4: '220 10% 55%',
      chart5: '220 10% 65%',
    },
  },
  {
    name: 'sepia',
    displayName: 'Sepia',
    colors: {
      background: '40 35% 95%',
      foreground: '40 20% 5%',
      card: '0 0% 100%',
      cardForeground: '40 20% 5%',
      popover: '0 0% 100%',
      popoverForeground: '40 20% 5%',
      primary: '40 20% 20%',
      primaryForeground: '40 35% 95%',
      secondary: '40 35% 88%',
      secondaryForeground: '40 20% 20%',
      muted: '40 35% 88%',
      mutedForeground: '40 20% 40%',
      accent: '40 35% 88%',
      accentForeground: '40 20% 20%',
      destructive: '0 84% 60%',
      destructiveForeground: '40 35% 95%',
      border: '40 35% 82%',
      input: '40 35% 82%',
      ring: '40 20% 20%',
      chart1: '40 20% 30%',
      chart2: '40 20% 40%',
      chart3: '40 20% 50%',
      chart4: '40 20% 60%',
      chart5: '40 20% 70%',
    },
  },
  {
    name: 'slate',
    displayName: 'Slate',
    colors: {
      background: '240 15% 96%',
      foreground: '240 10% 3%',
      card: '0 0% 100%',
      cardForeground: '240 10% 3%',
      popover: '0 0% 100%',
      popoverForeground: '240 10% 3%',
      primary: '240 10% 15%',
      primaryForeground: '240 15% 96%',
      secondary: '240 15% 90%',
      secondaryForeground: '240 10% 15%',
      muted: '240 15% 90%',
      mutedForeground: '240 10% 45%',
      accent: '240 15% 90%',
      accentForeground: '240 10% 15%',
      destructive: '0 84% 60%',
      destructiveForeground: '240 15% 96%',
      border: '240 15% 85%',
      input: '240 15% 85%',
      ring: '240 10% 15%',
      chart1: '240 10% 25%',
      chart2: '240 10% 35%',
      chart3: '240 10% 45%',
      chart4: '240 10% 55%',
      chart5: '240 10% 65%',
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
}
