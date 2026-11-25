// @vitest-environment jsdom

import BigNumber from 'bignumber.js';
import { describe, expect, it } from 'vitest';
import { formatCompactNumber, formatRelativeTime } from './i18n';

describe('formatCompactNumber', () => {
  it('abbreviates large values', () => {
    expect(formatCompactNumber(1500)).toBe('1.5K');
    expect(formatCompactNumber(new BigNumber('1234567'))).toBe('1.23M');
  });

  it('rounds ordinary values to two decimals', () => {
    expect(formatCompactNumber('12.3456')).toBe('12.35');
  });

  it('keeps significant digits for small fractions instead of rounding to zero', () => {
    expect(formatCompactNumber(0.000123456)).toBe('0.000123');
    expect(formatCompactNumber(0)).toBe('0');
  });
});

describe('formatRelativeTime', () => {
  const now = Date.UTC(2026, 0, 10, 12, 0, 0);
  const seconds = (ms: number) => Math.floor(ms / 1000);

  it('formats future times', () => {
    expect(formatRelativeTime(seconds(now) + 3 * 86400, now)).toBe('in 3d');
    expect(formatRelativeTime(seconds(now) + 90, now)).toBe('in 1m');
  });

  it('formats past times', () => {
    expect(formatRelativeTime(seconds(now) - 2 * 3600, now)).toBe('2h ago');
    expect(formatRelativeTime(seconds(now) - 45 * 86400, now)).toBe('1mo ago');
    expect(formatRelativeTime(seconds(now) - 400 * 86400, now)).toBe('1y ago');
  });

  it('uses seconds under a minute', () => {
    expect(formatRelativeTime(seconds(now) - 5, now)).toBe('5s ago');
  });
});
