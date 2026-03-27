import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { render, RenderOptions } from '@testing-library/react';
import { ReactElement } from 'react';
import { MemoryRouter } from 'react-router-dom';

// Activate an empty en-US catalog
i18n.load('en', {});
i18n.activate('en');

function AllProviders({ children }: { children: React.ReactNode }) {
  return (
    <I18nProvider i18n={i18n}>
      <MemoryRouter>{children}</MemoryRouter>
    </I18nProvider>
  );
}

export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
) {
  return render(ui, { wrapper: AllProviders, ...options });
}
