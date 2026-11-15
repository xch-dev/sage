import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { formatFiatPrice } from './utils';

beforeEach(() => {
  vi.stubGlobal('navigator', { language: 'en-US' });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('formatFiatPrice', () => {
  it.each([
    [0, '$0.00'],
    [0.00005, '< $0.0001'],
    [0.0001, '$0.00010'],
    [0.00345, '$0.00345'],
    [0.5, '$0.50'],
    [0.999, '$0.999'],
    // Rounds up to 1, so it's shown like 1.
    [0.9996, '$1.00'],
    [1, '$1.00'],
    [1234.5, '$1,234.50'],
  ])('formats %d USD as %s', (price, expected) => {
    expect(formatFiatPrice(price, 'USD')).toBe(expected);
  });

  it('formats in the given currency', () => {
    expect(formatFiatPrice(12.5, 'EUR')).toBe('€12.50');
    expect(formatFiatPrice(0.5, 'EUR')).toBe('€0.50');
  });

  it("uses the currency's own decimal places", () => {
    expect(formatFiatPrice(1234.5, 'JPY')).toBe('¥1,235');
    expect(formatFiatPrice(0, 'JPY')).toBe('¥0');
    expect(formatFiatPrice(0.5, 'JPY')).toBe('¥0.50');
  });
});
