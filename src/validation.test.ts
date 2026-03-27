import { describe, expect, it } from 'vitest';
import { isValidU32 } from './validation';

describe('isValidU32', () => {
  it('accepts zero', () => {
    expect(isValidU32(0)).toBe(true);
  });

  it('accepts max u32 - 1', () => {
    expect(isValidU32(2 ** 32 - 1)).toBe(true);
  });

  it('rejects 2^32 (overflow)', () => {
    expect(isValidU32(2 ** 32)).toBe(false);
  });

  it('rejects negative numbers', () => {
    expect(isValidU32(-1)).toBe(false);
  });

  it('rejects NaN', () => {
    expect(isValidU32(NaN)).toBe(false);
  });

  it('rejects Infinity', () => {
    expect(isValidU32(Infinity)).toBe(false);
    expect(isValidU32(-Infinity)).toBe(false);
  });

  it('respects custom minimum', () => {
    expect(isValidU32(0, 1)).toBe(false);
    expect(isValidU32(1, 1)).toBe(true);
    expect(isValidU32(100, 50)).toBe(true);
    expect(isValidU32(49, 50)).toBe(false);
  });

  it('accepts typical u32 values', () => {
    expect(isValidU32(1)).toBe(true);
    expect(isValidU32(1000)).toBe(true);
    expect(isValidU32(2147483647)).toBe(true); // max i32
  });
});
