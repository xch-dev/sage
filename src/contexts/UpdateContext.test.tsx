// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { hideUpdateToast, showUpdateToast } from '@/components/UpdateToast';
import { openUrl } from '@tauri-apps/plugin-opener';
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

vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn() }));

const fetchMock = vi.fn();

function storedNotifiedVersion(): unknown {
  return JSON.parse(
    window.localStorage.getItem(UPDATE_NOTIFIED_STORAGE_KEY) ?? 'null',
  );
}

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
  vi.mocked(openUrl).mockReset();
  vi.useRealTimers();
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

    expect(storedNotifiedVersion()).toBe('0.14.0');
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

  it('hides the toast for good when checks are turned off', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(showUpdateToast).toHaveBeenCalledTimes(1));

    act(() => result.current.setEnabled(false));

    expect(hideUpdateToast).toHaveBeenCalledWith('0.14.0');
    expect(storedNotifiedVersion()).toBe('0.14.0');

    // Turning checks back on finds the same release, but doesn't pop the toast up again.
    act(() => result.current.setEnabled(true));
    await waitFor(() => expect(result.current.status).toBe('available'));
    expect(showUpdateToast).toHaveBeenCalledTimes(1);
  });

  it('opens the release and closes its toast for good', async () => {
    respondWithTag('v0.14.0');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(showUpdateToast).toHaveBeenCalledTimes(1));

    act(() => result.current.openRelease());

    expect(openUrl).toHaveBeenCalledWith(
      'https://github.com/xch-dev/sage/releases/tag/v0.14.0',
    );
    expect(hideUpdateToast).toHaveBeenCalledWith('0.14.0');
    expect(storedNotifiedVersion()).toBe('0.14.0');
    // The update is still shown elsewhere, such as the sidebar.
    expect(result.current.update?.version).toBe('0.14.0');
  });

  it('ignores a check still running when checks are turned off', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    let respond: (response: Response) => void = () => undefined;
    fetchMock.mockImplementation(
      () =>
        new Promise<Response>((resolve) => {
          respond = resolve;
        }),
    );

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('checking'));

    act(() => result.current.setEnabled(false));
    await act(async () => {
      respond(new Response(JSON.stringify({ tag_name: 'v0.14.0' })));
    });

    expect(result.current.status).toBe('idle');
    expect(result.current.update).toBeNull();
    expect(showUpdateToast).not.toHaveBeenCalled();

    // The check that finished late doesn't schedule another one.
    await act(() => vi.advanceTimersByTimeAsync(12 * 60 * 60 * 1000));
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('checks again every 6 hours, and stops when checks are turned off', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    respondWithTag('v0.13.1');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('up-to-date'));
    expect(fetchMock).toHaveBeenCalledTimes(1);

    await act(() => vi.advanceTimersByTimeAsync(15 * 60 * 1000));
    expect(fetchMock).toHaveBeenCalledTimes(1);

    await act(() => vi.advanceTimersByTimeAsync(6 * 60 * 60 * 1000));
    expect(fetchMock).toHaveBeenCalledTimes(2);

    act(() => result.current.setEnabled(false));
    await act(() => vi.advanceTimersByTimeAsync(12 * 60 * 60 * 1000));
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('tries again sooner after a check fails', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    respondWithTag('v0.14.0');
    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'));

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('error'));

    await act(() => vi.advanceTimersByTimeAsync(15 * 60 * 1000 - 1000));
    expect(fetchMock).toHaveBeenCalledTimes(1);

    await act(() => vi.advanceTimersByTimeAsync(1000));
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(result.current.status).toBe('available');
  });

  it('retries a failed Check Now on the same schedule', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    respondWithTag('v0.13.1');

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.status).toBe('up-to-date'));

    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'));
    act(() => result.current.checkNow());
    await waitFor(() => expect(result.current.status).toBe('error'));
    expect(fetchMock).toHaveBeenCalledTimes(2);

    await act(() => vi.advanceTimersByTimeAsync(15 * 60 * 1000));
    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(result.current.status).toBe('up-to-date');
  });
});
