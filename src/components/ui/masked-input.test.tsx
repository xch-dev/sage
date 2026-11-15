// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, render } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

vi.mock('@/state', () => ({
  useWalletState: () => ({ sync: { unit: { ticker: 'XCH' } } }),
}));

import { FeeAmountInput } from './masked-input';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  localStorage.clear();
});

function renderFee(value: string, onValueChange: () => void) {
  return render(
    <I18nProvider i18n={i18n}>
      <FeeAmountInput value={value} onValueChange={onValueChange} />
    </I18nProvider>,
  );
}

describe('FeeAmountInput', () => {
  it('starts an empty input from the default fee', () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0.001' }));
    const onValueChange = vi.fn();
    renderFee('', onValueChange);

    expect(onValueChange).toHaveBeenCalledWith({
      floatValue: 0.001,
      value: '0.001',
    });
  });

  it("doesn't fill in the default when a value is cleared later", () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0.001' }));
    const onValueChange = vi.fn();
    const { rerender } = renderFee('0.5', onValueChange);

    // Like the offer page clearing its stored fee on Cancel.
    rerender(
      <I18nProvider i18n={i18n}>
        <FeeAmountInput value='' onValueChange={onValueChange} />
      </I18nProvider>,
    );

    // The input may echo the new value back, but never the default fee.
    expect(onValueChange).not.toHaveBeenCalledWith(
      expect.objectContaining({ value: '0.001' }),
      undefined,
    );
    expect(onValueChange).not.toHaveBeenCalledWith({
      floatValue: 0.001,
      value: '0.001',
    });
  });
});
