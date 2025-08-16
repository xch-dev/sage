import { useTheme } from '@/contexts/ThemeContext';

export function useThemeFonts() {
  const { currentTheme } = useTheme();

  // Provide fallback values if currentTheme is null
  if (!currentTheme) {
    return {
      // Font families
      sans: 'Inter, system-ui, sans-serif',
      serif: 'Georgia, serif',
      mono: 'Courier New, Monaco, Consolas, monospace',
      heading: 'Inter, system-ui, sans-serif',
      body: 'Inter, system-ui, sans-serif',

      // CSS variable references (for use in Tailwind classes)
      cssVars: {
        sans: 'var(--font-sans)',
        serif: 'var(--font-serif)',
        mono: 'var(--font-mono)',
        heading: 'var(--font-heading)',
        body: 'var(--font-body)',
      },

      // Inline style objects for direct application
      styles: {
        sans: { fontFamily: 'Inter, system-ui, sans-serif' },
        serif: { fontFamily: 'Georgia, serif' },
        mono: { fontFamily: 'Courier New, Monaco, Consolas, monospace' },
        heading: { fontFamily: 'Inter, system-ui, sans-serif' },
        body: { fontFamily: 'Inter, system-ui, sans-serif' },
      },
    };
  }

  return {
    // Font families
    sans: currentTheme.fonts.sans,
    serif: currentTheme.fonts.serif,
    mono: currentTheme.fonts.mono,
    heading: currentTheme.fonts.heading,
    body: currentTheme.fonts.body,

    // CSS variable references (for use in Tailwind classes)
    cssVars: {
      sans: 'var(--font-sans)',
      serif: 'var(--font-serif)',
      mono: 'var(--font-mono)',
      heading: 'var(--font-heading)',
      body: 'var(--font-body)',
    },

    // Inline style objects for direct application
    styles: {
      sans: { fontFamily: currentTheme.fonts.sans },
      serif: { fontFamily: currentTheme.fonts.serif },
      mono: { fontFamily: currentTheme.fonts.mono },
      heading: { fontFamily: currentTheme.fonts.heading },
      body: { fontFamily: currentTheme.fonts.body },
    },
  };
}
