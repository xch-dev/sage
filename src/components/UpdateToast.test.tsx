// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { cleanup, render, screen } from '@testing-library/react';
import { ReactElement } from 'react';
import { toast } from 'react-toastify';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { showUpdateToast, updateToastId } from './UpdateToast';

vi.mock('react-toastify', () => ({
  toast: { info: vi.fn(), dismiss: vi.fn() },
}));

vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn() }));

const release = {
  version: '0.14.0',
  url: 'https://github.com/xch-dev/sage/releases/tag/v0.14.0',
};

i18n.loadAndActivate({ locale: 'en', messages: {} });

afterEach(() => {
  cleanup();
  vi.mocked(toast.info).mockReset();
});

describe('showUpdateToast', () => {
  it('stays open until it is dismissed', () => {
    showUpdateToast(release, vi.fn());

    const options = vi.mocked(toast.info).mock.calls[0][1];
    expect(options).toMatchObject({
      toastId: updateToastId('0.14.0'),
      autoClose: false,
      closeOnClick: true,
    });
  });

  it('only counts closes made by the user as dismissing it', () => {
    const onDismiss = vi.fn();
    showUpdateToast(release, onDismiss);

    const onClose = vi.mocked(toast.info).mock.calls[0][1]?.onClose;

    onClose?.(undefined);
    expect(onDismiss).not.toHaveBeenCalled();

    onClose?.(true);
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('opens the release page when clicked', () => {
    showUpdateToast(release, vi.fn());

    const [content, options] = vi.mocked(toast.info).mock.calls[0];
    // The toast renders outside Lingui's provider, so its content has to render without one.
    render(content as ReactElement);
    expect(screen.getByText('Sage 0.14.0 is available.')).toBeDefined();

    options?.onClick?.({} as never);

    expect(openUrl).toHaveBeenCalledWith(release.url);
  });
});
