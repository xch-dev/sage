// @vitest-environment jsdom

import type { GetFeeRateResponse } from '@/bindings';
import type { OfferState } from '@/state';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const rates = vi.hoisted(() => ({
  current: null as GetFeeRateResponse | null,
}));

vi.mock('@/bindings', () => ({
  commands: {
    getToken: vi.fn().mockResolvedValue({ token: null }),
  },
}));
vi.mock('theme-o-rama', () => ({
  useTheme: () => ({ currentTheme: null }),
}));
vi.mock('@/lib/marketplaces', () => ({ marketplaces: [] }));
vi.mock('@/hooks/useFeeRates', () => ({
  useFeeRates: () => rates.current,
}));

import { MakeOfferConfirmationDialog } from './MakeOfferConfirmationDialog';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  rates.current = null;
});

const offerState: OfferState = {
  offered: { tokens: [], nfts: [], options: [] },
  requested: { tokens: [], nfts: [], options: [] },
  fee: '0',
  expiration: null,
  splitNftOffers: false,
  preSplit: true,
  preSplitFee: '',
};

function renderDialog(preSplitFee: string | null) {
  const onConfirm = vi.fn();
  render(
    <I18nProvider i18n={i18n}>
      <MakeOfferConfirmationDialog
        open
        onOpenChange={vi.fn()}
        onConfirm={onConfirm}
        offerState={offerState}
        splitNftOffers={false}
        fee='0'
        preSplitFee={preSplitFee}
      />
    </I18nProvider>,
  );
  return onConfirm;
}

/** The split fee as it's shown, after its label. */
function splitFee() {
  const line = screen.getByText(/Split Fee:/, { selector: 'p' });
  return line.textContent?.replace('Split Fee:', '').trim();
}

function confirm() {
  fireEvent.click(screen.getByRole('button', { name: 'Confirm & Create' }));
}

describe('MakeOfferConfirmationDialog split fee', () => {
  it("shows a tier's estimated rate and confirms the split at it", () => {
    rates.current = {
      fee_per_cost: 5,
      minimum_fee_per_cost: 5,
      tiers: [
        { name: 'fast', target_time: 60, fee_per_cost: 48.15 },
        { name: 'standard', target_time: 300, fee_per_cost: 12.5 },
        { name: 'economy', target_time: 3600, fee_per_cost: 5 },
      ],
    };
    const onConfirm = renderDialog('auto:fast');

    expect(splitFee()).toBe('Automatic, 48.15 mojos per cost');

    // Fixed at the rate shown, so a newer estimate doesn't change what's paid.
    confirm();
    expect(onConfirm).toHaveBeenCalledWith('auto:48.15');
  });

  it('shows the recommended rate without estimates', () => {
    rates.current = { fee_per_cost: 5, minimum_fee_per_cost: 5, tiers: [] };
    const onConfirm = renderDialog('auto:fast');

    expect(splitFee()).toBe('Automatic, 5 mojos per cost');
    confirm();
    expect(onConfirm).toHaveBeenCalledWith('auto:5');
  });

  it('leaves a tier to be priced later until the rates load', () => {
    const onConfirm = renderDialog('auto:standard');

    expect(splitFee()).toBe('Automatic');
    confirm();
    expect(onConfirm).toHaveBeenCalledWith('auto:standard');
  });

  it('confirms a split fee amount as it is', () => {
    const onConfirm = renderDialog('0.0001');

    confirm();
    expect(onConfirm).toHaveBeenCalledWith('0.0001');
  });
});
