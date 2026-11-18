// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

vi.mock('@/state', () => ({
  useWalletState: () => ({ sync: { unit: { ticker: 'XCH' } } }),
}));

const feeRates = vi.hoisted(() => ({
  current: {
    fee_per_cost: 5,
    minimum_fee_per_cost: 5,
    tiers: [] as { name: string; target_time: number; fee_per_cost: number }[],
  },
}));

const ESTIMATED_TIERS = [
  { name: 'fast', target_time: 60, fee_per_cost: 12.5 },
  { name: 'standard', target_time: 300, fee_per_cost: 8 },
  { name: 'economy', target_time: 3600, fee_per_cost: 5 },
];

vi.mock('@/hooks/useFeeRates', () => ({
  useFeeRates: () => feeRates.current,
}));

import { FeeAmountInput, TokenAmountInput } from './masked-input';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });

  // Radix's select uses these, which jsdom doesn't have.
  Element.prototype.hasPointerCapture ??= () => false;
  Element.prototype.releasePointerCapture ??= vi.fn();
  Element.prototype.scrollIntoView ??= vi.fn();
});

beforeEach(() => {
  feeRates.current = { ...feeRates.current, tiers: ESTIMATED_TIERS };
});

afterEach(() => {
  cleanup();
  localStorage.clear();
});

function openSpeeds() {
  const trigger = screen.getByLabelText('Fee speed');
  fireEvent.pointerDown(trigger, { button: 0, pointerType: 'mouse' });
  fireEvent.keyDown(trigger, { key: 'Enter' });
}

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
      value: 'auto:standard',
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

  it("shows a tier's estimated rate", () => {
    renderFee('auto:fast', vi.fn());

    expect(screen.getByLabelText('Fee speed').textContent).toBe('Fast');
    expect(
      screen.getByLabelText('Fee rate in mojos per cost').textContent,
    ).toBe('12.5 mojos/cost');
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows the standard tier for an automatic fee stored before tiers', () => {
    renderFee('auto', vi.fn());

    expect(screen.getByLabelText('Fee speed').textContent).toBe('Standard');
    expect(
      screen.getByLabelText('Fee rate in mojos per cost').textContent,
    ).toBe('8 mojos/cost');
  });

  it('pays the recommended rate at every tier without estimates', () => {
    feeRates.current = { ...feeRates.current, tiers: [] };
    renderFee('auto:fast', vi.fn());

    expect(
      screen.getByLabelText('Fee rate in mojos per cost').textContent,
    ).toBe('5 mojos/cost');

    openSpeeds();
    expect(
      screen.getByText(/Fee estimates aren't available on this network/),
    ).toBeTruthy();
  });

  it('lists every tier with its rate, and picks one', () => {
    const onValueChange = vi.fn();
    renderFee('auto:standard', onValueChange);

    openSpeeds();
    expect(screen.getByText('About a minute · 12.5 mojos/cost')).toBeTruthy();
    expect(screen.getByText('Within an hour · 5 mojos/cost')).toBeTruthy();
    expect(screen.queryByText(/Fee estimates aren't available/)).toBeNull();

    fireEvent.click(screen.getByRole('option', { name: /Economy/ }));
    expect(onValueChange).toHaveBeenLastCalledWith({
      floatValue: undefined,
      value: 'auto:economy',
    });
  });

  it("enters a custom rate, starting from the tier's rate", () => {
    const onValueChange = vi.fn();
    renderFee('auto:fast', onValueChange);

    openSpeeds();
    fireEvent.click(screen.getByRole('option', { name: /Custom rate/ }));
    expect(onValueChange).toHaveBeenLastCalledWith({
      floatValue: undefined,
      value: 'auto:12.5',
    });
  });

  it('edits a custom rate', () => {
    renderFee('auto:7', vi.fn());

    expect(screen.getByLabelText('Fee speed').textContent).toBe('Custom');
    const rate = screen.getByLabelText(
      'Fee rate in mojos per cost',
    ) as HTMLInputElement;
    expect(rate.value).toBe('7');
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
    const { rerender } = renderFee('auto:standard', onValueChange);

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
      value: 'auto:standard',
    });
  });

  it('switches to the default tier when the fee is made automatic', () => {
    localStorage.setItem(
      'defaultNetworkFee',
      JSON.stringify({ fee: 'auto:economy' }),
    );
    const onValueChange = vi.fn();
    renderFee('0.001', onValueChange);

    fireEvent.click(screen.getByLabelText('Calculate the fee automatically'));
    expect(onValueChange).toHaveBeenLastCalledWith({
      floatValue: undefined,
      value: 'auto:economy',
    });
  });
});

describe('TokenAmountInput', () => {
  function renderAmount(props: {
    wholeBalance?: boolean;
    onWholeBalanceChange?: (whole: boolean) => void;
    onValueChange?: () => void;
  }) {
    return render(
      <I18nProvider i18n={i18n}>
        <TokenAmountInput
          aria-label='Amount'
          value=''
          maxValue='10'
          ticker='xch'
          {...props}
        />
      </I18nProvider>,
    );
  }

  it('fills in the maximum without a whole balance option', () => {
    const onValueChange = vi.fn();
    renderAmount({ onValueChange });

    fireEvent.click(screen.getByLabelText('Use maximum spendable balance'));
    expect(onValueChange).toHaveBeenCalledWith(
      expect.objectContaining({ value: '10' }),
    );
  });

  it('switches to the whole balance instead of filling in an amount', () => {
    const onValueChange = vi.fn();
    const onWholeBalanceChange = vi.fn();
    renderAmount({ onValueChange, onWholeBalanceChange });

    fireEvent.click(screen.getByLabelText('Use maximum spendable balance'));
    expect(onWholeBalanceChange).toHaveBeenCalledWith(true);
    expect(onValueChange).not.toHaveBeenCalled();
  });

  it("can't be switched to the whole balance without a balance", () => {
    const onWholeBalanceChange = vi.fn();
    render(
      <I18nProvider i18n={i18n}>
        <TokenAmountInput
          aria-label='Amount'
          value=''
          maxValue='0'
          ticker='xch'
          onWholeBalanceChange={onWholeBalanceChange}
        />
      </I18nProvider>,
    );

    const max = screen.getByLabelText(
      'Use maximum spendable balance',
    ) as HTMLButtonElement;
    expect(max.disabled).toBe(true);
    fireEvent.click(max);
    expect(onWholeBalanceChange).not.toHaveBeenCalled();
  });

  it('says the whole balance is sent minus the fee, and can go back', () => {
    const onWholeBalanceChange = vi.fn();
    renderAmount({ wholeBalance: true, onWholeBalanceChange });

    expect(screen.getByText('Balance minus fee')).toBeTruthy();
    expect(screen.queryByLabelText('Amount')).toBeNull();

    fireEvent.click(screen.getByLabelText('Enter an amount instead'));
    expect(onWholeBalanceChange).toHaveBeenCalledWith(false);
  });
});
