import { useTheme } from '@/contexts/ThemeContext';

export function useThemeColors() {
  const { currentTheme } = useTheme();

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
