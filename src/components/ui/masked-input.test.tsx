// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

vi.mock('@/state', () => ({
  useWalletState: () => ({ sync: { unit: { ticker: 'XCH' } } }),
}));

vi.mock('@/hooks/useFeeRates', () => ({
  useFeeRates: () => ({ fee_per_cost: 5, minimum_fee_per_cost: 5 }),
}));

import { FeeAmountInput } from './masked-input';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  localStorage.clear();
});

function renderFee(value: string, onValueChange: () => void, allowAuto = true) {
  return render(
    <I18nProvider i18n={i18n}>
      <FeeAmountInput
        value={value}
        onValueChange={onValueChange}
        allowAuto={allowAuto}
      />
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

describe('FeeAmountInput with an automatic fee', () => {
  it('starts from an automatic fee by default', () => {
    const onValueChange = vi.fn();
    renderFee('', onValueChange);

    expect(onValueChange).toHaveBeenCalledWith({
      floatValue: undefined,
      value: 'auto',
    });
  });

  it("doesn't start an amount-only input from an automatic default", () => {
    const onValueChange = vi.fn();
    renderFee('', onValueChange, false);

    expect(onValueChange).not.toHaveBeenCalledWith(
      expect.objectContaining({ value: expect.stringMatching(/^auto/) }),
    );
    expect(screen.getByLabelText('Network fee amount')).toBeTruthy();
    expect(screen.queryByLabelText('Fee rate in mojos per cost')).toBeNull();
  });

  it('shows the recommended rate for an automatic fee', () => {
    renderFee('auto', vi.fn());

    const rate = screen.getByLabelText(
      'Fee rate in mojos per cost',
    ) as HTMLInputElement;
    expect(rate.value).toBe('5');
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('warns about a rate below the minimum', () => {
    renderFee('auto:2', vi.fn());
    expect(screen.getByRole('alert').textContent).toContain(
      'Below the minimum of 5 mojos per cost',
    );
  });

  it("doesn't warn about a rate at the minimum", () => {
    renderFee('auto:5', vi.fn());
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('switches between an automatic fee and an amount', () => {
    const onValueChange = vi.fn();
    const { rerender } = renderFee('auto', onValueChange);

    fireEvent.click(screen.getByLabelText('Enter a custom fee'));
    expect(onValueChange).toHaveBeenLastCalledWith({
      floatValue: undefined,
      value: '',
    });

    rerender(
      <I18nProvider i18n={i18n}>
        <FeeAmountInput value='0.001' onValueChange={onValueChange} />
      </I18nProvider>,
    );
    fireEvent.click(screen.getByLabelText('Calculate the fee automatically'));
    expect(onValueChange).toHaveBeenLastCalledWith({
      floatValue: undefined,
      value: 'auto',
    });
  });
});
