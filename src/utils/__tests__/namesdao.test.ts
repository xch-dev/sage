import { describe, expect, it, beforeEach } from 'vitest';
import { isValidXchName, formatXchName, resolveXchName } from '../namesdao';

describe('NamesdaoResolver', () => {
  describe('isValidXchName', () => {
    it('validates correct .xch names', () => {
      expect(isValidXchName('test.xch')).toBe(true);
      expect(isValidXchName('test123.xch')).toBe(true);
      expect(isValidXchName('123test.xch')).toBe(true);
      expect(isValidXchName('TEST.XCH')).toBe(true);
      expect(isValidXchName('test-name.xch')).toBe(true);
      expect(isValidXchName('test_name.xch')).toBe(true);
    });

    it('rejects invalid .xch names', () => {
      expect(isValidXchName('')).toBe(false);
      expect(isValidXchName('test')).toBe(false);
      expect(isValidXchName('test.')).toBe(false);
      expect(isValidXchName('test.xyz')).toBe(false);
      expect(isValidXchName('.xch')).toBe(true);
    });
  });

  describe('formatXchName', () => {
    it('formats names correctly', () => {
      expect(formatXchName('test')).toBe('test.xch');
      expect(formatXchName('TEST')).toBe('test.xch');
      expect(formatXchName('test.xch')).toBe('test.xch');
      expect(formatXchName('TEST.XCH')).toBe('test.xch');
      expect(formatXchName(' test ')).toBe('test.xch');
    });
  });

  describe('resolveXchName', () => {
    const mockAddress = 'xch1jhye8dmkhree0zr8t09rlzm9cc82mhuqtp5tlmsj4kuqvs69s2wsl90su4';
    
    beforeEach(() => {
      // Reset fetch mock before each test
      global.fetch = vi.fn();
    });

    it('resolves valid .xch name', async () => {
      global.fetch = vi.fn().mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ address: mockAddress }),
        }),
      );

      const result = await resolveXchName('namesdao.xch');
      expect(result).toBe(mockAddress);
      expect(fetch).toHaveBeenCalledWith(
        'https://namesdaolookup.xchstorage.com/namesdao.json',
      );
    });

    it('returns null for invalid name', async () => {
      const result = await resolveXchName('-----.xch');
      expect(result).toBeNull();
      expect(fetch).not.toHaveBeenCalled();
    });

    it('returns null for 404 response', async () => {
      global.fetch = vi.fn().mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 404,
        }),
      );

      const result = await resolveXchName('-----.xch');
      expect(result).toBeNull();
    });

    it('handles network errors gracefully', async () => {
      global.fetch = vi.fn().mockImplementationOnce(() =>
        Promise.reject(new Error('Network error')),
      );

      const result = await resolveXchName('namesdao.xch');
      expect(result).toBeNull();
    });

    it('caches successful resolutions', async () => {
      global.fetch = vi.fn().mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ address: mockAddress }),
        }),
      );

      // First call should hit the API
      await resolveXchName('namesdao.xch');
      expect(fetch).toHaveBeenCalledTimes(1);

      // Second call should use cache
      const result = await resolveXchName('namesdao.xch');
      expect(result).toBe(mockAddress);
      expect(fetch).toHaveBeenCalledTimes(1);
    });
  });
});
