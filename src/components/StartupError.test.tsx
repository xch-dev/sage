// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { StartupError } from './StartupError';

const { resetAppsDatabase, quit } = vi.hoisted(() => ({
  resetAppsDatabase: vi.fn(() => Promise.resolve(null)),
  quit: vi.fn(() => Promise.resolve(null)),
}));

vi.mock('@/bindings', () => ({
  commands: { resetAppsDatabase, quit },
}));

vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError: vi.fn() }),
}));

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: () => 'macos',
}));

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function renderError(kind: 'apps_database' | 'internal') {
  return render(
    <I18nProvider i18n={i18n}>
      <StartupError error={{ kind, reason: 'it broke' }} />
    </I18nProvider>,
  );
}

describe('StartupError', () => {
  it('deletes the apps database only after a second confirmation', () => {
    renderError('apps_database');

    expect(
      screen.getByText('The apps database could not be opened'),
    ).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Delete and restart' }));
    expect(resetAppsDatabase).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole('button', { name: 'Yes, delete and restart' }),
    );
    expect(resetAppsDatabase).toHaveBeenCalledTimes(1);
  });

  it('offers no reset for other startup errors', () => {
    renderError('internal');

    expect(screen.getByText('Sage could not start')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /delete/i })).toBeNull();
  });
});
