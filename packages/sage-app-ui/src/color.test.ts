// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type ColorModule = typeof import('./color');

// The module caches its canvas, so each test gets a fresh copy.
async function load(getContext: () => unknown = () => null) {
  vi.resetModules();
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(
    getContext as HTMLCanvasElement['getContext'],
  );
  return (await import('./color')) as ColorModule;
}

function setTheme(vars: Record<string, string>) {
  for (const [name, value] of Object.entries(vars)) {
    document.documentElement.style.setProperty(name, value);
  }
}

beforeEach(() => {
  document.documentElement.removeAttribute('style');
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('parseSerializedColor', () => {
  it('reads rgb() and rgba() in either syntax', async () => {
    const { parseSerializedColor } = await load();
    expect(parseSerializedColor('rgb(1, 2, 3)')).toEqual({
      r: 1,
      g: 2,
      b: 3,
      a: 1,
    });
    expect(parseSerializedColor('rgba(10, 20, 30, 0.5)')).toEqual({
      r: 10,
      g: 20,
      b: 30,
      a: 0.5,
    });
    expect(parseSerializedColor('rgb(10 20 30 / 50%)')).toEqual({
      r: 10,
      g: 20,
      b: 30,
      a: 0.5,
    });
  });

  it('reads exponent notation', async () => {
    const { parseSerializedColor } = await load();
    expect(parseSerializedColor('rgba(255, 1e-5, 2.5e1, 1e-1)')).toEqual({
      r: 255,
      g: 1e-5,
      b: 25,
      a: 0.1,
    });
  });

  it('scales color(srgb) channels to 0..255', async () => {
    const { parseSerializedColor } = await load();
    expect(parseSerializedColor('color(srgb 1 0.5 0 / 0.25)')).toEqual({
      r: 255,
      g: 127.5,
      b: 0,
      a: 0.25,
    });
  });

  it('rejects color spaces it cannot convert', async () => {
    const { parseSerializedColor } = await load();
    for (const value of [
      'oklch(0.7 0.1 200)',
      'lab(50% 20 -30)',
      'color(display-p3 1 0 0)',
      'red',
      'rgb(1, 2)',
      '',
    ]) {
      expect(parseSerializedColor(value)).toBeNull();
    }
  });
});

describe('toCssColor', () => {
  it('wraps bare HSL channels and keeps real colors', async () => {
    const { toCssColor } = await load();
    expect(toCssColor(' 222 47% 11% ')).toBe('hsl(222 47% 11%)');
    expect(toCssColor('#f5f5f5')).toBe('#f5f5f5');
    expect(toCssColor('rgb(1 2 3)')).toBe('rgb(1 2 3)');
  });

  it('rejects values that are not colors', async () => {
    const { toCssColor } = await load();
    expect(toCssColor('')).toBeNull();
    expect(toCssColor('not a color')).toBeNull();
  });
});

describe('parseColor', () => {
  it('falls back to the computed style without a canvas', async () => {
    const { parseColor } = await load();
    expect(parseColor('0 0% 100%')).toEqual({ r: 255, g: 255, b: 255, a: 1 });
    expect(parseColor('#f5f5f5')).toEqual({ r: 245, g: 245, b: 245, a: 1 });
    // jsdom keeps oklch() as is, which the fallback cannot read.
    expect(parseColor('oklch(0.5 0.1 200)')).toBeNull();
  });

  it('reads the pixel the browser draws', async () => {
    // A canvas that "draws" oklch(0.5 0.1 200) as rgb(0, 115, 128).
    let fill = '';
    const context = {
      globalCompositeOperation: '',
      get fillStyle() {
        return fill;
      },
      set fillStyle(value: string) {
        fill = value;
      },
      fillRect: () => undefined,
      getImageData: () => ({
        data:
          fill === 'oklch(0.5 0.1 200)'
            ? [0, 115, 128, 255]
            : [255, 255, 255, 255],
      }),
    };
    const { parseColor } = await load(() => context);
    expect(parseColor('oklch(0.5 0.1 200)')).toEqual({
      r: 0,
      g: 115,
      b: 128,
      a: 1,
    });
  });

  it('falls back to the computed style when the canvas rejects a color', async () => {
    // A canvas that keeps its previous fill style, like a real one does for
    // a color it cannot parse.
    let fill = '#000000';
    const context = {
      globalCompositeOperation: '',
      get fillStyle() {
        return fill;
      },
      set fillStyle(value: string) {
        if (value.startsWith('#')) fill = value;
      },
      fillRect: () => undefined,
      getImageData: () => ({
        data: fill === '#000000' ? [0, 0, 0, 255] : [255, 255, 255, 255],
      }),
    };
    const { parseColor } = await load(() => context);
    expect(parseColor('rgb(1 2 3)')).toEqual({ r: 1, g: 2, b: 3, a: 1 });
  });
});

describe('contrastRatio', () => {
  it('matches the WCAG extremes', async () => {
    const { contrastRatio } = await load();
    const black = { r: 0, g: 0, b: 0, a: 1 };
    const white = { r: 255, g: 255, b: 255, a: 1 };
    expect(contrastRatio(black, white)).toBeCloseTo(21);
    expect(contrastRatio(white, white)).toBeCloseTo(1);
  });
});

describe('resolveReadableSurface', () => {
  const candidates = ['--secondary', '--popover', '--background'];

  it('keeps the first readable surface', async () => {
    const { resolveReadableSurface } = await load();
    setTheme({
      '--foreground': '#111111',
      '--secondary': '#f5f5f5',
      '--popover': '#ffffff',
    });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      '#f5f5f5',
    );
  });

  it('skips a light surface under light text', async () => {
    const { resolveReadableSurface } = await load();
    setTheme({
      '--foreground': '#e0f7ff',
      '--secondary': '#f5f5f5',
      '--popover': '#0b1f33',
    });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      '#0b1f33',
    );
  });

  it('returns bare HSL channels as a valid color', async () => {
    const { resolveReadableSurface } = await load();
    setTheme({ '--foreground': '0 0% 100%', '--secondary': '222 47% 11%' });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      'hsl(222 47% 11%)',
    );
  });

  it('skips transparent and too translucent surfaces', async () => {
    const { resolveReadableSurface } = await load();
    setTheme({
      '--foreground': '#ffffff',
      '--secondary': 'transparent',
      '--popover': 'rgba(0, 0, 0, 0.6)',
      '--background': '#000000',
    });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      '#000000',
    );
    expect(resolveReadableSurface({ candidates, minAlpha: 0.5 })).toBe(
      'rgba(0, 0, 0, 0.6)',
    );
  });

  it('falls back by the text color when nothing is readable', async () => {
    const { resolveReadableSurface } = await load();
    setTheme({ '--foreground': '#e0f7ff', '--secondary': '#f5f5f5' });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      '#1d2530',
    );
    setTheme({ '--foreground': '#111111', '--secondary': '#222222' });
    expect(resolveReadableSurface({ candidates, minAlpha: 0.9 })).toBe(
      '#ffffff',
    );
  });
});
