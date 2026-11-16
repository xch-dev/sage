// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { hideUpdateToast, showUpdateToast } from '@/components/UpdateToast';
import {
  UPDATE_CHECKS_STORAGE_KEY,
  UPDATE_NOTIFIED_STORAGE_KEY,
  UpdateProvider,
  useUpdates,
} from './UpdateContext';

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: () => Promise.resolve('0.13.1'),
}));

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: () => 'macos',
}));

vi.mock('@/components/UpdateToast', () => ({
  showUpdateToast: vi.fn(),
  hideUpdateToast: vi.fn(),
}));

const fetchMock = vi.fn();

function respondWithTag(tag: string) {
  fetchMock.mockImplementation(() =>
    Promise.resolve(new Response(JSON.stringify({ tag_name: tag }))),
  );
}

function wrapper({ children }: { children: ReactNode }) {
  return <UpdateProvider>{children}</UpdateProvider>;
}

beforeEach(() => {
  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
  cleanup();
  window.localStorage.clear();
  fetchMock.mockReset();
  vi.mocked(showUpdateToast).mockReset();
  vi.mocked(hideUpdateToast).mockReset();
  vi.unstubAllGlobals();
});

describe('UpdateProvider', () => {
  it('reports a newer release', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('available'));
    expect(result.current.update).toEqual({
      version: '0.14.0',
      url: 'https://github.com/xch-dev/sage/releases/tag/v0.14.0',
    });
  });

  it('reports up to date when the latest release is this version', async () => {
    respondWithTag('v0.13.1');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('up-to-date'));
    expect(result.current.update).toBeNull();
  });

  it('reports an error when GitHub cannot be reached', async () => {
    fetchMock.mockRejectedValue(new TypeError('Failed to fetch'));
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('error'));
    expect(result.current.update).toBeNull();
  });

  it('makes no requests while checks are off, but still checks on request', async () => {
    window.localStorage.setItem(UPDATE_CHECKS_STORAGE_KEY, 'false');
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.currentVersion).toBe('0.13.1'));
    // Give an effect that would fetch the chance to run.
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(result.current.status).toBe('idle');

    act(() => result.current.checkNow());

    await waitFor(() => expect(result.current.status).toBe('available'));
  });

  it('clears a found update when checks are turned off', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('available'));

    act(() => result.current.setEnabled(false));

    expect(result.current.status).toBe('idle');
    expect(result.current.update).toBeNull();
  });

  it('shows a toast once for a new release, and not again after it is dismissed', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(showUpdateToast).toHaveBeenCalledTimes(1));
    const [release, onDismiss] = vi.mocked(showUpdateToast).mock.calls[0];
    expect(release.version).toBe('0.14.0');

    // Another check finding the same release leaves the open toast alone.
    act(() => result.current.checkNow());
    await waitFor(() => expect(result.current.status).toBe('available'));
    expect(showUpdateToast).toHaveBeenCalledTimes(1);
    expect(hideUpdateToast).not.toHaveBeenCalled();

    act(() => onDismiss());

    expect(
      JSON.parse(
        window.localStorage.getItem(UPDATE_NOTIFIED_STORAGE_KEY) ?? 'null',
      ),
    ).toBe('0.14.0');
    expect(showUpdateToast).toHaveBeenCalledTimes(1);
  });

  it('skips the toast for a release that was already dismissed', async () => {
    window.localStorage.setItem(
      UPDATE_NOTIFIED_STORAGE_KEY,
      JSON.stringify('0.14.0'),
    );
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('available'));
    expect(showUpdateToast).not.toHaveBeenCalled();
  });

  it('shows a toast for a newer release than the one dismissed', async () => {
    window.localStorage.setItem(
      UPDATE_NOTIFIED_STORAGE_KEY,
      JSON.stringify('0.14.0'),
    );
    respondWithTag('v0.15.0');

    renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(showUpdateToast).toHaveBeenCalledTimes(1));
    expect(vi.mocked(showUpdateToast).mock.calls[0][0].version).toBe('0.15.0');
  });

  it('hides the toast when checks are turned off', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(showUpdateToast).toHaveBeenCalledTimes(1));

    act(() => result.current.setEnabled(false));

    expect(hideUpdateToast).toHaveBeenCalledWith('0.14.0');
    expect(window.localStorage.getItem(UPDATE_NOTIFIED_STORAGE_KEY)).toBeNull();
  });
});
