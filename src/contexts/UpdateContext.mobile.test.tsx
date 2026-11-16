// @vitest-environment jsdom

import { showUpdateToast } from '@/components/UpdateToast';
import { act, renderHook, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { UpdateProvider, useUpdates } from './UpdateContext';

// Whether updates are supported is read once when the module loads, so mobile needs its own file.
vi.mock('@tauri-apps/plugin-os', () => ({
  platform: () => 'ios',
}));

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: () => Promise.resolve('0.13.1'),
}));

vi.mock('@/components/UpdateToast', () => ({
  showUpdateToast: vi.fn(),
  hideUpdateToast: vi.fn(),
}));

function wrapper({ children }: { children: ReactNode }) {
  return <UpdateProvider>{children}</UpdateProvider>;
}

describe('UpdateProvider on mobile', () => {
  it('never asks GitHub, since the app stores handle updates', async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    const { result } = renderHook(() => useUpdates(), { wrapper });

    await waitFor(() => expect(result.current.currentVersion).toBe('0.13.1'));
    act(() => result.current.checkNow());
    // Give an effect that would fetch the chance to run.
    await new Promise((resolve) => setTimeout(resolve, 10));

    expect(result.current.supported).toBe(false);
    expect(result.current.status).toBe('idle');
    expect(fetchMock).not.toHaveBeenCalled();
    expect(showUpdateToast).not.toHaveBeenCalled();

    vi.unstubAllGlobals();
  });
});
