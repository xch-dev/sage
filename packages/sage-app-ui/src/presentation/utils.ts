import { resolveReadableSurface } from '../color';

// Modal tints may stay translucent: the alpha below is applied on top of the
// surface's own, and modals sit over a dimmed backdrop, so a half-opaque theme
// surface still reads as a panel. Contrast with peer rows in the wallet, which
// need a near-opaque surface to hide the delete button underneath.
const MIN_ALPHA = 0.5;

export function resolveBackgroundTintWithAlpha(alpha: number = 0.85): string {
  // `--popover` (the theme's own floating-surface color) is tried last, after
  // the surfaces modals have always used.
  const tint = resolveReadableSurface({
    candidates: [
      '--background',
      '--secondary',
      '--muted',
      '--card',
      '--popover',
    ],
    minAlpha: MIN_ALPHA,
  });

  return `color-mix(in srgb, ${tint} ${alpha * 100}%, transparent)`;
}
