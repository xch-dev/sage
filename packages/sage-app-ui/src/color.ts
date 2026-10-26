// Theme color helpers shared by the wallet and the built-in apps. This module
// has no dependencies so the wallet can import it as `sage-app-ui/color`
// without pulling in the app SDK.

/** An sRGB color: channels 0..255, alpha 0..1. */
export interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

/**
 * A theme value as a color the browser accepts, or null if it is not one.
 * Theme variables may hold bare HSL channels, e.g. "222 47% 11%", which are
 * only valid inside `hsl(...)`, so those come back wrapped.
 */
export function toCssColor(value: string): string | null {
  const trimmed = value.trim();
  if (!trimmed) return null;
  if (isColor(trimmed)) return trimmed;
  if (!trimmed.includes('(')) {
    const wrapped = `hsl(${trimmed})`;
    if (isColor(wrapped)) return wrapped;
  }
  return null;
}

/**
 * A theme value as sRGB, or null if it is not a color or cannot be read.
 * The browser does the conversion: the color is drawn to a 1x1 canvas and the
 * pixel read back, which handles every syntax it supports (oklch, lab,
 * color(), relative colors, ...). Without a canvas it falls back to the
 * computed style, and only accepts the plain sRGB forms it can parse safely.
 */
export function parseColor(value: string): Rgba | null {
  const css = toCssColor(value);
  if (!css || typeof document === 'undefined') return null;
  return rasterize(css) ?? parseSerializedColor(computedColor(css));
}

const NUMBER = String.raw`[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?`;
const ALPHA = String.raw`(?:\s*[,/]\s*(${NUMBER})(%)?)?`;
const RGB = new RegExp(
  String.raw`^rgba?\(\s*(${NUMBER})(?:\s*,\s*|\s+)(${NUMBER})(?:\s*,\s*|\s+)(${NUMBER})${ALPHA}\s*\)$`,
  'i',
);
const SRGB = new RegExp(
  String.raw`^color\(\s*srgb\s+(${NUMBER})\s+(${NUMBER})\s+(${NUMBER})${ALPHA}\s*\)$`,
  'i',
);

/**
 * Parses a serialized sRGB color: `rgb()`/`rgba()` with 0..255 channels or
 * `color(srgb ...)` with 0..1 channels. Anything else (oklch, lab, other color
 * spaces, keywords) is rejected rather than misread.
 */
export function parseSerializedColor(value: string): Rgba | null {
  const text = value.trim();
  const rgb = RGB.exec(text);
  const srgb = rgb ? null : SRGB.exec(text);
  const match = rgb ?? srgb;
  if (!match) return null;

  const scale = srgb ? 255 : 1;
  const channel = (n: string) => clamp(Number(n) * scale, 0, 255);
  const alpha =
    match[4] === undefined
      ? 1
      : clamp(Number(match[4]) / (match[5] ? 100 : 1), 0, 1);
  const color = {
    r: channel(match[1]),
    g: channel(match[2]),
    b: channel(match[3]),
    a: alpha,
  };
  return Object.values(color).every(Number.isFinite) ? color : null;
}

/** WCAG relative luminance of an sRGB color. */
export function relativeLuminance({ r, g, b }: Rgba): number {
  const channel = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

/** WCAG contrast ratio between two colors, 1..21. */
export function contrastRatio(a: Rgba, b: Rgba): number {
  const [hi, lo] = [relativeLuminance(a), relativeLuminance(b)].sort(
    (x, y) => y - x,
  );
  return (hi + 0.05) / (lo + 0.05);
}

export interface ReadableSurfaceOptions {
  /** Theme variables to try in order, e.g. `['--secondary', '--card']`. */
  candidates: string[];
  /** The lowest alpha a surface may have and still count as one. */
  minAlpha: number;
  /** The lowest contrast against `--foreground`. Defaults to 3:1. */
  minContrast?: number;
  /** The element whose variables are read. Defaults to the document root. */
  root?: Element;
}

const LIGHT_SURFACE = '#ffffff';
const DARK_SURFACE = '#1d2530';

/**
 * The first candidate surface the theme's text can be read on, as a valid CSS
 * color. Themes with a background image set `--background` to transparent, and
 * a theme's light accents can sit under light text (the awizard NFT theme's
 * `--secondary` is #f5f5f5 under #e0f7ff text), so every candidate is checked
 * against `--foreground`. If none passes, the fallback is chosen from the
 * text's own luminance, never white under light text or dark under dark.
 */
export function resolveReadableSurface({
  candidates,
  minAlpha,
  minContrast = 3,
  root,
}: ReadableSurfaceOptions): string {
  if (typeof document === 'undefined') return DARK_SURFACE;

  const style = getComputedStyle(root ?? document.documentElement);
  const foreground = parseColor(style.getPropertyValue('--foreground'));

  for (const name of candidates) {
    const css = toCssColor(style.getPropertyValue(name));
    if (!css || css.toLowerCase() === 'transparent') continue;

    const surface = parseColor(css);
    // A color the browser accepts but that cannot be read back is used as is.
    if (!surface) return css;
    if (surface.a < minAlpha) continue;
    if (!foreground || contrastRatio(surface, foreground) >= minContrast) {
      return css;
    }
  }

  return foreground && relativeLuminance(foreground) < 0.5
    ? LIGHT_SURFACE
    : DARK_SURFACE;
}

function isColor(css: string): boolean {
  if (typeof CSS !== 'undefined' && typeof CSS.supports === 'function') {
    return CSS.supports('color', css);
  }
  if (typeof document === 'undefined') return false;
  const probe = document.createElement('span');
  probe.style.color = css;
  return probe.style.color !== '';
}

let canvasContext: CanvasRenderingContext2D | null | undefined;

function rasterize(css: string): Rgba | null {
  if (canvasContext === undefined) {
    canvasContext = null;
    try {
      const canvas = document.createElement('canvas');
      canvas.width = 1;
      canvas.height = 1;
      canvasContext = canvas.getContext('2d', { willReadFrequently: true });
    } catch {
      // No canvas support (e.g. jsdom): use the computed style instead.
    }
  }
  const context = canvasContext;
  if (!context) return null;

  // The canvas keeps its previous fill style when it rejects a color, so the
  // color is drawn over two different ones: if they disagree, it was rejected.
  const draw = (previous: string) => {
    context.globalCompositeOperation = 'copy';
    context.fillStyle = previous;
    context.fillStyle = css;
    context.fillRect(0, 0, 1, 1);
    return context.getImageData(0, 0, 1, 1).data;
  };
  const [r, g, b, a] = draw('#000000');
  const check = draw('#ffffff');
  if (check[0] !== r || check[1] !== g || check[2] !== b || check[3] !== a) {
    return null;
  }
  return { r, g, b, a: a / 255 };
}

function computedColor(css: string): string {
  const probe = document.createElement('span');
  probe.style.display = 'none';
  probe.style.color = css;
  document.body.appendChild(probe);
  const computed = getComputedStyle(probe).color;
  probe.remove();
  return computed;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
