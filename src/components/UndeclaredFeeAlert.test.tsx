// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';
import {
  UndeclaredFeeAlert,
  UndeclaredFeeAlertProps,
} from './UndeclaredFeeAlert';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(cleanup);

const unit = { ticker: 'XCH', precision: 12 };

function renderAlert(props: Omit<UndeclaredFeeAlertProps, 'unit'>) {
  return render(
    <I18nProvider i18n={i18n}>
      <UndeclaredFeeAlert unit={unit} {...props} />
    </I18nProvider>,
  );
}

describe('UndeclaredFeeAlert', () => {
  it('warns with the actual fee when it differs from the declared fee', () => {
    renderAlert({ fee: 250_100_000_000, declaredFee: 100_000_000 });

    expect(screen.getByText('Undeclared Fee')).toBeTruthy();
    expect(screen.getByText(/pays a fee of 0\.2501 XCH/)).toBeTruthy();
  });

  it('warns when nothing is declared', () => {
    renderAlert({ fee: '5', declaredFee: '0' });

    expect(screen.getByText('Undeclared Fee')).toBeTruthy();
  });

  it('compares amounts given as strings and numbers by value', () => {
    const { container } = renderAlert({
      fee: '100000000',
      declaredFee: 100_000_000,
    });

    expect(container.textContent).toBe('');
  });

  it('shows nothing for a zero fee with no declared fee', () => {
    const { container } = renderAlert({ fee: 0, declaredFee: undefined });

    expect(container.textContent).toBe('');
  });
});
