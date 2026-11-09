// @vitest-environment jsdom

import { act, cleanup, renderHook } from '@testing-library/react';
import { ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  openInAppDeepLink: vi.fn((text: string) => text.startsWith('sage:')),
}));

// state.ts subscribes to sync events on import
vi.mock('@/bindings', () => ({
  commands: {},
  events: { syncEvent: { listen: () => Promise.resolve(vi.fn()) } },
}));
vi.mock('@tauri-apps/plugin-os', () => ({ platform: () => 'macos' }));
vi.mock('@tauri-apps/plugin-barcode-scanner', () => ({}));
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({}));
vi.mock('@/hooks/useErrors', () => ({ useErrors: () => ({}) }));
vi.mock('@/lib/scanImage', () => ({}));
vi.mock('@/hooks/useDeepLink', () => ({
  openInAppDeepLink: mocks.openInAppDeepLink,
}));

const { useNavigationStore } = await import('@/state');
const { useScannerOrClipboard } = await import('./useScannerOrClipboard');

const PAGE = '/settings';

function wrapper({ children }: { children: ReactNode }) {
  return <MemoryRouter initialEntries={[PAGE]}>{children}</MemoryRouter>;
}

// Two scan fields on one page, as on the Settings page
function renderTwoFields() {
  const fields = [vi.fn(), vi.fn()];
  renderHook(
    () => {
      useScannerOrClipboard(fields[0]);
      useScannerOrClipboard(fields[1]);
    },
    { wrapper },
  );
  return fields;
}

describe('useScannerOrClipboard', () => {
  afterEach(() => {
    cleanup();
    mocks.openInAppDeepLink.mockClear();
    useNavigationStore.setState({ returnValues: {} });
  });

  it('opens a scanned sage: link once when several fields receive it', () => {
    const fields = renderTwoFields();
    act(() => {
      useNavigationStore
        .getState()
        .setReturnValue(PAGE, { status: 'success', data: 'sage:send' });
    });

    expect(mocks.openInAppDeepLink).toHaveBeenCalledTimes(1);
    expect(fields[0]).not.toHaveBeenCalled();
    expect(fields[1]).not.toHaveBeenCalled();
    expect(useNavigationStore.getState().returnValues[PAGE]?.status).toBe(
      'completed',
    );
  });

  it('still hands other scanned text to the fields', () => {
    const fields = renderTwoFields();
    act(() => {
      useNavigationStore
        .getState()
        .setReturnValue(PAGE, { status: 'success', data: 'xch1abc' });
    });

    expect(fields[0]).toHaveBeenCalledWith('xch1abc');
  });
});
