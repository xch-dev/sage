export function resolveBackgroundTintWithAlpha(alpha: number = 0.85): string {
  const tint = resolveBackgroundTint();

  return colorWithAlpha(tint, alpha);
}

// A panel tint must be a surface the theme's text can be read on. Themes with a
// background image set `--background` to transparent, and the next candidate can
// be a light accent: the awizard theme's `--secondary` is #f5f5f5 under #e0f7ff
// text, which made every approval modal white with unreadable text. So each
// candidate is checked against `--foreground`, and `--popover` (the theme's own
// floating-surface color) is tried before the hard fallback.
const MIN_CONTRAST = 3;
const MIN_ALPHA = 0.5;

function resolveBackgroundTint(): string {
  const root = getComputedStyle(document.documentElement);
  const read = (name: string) => root.getPropertyValue(name).trim();

  const candidates = [
    read('--background'),
    read('--secondary'),
    read('--muted'),
    read('--card'),
    read('--popover'),
  ];
  const foreground = toRgba(read('--foreground'));

  const readable = candidates.find((value) => {
    if (
      value.length === 0 ||
      value === 'transparent' ||
      value === 'rgba(0, 0, 0, 0)'
    ) {
      return false;
    }
    const surface = toRgba(value);
    // A color the browser cannot read keeps the old behavior: use it as is.
    if (!surface || !foreground) return true;
    return (
      surface.a >= MIN_ALPHA &&
      contrastRatio(surface, foreground) >= MIN_CONTRAST
    );
  });

  if (readable) return readable;

  // Nothing in the theme is readable under its own text: never default to white
  // under light text, or to a dark tint under dark text.
  return foreground && relativeLuminance(foreground) < 0.5
    ? '#ffffff'
    : '#1d2530';
}

interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

/** A theme value as RGBA, through the browser's own color parser; null if it is not a color. */
function toRgba(value: string): Rgba | null {
  if (!value || typeof document === 'undefined') return null;
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

function colorWithAlpha(color: string, alpha: number): string {
  if (color.startsWith('#')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  if (color.startsWith('rgb') || color.startsWith('hsl')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  // HSL channel format, e.g. "222 47% 11%"
  return `hsl(${color} / ${alpha})`;
}
