import { useTheme } from '@/contexts/ThemeContext';

export function useThemeFonts() {
    const { currentTheme } = useTheme();

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
