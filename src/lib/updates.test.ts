import { afterEach, describe, expect, it, vi } from 'vitest';
import { sageLatestReleaseApiUrl } from './urls';
import { fetchLatestRelease, isNewerVersion, parseVersion } from './updates';

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('parseVersion', () => {
  it('parses versions with and without a leading v', () => {
    expect(parseVersion('v0.13.1')).toEqual({
      numbers: [0, 13, 1],
      prerelease: null,
    });
    expect(parseVersion('1.2.3-rc.1')).toEqual({
      numbers: [1, 2, 3],
      prerelease: 'rc.1',
    });
  });

  it('rejects anything that is not a version', () => {
    expect(parseVersion('latest')).toBeNull();
    expect(parseVersion('1.2')).toBeNull();
    expect(parseVersion('v1.2.3/../evil')).toBeNull();
  });
});

describe('isNewerVersion', () => {
  it('compares each part numerically', () => {
    expect(isNewerVersion('0.14.0', '0.13.1')).toBe(true);
    expect(isNewerVersion('0.13.10', '0.13.9')).toBe(true);
    expect(isNewerVersion('1.0.0', '0.99.99')).toBe(true);
    expect(isNewerVersion('0.13.1', '0.13.1')).toBe(false);
    expect(isNewerVersion('0.13.0', '0.13.1')).toBe(false);
  });

  it('treats a release as newer than its prerelease', () => {
    expect(isNewerVersion('0.14.0', '0.14.0-rc.1')).toBe(true);
    expect(isNewerVersion('0.14.0-rc.2', '0.14.0-rc.1')).toBe(false);
    expect(isNewerVersion('0.14.0-rc.1', '0.14.0')).toBe(false);
  });

  it('never reports an update for an unparseable version', () => {
    expect(isNewerVersion('garbage', '0.13.1')).toBe(false);
    expect(isNewerVersion('0.14.0', 'garbage')).toBe(false);
  });
});

describe('fetchLatestRelease', () => {
  it('returns the version and builds the release page from the tag', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          tag_name: 'v0.14.0',
          html_url: 'https://example.com/elsewhere',
        }),
      ),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(fetchLatestRelease()).resolves.toEqual({
      version: '0.14.0',
      url: 'https://github.com/xch-dev/sage/releases/tag/v0.14.0',
    });
    expect(fetchMock).toHaveBeenCalledWith(
      sageLatestReleaseApiUrl,
      expect.anything(),
    );
  });

  it('rejects a failed response or a malformed tag', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(new Response('{}', { status: 403 })),
    );
    await expect(fetchLatestRelease()).rejects.toThrow('403');

    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          new Response(JSON.stringify({ tag_name: 'nightly' })),
        ),
    );
    await expect(fetchLatestRelease()).rejects.toThrow('unexpected tag');
  });

  it('gives up on a request that hangs', async () => {
    vi.useFakeTimers();
    vi.stubGlobal(
      'fetch',
      vi.fn(
        (_url: string, init: RequestInit) =>
          new Promise((_resolve, reject) => {
            init.signal?.addEventListener('abort', () =>
              reject(new DOMException('Aborted', 'AbortError')),
            );
          }),
      ),
    );

    const result = expect(fetchLatestRelease()).rejects.toThrow('Aborted');
    await vi.advanceTimersByTimeAsync(15_000);
    await result;
  });
});
