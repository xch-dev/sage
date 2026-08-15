import { commands } from '@/bindings';
import type { Theme } from 'theme-o-rama';

const SAGE_THEME_VAR_NAMES = [
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
  '--ring',
  '--radius',
];

function normalizeCssVarValue(value: string): string {
  const trimmed = value.trim();

  // unwrap hsl(...) → inner value (for Tailwind hsl(var(--...)))
  const match = trimmed.match(/^hsl\((.*)\)$/i);
  if (match) {
    return match[1].trim();
  }

  return trimmed;
}

function collectResolvedThemeCssVars(): Record<string, string> {
  const style = getComputedStyle(document.documentElement);

  return Object.fromEntries(
    SAGE_THEME_VAR_NAMES.map((name) => {
      const raw = style.getPropertyValue(name);
      return [name, normalizeCssVarValue(raw)];
    }),
  );
}

export async function publishEnvironmentThemeToRust(
  currentTheme: Theme,
): Promise<void> {
  await commands.appsSetEnvironmentTheme({
    name: currentTheme.name,
    displayName: currentTheme.displayName,
    mostLike: currentTheme.mostLike ?? null,
    inherits: currentTheme.inherits ?? null,
    cssVars: collectResolvedThemeCssVars(),
  });
}
