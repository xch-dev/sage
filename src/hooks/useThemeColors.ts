import { useTheme } from '@/contexts/ThemeContext';

export function useThemeColors() {
  const { currentTheme } = useTheme();

  // If no theme is available, return CSS variable references
  // CSS already provides light theme defaults
  if (!currentTheme) {
    return {
      // Return CSS variable references - CSS provides light theme defaults
      background: 'hsl(var(--background))',
      foreground: 'hsl(var(--foreground))',
      card: 'hsl(var(--card))',
      cardForeground: 'hsl(var(--card-foreground))',
      popover: 'hsl(var(--popover))',
      popoverForeground: 'hsl(var(--popover-foreground))',
      primary: 'hsl(var(--primary))',
      primaryForeground: 'hsl(var(--primary-foreground))',
      secondary: 'hsl(var(--secondary))',
      secondaryForeground: 'hsl(var(--secondary-foreground))',
      muted: 'hsl(var(--muted))',
      mutedForeground: 'hsl(var(--muted-foreground))',
      accent: 'hsl(var(--accent))',
      accentForeground: 'hsl(var(--accent-foreground))',
      destructive: 'hsl(var(--destructive))',
      destructiveForeground: 'hsl(var(--destructive-foreground))',
      border: 'hsl(var(--border))',
      input: 'hsl(var(--input))',
      ring: 'hsl(var(--ring))',
      chart1: 'hsl(var(--chart-1))',
      chart2: 'hsl(var(--chart-2))',
      chart3: 'hsl(var(--chart-3))',
      chart4: 'hsl(var(--chart-4))',
      chart5: 'hsl(var(--chart-5))',

      // Raw HSL values for use with opacity
      backgroundHsl: 'var(--background)',
      foregroundHsl: 'var(--foreground)',
      primaryHsl: 'var(--primary)',
      secondaryHsl: 'var(--secondary)',
      accentHsl: 'var(--accent)',
      destructiveHsl: 'var(--destructive)',

      // Font families - use CSS defaults
      fonts: {
        sans: 'Inter, system-ui, sans-serif',
        serif: 'Georgia, serif',
        mono: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace',
        heading: 'Inter, system-ui, sans-serif',
        body: 'Inter, system-ui, sans-serif',
      },

      // Button configurations - use CSS variable references
      buttons: {
        default: {
          background: 'var(--btn-default-bg)',
          color: 'var(--btn-default-color)',
          border: 'var(--btn-default-border)',
          borderStyle: 'var(--btn-default-border-style)',
          borderWidth: 'var(--btn-default-border-width)',
          borderColor: 'var(--btn-default-border-color)',
          borderRadius: 'var(--btn-default-radius)',
          boxShadow: 'var(--btn-default-shadow)',
          hover: {
            background: 'var(--btn-default-hover-bg)',
            color: 'var(--btn-default-hover-color)',
            transform: 'var(--btn-default-hover-transform)',
            borderStyle: 'var(--btn-default-hover-border-style)',
            borderColor: 'var(--btn-default-hover-border-color)',
            boxShadow: 'var(--btn-default-hover-shadow)',
          },
          active: {
            background: 'var(--btn-default-active-bg)',
            color: 'var(--btn-default-active-color)',
            transform: 'var(--btn-default-active-transform)',
            borderStyle: 'var(--btn-default-active-border-style)',
            borderColor: 'var(--btn-default-active-border-color)',
            boxShadow: 'var(--btn-default-active-shadow)',
          },
        },
        outline: {
          background: 'var(--btn-outline-bg)',
          color: 'var(--btn-outline-color)',
          border: 'var(--btn-outline-border)',
          borderStyle: 'var(--btn-outline-border-style)',
          borderWidth: 'var(--btn-outline-border-width)',
          borderColor: 'var(--btn-outline-border-color)',
          borderRadius: 'var(--btn-outline-radius)',
          boxShadow: 'var(--btn-outline-shadow)',
          hover: {
            background: 'var(--btn-outline-hover-bg)',
            color: 'var(--btn-outline-hover-color)',
            transform: 'var(--btn-outline-hover-transform)',
            borderStyle: 'var(--btn-outline-hover-border-style)',
            borderColor: 'var(--btn-outline-hover-border-color)',
            boxShadow: 'var(--btn-outline-hover-shadow)',
          },
          active: {
            background: 'var(--btn-outline-active-bg)',
            color: 'var(--btn-outline-active-color)',
            transform: 'var(--btn-outline-active-transform)',
            borderStyle: 'var(--btn-outline-active-border-style)',
            borderColor: 'var(--btn-outline-active-border-color)',
            boxShadow: 'var(--btn-outline-active-shadow)',
          },
        },
        secondary: {
          background: 'var(--btn-secondary-bg)',
          color: 'var(--btn-secondary-color)',
          border: 'var(--btn-secondary-border)',
          borderStyle: 'var(--btn-secondary-border-style)',
          borderWidth: 'var(--btn-secondary-border-width)',
          borderColor: 'var(--btn-secondary-border-color)',
          borderRadius: 'var(--btn-secondary-radius)',
          boxShadow: 'var(--btn-secondary-shadow)',
          hover: {
            background: 'var(--btn-secondary-hover-bg)',
            color: 'var(--btn-secondary-hover-color)',
            transform: 'var(--btn-secondary-hover-transform)',
            borderStyle: 'var(--btn-secondary-hover-border-style)',
            borderColor: 'var(--btn-secondary-hover-border-color)',
            boxShadow: 'var(--btn-secondary-hover-shadow)',
          },
          active: {
            background: 'var(--btn-secondary-active-bg)',
            color: 'var(--btn-secondary-active-color)',
            transform: 'var(--btn-secondary-active-transform)',
            borderStyle: 'var(--btn-secondary-active-border-style)',
            borderColor: 'var(--btn-secondary-active-border-color)',
            boxShadow: 'var(--btn-secondary-active-shadow)',
          },
        },
        destructive: {
          background: 'var(--btn-destructive-bg)',
          color: 'var(--btn-destructive-color)',
          border: 'var(--btn-destructive-border)',
          borderStyle: 'var(--btn-destructive-border-style)',
          borderWidth: 'var(--btn-destructive-border-width)',
          borderColor: 'var(--btn-destructive-border-color)',
          borderRadius: 'var(--btn-destructive-radius)',
          boxShadow: 'var(--btn-destructive-shadow)',
          hover: {
            background: 'var(--btn-destructive-hover-bg)',
            color: 'var(--btn-destructive-hover-color)',
            transform: 'var(--btn-destructive-hover-transform)',
            borderStyle: 'var(--btn-destructive-hover-border-style)',
            borderColor: 'var(--btn-destructive-hover-border-color)',
            boxShadow: 'var(--btn-destructive-hover-shadow)',
          },
          active: {
            background: 'var(--btn-destructive-active-bg)',
            color: 'var(--btn-destructive-active-color)',
            transform: 'var(--btn-destructive-active-transform)',
            borderStyle: 'var(--btn-destructive-active-border-style)',
            borderColor: 'var(--btn-destructive-active-border-color)',
            boxShadow: 'var(--btn-destructive-active-shadow)',
          },
        },
        ghost: {
          background: 'var(--btn-ghost-bg)',
          color: 'var(--btn-ghost-color)',
          border: 'var(--btn-ghost-border)',
          borderStyle: 'var(--btn-ghost-border-style)',
          borderWidth: 'var(--btn-ghost-border-width)',
          borderColor: 'var(--btn-ghost-border-color)',
          borderRadius: 'var(--btn-ghost-radius)',
          boxShadow: 'var(--btn-ghost-shadow)',
          hover: {
            background: 'var(--btn-ghost-hover-bg)',
            color: 'var(--btn-ghost-hover-color)',
            transform: 'var(--btn-ghost-hover-transform)',
            borderStyle: 'var(--btn-ghost-hover-border-style)',
            borderColor: 'var(--btn-ghost-hover-border-color)',
            boxShadow: 'var(--btn-ghost-hover-shadow)',
          },
          active: {
            background: 'var(--btn-ghost-active-bg)',
            color: 'var(--btn-ghost-active-color)',
            transform: 'var(--btn-ghost-active-transform)',
            borderStyle: 'var(--btn-ghost-active-border-style)',
            borderColor: 'var(--btn-ghost-active-border-color)',
            boxShadow: 'var(--btn-ghost-active-shadow)',
          },
        },
        link: {
          background: 'var(--btn-link-bg)',
          color: 'var(--btn-link-color)',
          border: 'var(--btn-link-border)',
          borderStyle: 'var(--btn-link-border-style)',
          borderWidth: 'var(--btn-link-border-width)',
          borderColor: 'var(--btn-link-border-color)',
          borderRadius: 'var(--btn-link-radius)',
          boxShadow: 'var(--btn-link-shadow)',
          hover: {
            background: 'var(--btn-link-hover-bg)',
            color: 'var(--btn-link-hover-color)',
            transform: 'var(--btn-link-hover-transform)',
            borderStyle: 'var(--btn-link-hover-border-style)',
            borderColor: 'var(--btn-link-hover-border-color)',
            boxShadow: 'var(--btn-link-hover-shadow)',
          },
          active: {
            background: 'var(--btn-link-active-bg)',
            color: 'var(--btn-link-active-color)',
            transform: 'var(--btn-link-active-transform)',
            borderStyle: 'var(--btn-link-active-border-style)',
            borderColor: 'var(--btn-link-active-border-color)',
            boxShadow: 'var(--btn-link-active-shadow)',
          },
        },
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
    fonts: currentTheme.fonts,

    // Button configurations with fallbacks to theme defaults
    buttons: {
      default: {
        background:
          currentTheme.buttons?.default?.background ||
          `hsl(${currentTheme.colors.primary})`,
        color:
          currentTheme.buttons?.default?.color ||
          `hsl(${currentTheme.colors.primaryForeground})`,
        border: currentTheme.buttons?.default?.border || 'none',
        borderStyle: currentTheme.buttons?.default?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.default?.borderWidth || '0',
        borderColor:
          currentTheme.buttons?.default?.borderColor || 'transparent',
        borderRadius:
          currentTheme.buttons?.default?.borderRadius ||
          currentTheme.corners.md,
        boxShadow:
          currentTheme.buttons?.default?.boxShadow ||
          currentTheme.shadows.button,
        hover: currentTheme.buttons?.default?.hover,
        active: currentTheme.buttons?.default?.active,
      },
      outline: {
        background: currentTheme.buttons?.outline?.background || 'transparent',
        color:
          currentTheme.buttons?.outline?.color ||
          `hsl(${currentTheme.colors.foreground})`,
        border:
          currentTheme.buttons?.outline?.border ||
          `1px solid hsl(${currentTheme.colors.border})`,
        borderStyle: currentTheme.buttons?.outline?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.outline?.borderWidth || '1px',
        borderColor:
          currentTheme.buttons?.outline?.borderColor ||
          `hsl(${currentTheme.colors.border})`,
        borderRadius:
          currentTheme.buttons?.outline?.borderRadius ||
          currentTheme.corners.md,
        boxShadow:
          currentTheme.buttons?.outline?.boxShadow ||
          currentTheme.shadows.button,
        hover: currentTheme.buttons?.outline?.hover,
        active: currentTheme.buttons?.outline?.active,
      },
      secondary: {
        background:
          currentTheme.buttons?.secondary?.background ||
          `hsl(${currentTheme.colors.secondary})`,
        color:
          currentTheme.buttons?.secondary?.color ||
          `hsl(${currentTheme.colors.secondaryForeground})`,
        border: currentTheme.buttons?.secondary?.border || 'none',
        borderStyle: currentTheme.buttons?.secondary?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.secondary?.borderWidth || '0',
        borderColor:
          currentTheme.buttons?.secondary?.borderColor || 'transparent',
        borderRadius:
          currentTheme.buttons?.secondary?.borderRadius ||
          currentTheme.corners.md,
        boxShadow:
          currentTheme.buttons?.secondary?.boxShadow ||
          currentTheme.shadows.button,
        hover: currentTheme.buttons?.secondary?.hover,
        active: currentTheme.buttons?.secondary?.active,
      },
      destructive: {
        background:
          currentTheme.buttons?.destructive?.background ||
          `hsl(${currentTheme.colors.destructive})`,
        color:
          currentTheme.buttons?.destructive?.color ||
          `hsl(${currentTheme.colors.destructiveForeground})`,
        border: currentTheme.buttons?.destructive?.border || 'none',
        borderStyle: currentTheme.buttons?.destructive?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.destructive?.borderWidth || '0',
        borderColor:
          currentTheme.buttons?.destructive?.borderColor || 'transparent',
        borderRadius:
          currentTheme.buttons?.destructive?.borderRadius ||
          currentTheme.corners.md,
        boxShadow:
          currentTheme.buttons?.destructive?.boxShadow ||
          currentTheme.shadows.button,
        hover: currentTheme.buttons?.destructive?.hover,
        active: currentTheme.buttons?.destructive?.active,
      },
      ghost: {
        background: currentTheme.buttons?.ghost?.background || 'transparent',
        color:
          currentTheme.buttons?.ghost?.color ||
          `hsl(${currentTheme.colors.foreground})`,
        border: currentTheme.buttons?.ghost?.border || 'none',
        borderStyle: currentTheme.buttons?.ghost?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.ghost?.borderWidth || '0',
        borderColor: currentTheme.buttons?.ghost?.borderColor || 'transparent',
        borderRadius:
          currentTheme.buttons?.ghost?.borderRadius || currentTheme.corners.md,
        boxShadow: currentTheme.buttons?.ghost?.boxShadow || 'none',
        hover: currentTheme.buttons?.ghost?.hover,
        active: currentTheme.buttons?.ghost?.active,
      },
      link: {
        background: currentTheme.buttons?.link?.background || 'transparent',
        color:
          currentTheme.buttons?.link?.color ||
          `hsl(${currentTheme.colors.primary})`,
        border: currentTheme.buttons?.link?.border || 'none',
        borderStyle: currentTheme.buttons?.link?.borderStyle || 'solid',
        borderWidth: currentTheme.buttons?.link?.borderWidth || '0',
        borderColor: currentTheme.buttons?.link?.borderColor || 'transparent',
        borderRadius: currentTheme.buttons?.link?.borderRadius || '0',
        boxShadow: currentTheme.buttons?.link?.boxShadow || 'none',
        hover: currentTheme.buttons?.link?.hover,
        active: currentTheme.buttons?.link?.active,
      },
    },
  };
}
