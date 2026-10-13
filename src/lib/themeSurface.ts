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
// background, and never to white under white text.
const MIN_CONTRAST = 3;
const MIN_ALPHA = 0.9;

export function getReadableSurfaceColor(): string {
  const fallback = 'var(--secondary)';
  if (typeof document === 'undefined') return fallback;
  const root = getComputedStyle(document.documentElement);
  const read = (name: string) => root.getPropertyValue(name).trim();
  const foreground = toRgba(read('--foreground'));
  if (!foreground) return fallback;

  for (const name of ['--secondary', '--popover', '--card', '--background']) {
    const surface = toRgba(read(name));
    if (
      surface &&
      surface.a >= MIN_ALPHA &&
      contrastRatio(surface, foreground) >= MIN_CONTRAST
    ) {
      return `var(${name})`;
    }
  }

  return relativeLuminance(foreground) < 0.5 ? '#ffffff' : '#1d2530';
}

interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

/** A theme value as RGBA, through the browser's own color parser; null if it is not a color. */
function toRgba(value: string): Rgba | null {
  if (!value) return null;
  // Theme variables may hold bare HSL channels, e.g. "222 47% 11%".
  const css = /^-?[\d.]+(deg|rad|turn)?\s+[\d.]+%\s+[\d.]+%/.test(value)
    ? `hsl(${value})`
    : value;
  const probe = document.createElement('span');
  probe.style.color = css;
  if (!probe.style.color) return null;
  probe.style.display = 'none';
  document.body.appendChild(probe);
  const computed = getComputedStyle(probe).color;
  probe.remove();
  const numbers = computed.match(/[\d.]+/g)?.map(Number) ?? [];
  if (numbers.length < 3) return null;
  // "color(srgb r g b / a)" carries channels as 0..1; "rgb(r, g, b)" as 0..255.
  const unit = computed.startsWith('color(') ? 255 : 1;
  return {
    r: numbers[0] * unit,
    g: numbers[1] * unit,
    b: numbers[2] * unit,
    a: numbers.length > 3 ? numbers[3] : 1,
  };
}

function relativeLuminance({ r, g, b }: Rgba): number {
  const channel = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrastRatio(a: Rgba, b: Rgba): number {
  const [hi, lo] = [relativeLuminance(a), relativeLuminance(b)].sort(
    (x, y) => y - x,
  );
  return (hi + 0.05) / (lo + 0.05);
}
