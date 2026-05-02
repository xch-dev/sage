import type * as Generated from './generated-types';

export const sageThemeVars = {
  background: '--background',
  foreground: '--foreground',
  card: '--card',
  cardForeground: '--card-foreground',
  popover: '--popover',
  popoverForeground: '--popover-foreground',
  primary: '--primary',
  primaryForeground: '--primary-foreground',
  secondary: '--secondary',
  secondaryForeground: '--secondary-foreground',
  muted: '--muted',
  mutedForeground: '--muted-foreground',
  accent: '--accent',
  accentForeground: '--accent-foreground',
  destructive: '--destructive',
  destructiveForeground: '--destructive-foreground',
  border: '--border',
  input: '--input',
  ring: '--ring',
  radius: '--radius',
} as const;

export type SageThemeVar = keyof typeof sageThemeVars;

export function cssVar(name: SageThemeVar): string {
  return `var(${sageThemeVars[name]})`;
}

export function rawCssVar(name: SageThemeVar): string {
  return `var(${sageThemeVars[name]})`;
}

function normalizeCssVarValue(value: string): string {
  const trimmed = value.trim();

  const hslTuple =
    /^-?\d+(\.\d+)?(?:deg|rad|turn)?\s+-?\d+(\.\d+)?%\s+-?\d+(\.\d+)?%(?:\s*\/\s*(?:\d+(\.\d+)?%?|\.\d+))?$/;

  if (hslTuple.test(trimmed)) {
    return `hsl(${trimmed})`;
  }

  return trimmed;
}

export function applySageThemeCssVars(
  theme: Generated.EnvironmentThemeView,
): void {
  let el = document.getElementById('sage-environment-theme-vars');

  if (!el) {
    el = document.createElement('style');
    el.id = 'sage-environment-theme-vars';
    document.head.appendChild(el);
  }

  const vars = Object.entries(theme.cssVars)
    .filter((entry): entry is [string, string] => {
      const [key, value] = entry;

      return (
        key.startsWith('--') &&
        typeof value === 'string' &&
        value.trim().length > 0
      );
    })
    .map(([key, value]) => `${key}: ${normalizeCssVarValue(value)};`)
    .join(' ');

  el.textContent = vars ? `:root { ${vars} }` : '';
}

export function clearSageThemeCssVars(): void {
  document.getElementById('sage-environment-theme-vars')?.remove();
}
