// theme-o-rama 0.4 no longer persists the selected theme itself, so Sage does.
// These are the keys theme-o-rama 0.2 used, so existing selections carry over.
const THEME_STORAGE_KEY = 'theme';
const LAST_USED_NON_CORE_THEME_KEY = 'theme-o-rama-last-used-non-core-theme';

const CORE_THEMES = new Set(['light', 'dark']);

export function getStoredTheme(): string | null {
  return localStorage.getItem(THEME_STORAGE_KEY);
}

export function getLastUsedNonCoreTheme(): string | null {
  return localStorage.getItem(LAST_USED_NON_CORE_THEME_KEY);
}

export function storeTheme(themeName: string) {
  localStorage.setItem(THEME_STORAGE_KEY, themeName);

  if (!CORE_THEMES.has(themeName)) {
    localStorage.setItem(LAST_USED_NON_CORE_THEME_KEY, themeName);
  }
}
