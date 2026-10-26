import { resolveReadableSurface } from 'sage-app-ui/color';
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

// A solid surface the theme's text can be read on, for rows that must hide what
// is behind them (the peer list's swipe-to-delete rows sit over a red trash
// can). `--secondary` is kept whenever it is readable, so most themes look the
// same; a theme whose secondary is light under light text (the awizard NFT
// theme: #f5f5f5 under #e0f7ff) falls through to its popover, card or
// background. The alpha floor is higher than the modal tint's in sage-app-ui
// because a translucent row would let the trash can show through.
export function getReadableSurfaceColor(): string {
  return resolveReadableSurface({
    candidates: ['--secondary', '--popover', '--card', '--background'],
    minAlpha: 0.9,
  });
}
