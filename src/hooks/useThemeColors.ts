import { useTheme } from '@/contexts/ThemeContext';

export function useThemeColors() {
  const { currentTheme } = useTheme();

  // Provide fallback values if currentTheme is null
  if (!currentTheme) {
    return {
      // Default light theme colors
      background: 'hsl(0 0% 100%)',
      foreground: 'hsl(0 0% 3.9%)',
      card: 'hsl(0 0% 98%)',
      cardForeground: 'hsl(0 0% 3.9%)',
      popover: 'hsl(0 0% 100%)',
      popoverForeground: 'hsl(0 0% 3.9%)',
      primary: 'hsl(0 0% 9%)',
      primaryForeground: 'hsl(0 0% 98%)',
      secondary: 'hsl(0 0% 96.1%)',
      secondaryForeground: 'hsl(0 0% 9%)',
      muted: 'hsl(0 0% 96.1%)',
      mutedForeground: 'hsl(0 0% 45.1%)',
      accent: 'hsl(0 0% 96.1%)',
      accentForeground: 'hsl(0 0% 9%)',
      destructive: 'hsl(0 84.2% 60.2%)',
      destructiveForeground: 'hsl(0 0% 98%)',
      border: 'hsl(0 0% 89.8%)',
      input: 'hsl(0 0% 96.1%)',
      ring: 'hsl(0 0% 3.9%)',
      chart1: 'hsl(12 76% 61%)',
      chart2: 'hsl(173 58% 39%)',
      chart3: 'hsl(197 37% 24%)',
      chart4: 'hsl(43 74% 66%)',
      chart5: 'hsl(27 87% 67%)',

      // Raw HSL values for use with opacity
      backgroundHsl: '0 0% 100%',
      foregroundHsl: '0 0% 3.9%',
      primaryHsl: '0 0% 9%',
      secondaryHsl: '0 0% 96.1%',
      accentHsl: '0 0% 96.1%',
      destructiveHsl: '0 84.2% 60.2%',

      // Font families
      fonts: {
        sans: 'Inter, system-ui, sans-serif',
        serif: 'Georgia, serif',
        mono: 'Courier New, Monaco, Consolas, monospace',
        heading: 'Inter, system-ui, sans-serif',
        body: 'Inter, system-ui, sans-serif',
      },
    };
  }

  return {
    // Convert HSL values to CSS color strings
    background: `hsl(${currentTheme.colors.background})`,
    foreground: `hsl(${currentTheme.colors.foreground})`,
    card: `hsl(${currentTheme.colors.card})`,
    cardForeground: `hsl(${currentTheme.colors.cardForeground})`,
    popover: `hsl(${currentTheme.colors.popover})`,
    popoverForeground: `hsl(${currentTheme.colors.popoverForeground})`,
    primary: `hsl(${currentTheme.colors.primary})`,
    primaryForeground: `hsl(${currentTheme.colors.primaryForeground})`,
    secondary: `hsl(${currentTheme.colors.secondary})`,
    secondaryForeground: `hsl(${currentTheme.colors.secondaryForeground})`,
    muted: `hsl(${currentTheme.colors.muted})`,
    mutedForeground: `hsl(${currentTheme.colors.mutedForeground})`,
    accent: `hsl(${currentTheme.colors.accent})`,
    accentForeground: `hsl(${currentTheme.colors.accentForeground})`,
    destructive: `hsl(${currentTheme.colors.destructive})`,
    destructiveForeground: `hsl(${currentTheme.colors.destructiveForeground})`,
    border: `hsl(${currentTheme.colors.border})`,
    input: `hsl(${currentTheme.colors.input})`,
    ring: `hsl(${currentTheme.colors.ring})`,
    chart1: `hsl(${currentTheme.colors.chart1})`,
    chart2: `hsl(${currentTheme.colors.chart2})`,
    chart3: `hsl(${currentTheme.colors.chart3})`,
    chart4: `hsl(${currentTheme.colors.chart4})`,
    chart5: `hsl(${currentTheme.colors.chart5})`,

    // Raw HSL values for use with opacity
    backgroundHsl: currentTheme.colors.background,
    foregroundHsl: currentTheme.colors.foreground,
    primaryHsl: currentTheme.colors.primary,
    secondaryHsl: currentTheme.colors.secondary,
    accentHsl: currentTheme.colors.accent,
    destructiveHsl: currentTheme.colors.destructive,

    // Font families
    fonts: {
      sans: currentTheme.fonts.sans,
      serif: currentTheme.fonts.serif,
      mono: currentTheme.fonts.mono,
      heading: currentTheme.fonts.heading,
      body: currentTheme.fonts.body,
    },
  };
}
