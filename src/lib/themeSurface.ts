import type { CSSProperties } from 'react';
import type { Theme } from 'theme-o-rama';

export function getFloatingSurfaceStyle(
  theme: Theme | null | undefined,
): CSSProperties {
  if (!theme?.backgroundImage) return {};

  const isDark = (theme.inherits ?? theme.mostLike) === 'dark';

  return {
    backgroundColor: isDark
      ? 'rgba(0, 0, 0, 0.75)'
      : 'rgba(255, 255, 255, 0.8)',
  };
}
