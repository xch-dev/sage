import { describe, expect, it } from 'vitest';
import BigNumber from 'bignumber.js';
import {
  toMojos,
  fromMojos,
  toDecimal,
  toAddress,
  addressInfo,
  isValidAddress,
  puzzleHash,
  toHex,
  isHex,
  formatAddress,
  formatUsdPrice,
  isValidUrl,
  isValidAssetId,
  deepMerge,
  decodeHexMessage,
} from './utils';

// A known puzzle-hash for round-trip tests (32 bytes = 64 hex chars)
const KNOWN_PUZZLE_HASH =
  'a0b0c0d0e0f0a1b1c1d1e1f1a2b2c2d2e2f2a3b3c3d3e3f3a4b4c4d4e4f4a5b5';

describe('toMojos / fromMojos / toDecimal', () => {
  it('converts XCH amount with 12 decimal precision', () => {
    expect(toMojos('1', 12)).toBe('1000000000000');
    expect(toMojos('0.5', 12)).toBe('500000000000');
    expect(toDecimal('1000000000000', 12)).toBe('1');
    expect(toDecimal('500000000000', 12)).toBe('0.5');
  });

  it('converts CAT amount with 3 decimal precision', () => {
    expect(toMojos('1', 3)).toBe('1000');
    expect(toMojos('0.5', 3)).toBe('500');
    expect(toDecimal('1000', 3)).toBe('1');
    expect(toDecimal('500', 3)).toBe('0.5');
  });

  it('handles zero', () => {
    expect(toMojos('0', 12)).toBe('0');
    expect(toDecimal('0', 12)).toBe('0');
    expect(fromMojos(0, 12).isEqualTo(0)).toBe(true);
  });

  it('handles large amounts without scientific notation', () => {
    const result = toMojos('1000000', 12);
    expect(result).toBe('1000000000000000000');
    expect(result).not.toContain('e');
  });

  it('handles very small amounts', () => {
    expect(toMojos('0.000000000001', 12)).toBe('1');
    // BigNumber.toString() uses exponential notation for very small values
    expect(fromMojos('1', 12).isEqualTo('0.000000000001')).toBe(true);
  });

  it('fromMojos returns a BigNumber', () => {
    const result = fromMojos('1000', 3);
    expect(result).toBeInstanceOf(BigNumber);
    expect(result.toString()).toBe('1');
  });
});

describe('toAddress / addressInfo / puzzleHash / isValidAddress', () => {
  it('round-trips puzzle hash → address → puzzle hash (xch)', () => {
    const addr = toAddress(KNOWN_PUZZLE_HASH, 'xch');
    expect(addr.startsWith('xch')).toBe(true);
    const info = addressInfo(addr);
    expect(info.puzzleHash).toBe(KNOWN_PUZZLE_HASH);
    expect(info.prefix).toBe('xch');
  });

  it('round-trips with txch prefix', () => {
    const addr = toAddress(KNOWN_PUZZLE_HASH, 'txch');
    expect(addr.startsWith('txch')).toBe(true);
    const info = addressInfo(addr);
    expect(info.puzzleHash).toBe(KNOWN_PUZZLE_HASH);
    expect(info.prefix).toBe('txch');
  });

  it('puzzleHash extracts from address', () => {
    const addr = toAddress(KNOWN_PUZZLE_HASH, 'xch');
    expect(puzzleHash(addr)).toBe(KNOWN_PUZZLE_HASH);
  });

  it('isValidAddress accepts valid xch address', () => {
    const addr = toAddress(KNOWN_PUZZLE_HASH, 'xch');
    expect(isValidAddress(addr, 'xch')).toBe(true);
  });

  it('isValidAddress rejects wrong prefix', () => {
    const addr = toAddress(KNOWN_PUZZLE_HASH, 'xch');
    expect(isValidAddress(addr, 'txch')).toBe(false);
  });

  it('isValidAddress rejects garbage input', () => {
    expect(isValidAddress('not-an-address', 'xch')).toBe(false);
    expect(isValidAddress('', 'xch')).toBe(false);
  });

  it('handles 0x-prefixed puzzle hash', () => {
    const addr = toAddress('0x' + KNOWN_PUZZLE_HASH, 'xch');
    const info = addressInfo(addr);
    expect(info.puzzleHash).toBe(KNOWN_PUZZLE_HASH);
  });
});

describe('toHex / isHex', () => {
  it('converts bytes to hex', () => {
    expect(toHex(new Uint8Array([0, 1, 255]))).toBe('0001ff');
    expect(toHex(new Uint8Array([]))).toBe('');
  });

  it('isHex validates hex strings', () => {
    expect(isHex('abcdef0123456789')).toBe(true);
    expect(isHex('ABCDEF')).toBe(true);
    expect(isHex('0xabcdef')).toBe(true);
  });

  it('isHex rejects non-hex strings', () => {
    expect(isHex('xyz')).toBe(false);
    expect(isHex('')).toBe(false);
    expect(isHex('0xgh')).toBe(false);
  });
});

describe('decodeHexMessage', () => {
  it('decodes hex-encoded ASCII message', () => {
    expect(decodeHexMessage('48656c6c6f')).toBe('Hello');
  });

  it('handles 0x prefix', () => {
    expect(decodeHexMessage('0x48656c6c6f')).toBe('Hello');
  });
});

describe('formatAddress', () => {
  it('truncates long addresses', () => {
    const addr = 'abcdefghijklmnopqrstuvwxyz';
    expect(formatAddress(addr, 4, 4)).toBe('abcd...wxyz');
  });

  it('returns full address when shorter than chars + trailingChars', () => {
    expect(formatAddress('short', 8, 8)).toBe('short');
  });

  it('strips 0x prefix before truncating', () => {
    // 0x prefix is stripped, remaining "abcdefghijklmnop" is 16 chars
    // which equals 8 + 8, so it returns the full original address
    const addr = '0xabcdefghijklmnop';
    expect(formatAddress(addr)).toBe(addr);

    // With a longer address the 0x is stripped and the rest is truncated
    const longAddr = '0x' + 'a'.repeat(30);
    const result = formatAddress(longAddr);
    expect(result.startsWith('aaaaaaaa...')).toBe(true);
  });

  it('uses default chars=8', () => {
    const addr = 'a'.repeat(30);
    const result = formatAddress(addr);
    expect(result).toBe('aaaaaaaa...aaaaaaaa');
  });
});

describe('formatUsdPrice', () => {
  it('formats sub-cent prices', () => {
    expect(formatUsdPrice(0.001)).toBe('< 0.01¢');
    expect(formatUsdPrice(0.009)).toBe('< 0.01¢');
  });

  it('formats cent-level prices', () => {
    expect(formatUsdPrice(0.5)).toBe('50.00¢');
    expect(formatUsdPrice(0.01)).toBe('1.00¢');
    expect(formatUsdPrice(0.99)).toBe('99.00¢');
  });

  it('formats dollar prices', () => {
    expect(formatUsdPrice(1)).toBe('$1.00');
    expect(formatUsdPrice(42.5)).toBe('$42.50');
    expect(formatUsdPrice(1000)).toBe('$1000.00');
  });
});

describe('isValidUrl', () => {
  it('accepts https URLs', () => {
    expect(isValidUrl('https://example.com')).toBe(true);
    expect(isValidUrl('https://example.com/path?q=1')).toBe(true);
  });

  it('accepts http URLs', () => {
    expect(isValidUrl('http://example.com')).toBe(true);
  });

  it('rejects localhost', () => {
    expect(isValidUrl('http://localhost')).toBe(false);
    expect(isValidUrl('https://localhost:3000')).toBe(false);
  });

  it('rejects 127.0.0.1', () => {
    expect(isValidUrl('http://127.0.0.1')).toBe(false);
    expect(isValidUrl('https://127.0.0.1:8080')).toBe(false);
  });

  it('rejects file:// protocol', () => {
    expect(isValidUrl('file:///etc/passwd')).toBeFalsy();
  });

  it('rejects ftp:// protocol', () => {
    expect(isValidUrl('ftp://example.com')).toBeFalsy();
  });

  it('rejects garbage strings', () => {
    expect(isValidUrl('not-a-url')).toBeFalsy();
    expect(isValidUrl('')).toBeFalsy();
  });
});

describe('isValidAssetId', () => {
  it('accepts valid 64-char hex', () => {
    expect(isValidAssetId('a'.repeat(64))).toBe(true);
    expect(isValidAssetId('0123456789abcdefABCDEF' + 'a'.repeat(42))).toBe(
      true,
    );
  });

  it('rejects too-short strings', () => {
    expect(isValidAssetId('abc')).toBe(false);
    expect(isValidAssetId('a'.repeat(63))).toBe(false);
  });

  it('rejects too-long strings', () => {
    expect(isValidAssetId('a'.repeat(65))).toBe(false);
  });

  it('rejects non-hex characters', () => {
    expect(isValidAssetId('g'.repeat(64))).toBe(false);
    expect(isValidAssetId('z' + 'a'.repeat(63))).toBe(false);
  });
});

describe('deepMerge', () => {
  it('merges nested objects', () => {
    const target = { a: 1, b: { c: 2, d: 3 } };
    const source = { b: { c: 10 } };
    const result = deepMerge(target, source);
    expect(result).toEqual({ a: 1, b: { c: 10, d: 3 } });
  });

  it('does not mutate the original target', () => {
    const target = { a: { b: 1 } };
    const source = { a: { b: 2 } };
    const result = deepMerge(target, source);
    expect(result.a.b).toBe(2);
    expect(target.a.b).toBe(1);
  });

  it('replaces arrays instead of merging them', () => {
    const target = { arr: [1, 2, 3] };
    const source = { arr: [4, 5] };
    const result = deepMerge(target, source);
    expect(result.arr).toEqual([4, 5]);
  });

  it('adds new keys from source', () => {
    const target = { a: 1 } as Record<string, unknown>;
    const source = { b: 2 };
    const result = deepMerge(target, source);
    expect(result).toEqual({ a: 1, b: 2 });
  });
});
