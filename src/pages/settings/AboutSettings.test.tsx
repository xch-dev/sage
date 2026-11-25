// @vitest-environment jsdom

import type { UpdateContextType } from '@/contexts/UpdateContext';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { UpdateSettings } from './AboutSettings';

let updates: UpdateContextType;

vi.mock('@/contexts/UpdateContext', () => ({
  useUpdates: () => updates,
}));

vi.mock('../../bindings', () => ({
  commands: {},
  events: { syncEvent: { listen: () => Promise.resolve(() => undefined) } },
}));

vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError: vi.fn() }),
}));

i18n.loadAndActivate({ locale: 'en', messages: {} });

function renderWith(overrides: Partial<UpdateContextType>) {
  updates = {
    supported: true,
    enabled: true,
    setEnabled: vi.fn(),
    currentVersion: '0.13.1',
    status: 'up-to-date',
    update: null,
    checkNow: vi.fn(),
    openRelease: vi.fn(),
    ...overrides,
  };
  render(
    <I18nProvider i18n={i18n}>
      <UpdateSettings />
    </I18nProvider>,
  );
}

afterEach(() => {
  cleanup();
});

describe('UpdateSettings', () => {
  it('checks on request when there is no update', () => {
    renderWith({ status: 'up-to-date' });

    expect(screen.getByText('Sage is up to date.')).toBeDefined();
    fireEvent.click(screen.getByRole('button', { name: 'Check Now' }));

    expect(updates.checkNow).toHaveBeenCalledTimes(1);
  });

  it('disables Check Now while a check is running', () => {
    renderWith({ status: 'checking' });

    expect(screen.getByText('Checking for updates...')).toBeDefined();
    expect(
      screen
        .getByRole('button', { name: 'Check Now' })
        .hasAttribute('disabled'),
    ).toBe(true);
  });

  it('opens the release page when an update is available', () => {
    const update = {
      version: '0.14.0',
      url: 'https://github.com/xch-dev/sage/releases/tag/v0.14.0',
    };
    renderWith({ status: 'available', update });

    expect(screen.getByText('Sage 0.14.0 is available.')).toBeDefined();
    fireEvent.click(screen.getByRole('button', { name: 'View Release' }));

    expect(updates.openRelease).toHaveBeenCalledTimes(1);
  });

  it('says checks are off only when they are', () => {
    renderWith({ status: 'idle', enabled: false });
    expect(screen.getByText('Automatic checks are off.')).toBeDefined();
    cleanup();

    renderWith({ status: 'idle', enabled: true });
    expect(screen.queryByText('Automatic checks are off.')).toBeNull();
  });

  it('turns automatic checks off with the switch', () => {
    renderWith({ enabled: true });

    fireEvent.click(screen.getByRole('switch', { name: 'Check for Updates' }));

    expect(updates.setEnabled).toHaveBeenCalledWith(false);
  });
});
