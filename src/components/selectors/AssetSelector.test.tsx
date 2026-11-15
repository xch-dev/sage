// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

vi.mock('@/bindings', () => ({
  commands: {
    getCats: vi.fn(() =>
      Promise.resolve({
        cats: [
          {
            asset_id: 'cat1',
            name: 'Cat',
            ticker: 'CAT',
            precision: 3,
            balance: '500000',
            selectable_balance: '500000',
            visible: true,
          },
        ],
      }),
    ),
    getToken: vi.fn(() =>
      Promise.resolve({
        token: {
          asset_id: null,
          name: 'Chia',
          ticker: 'XCH',
          precision: 12,
          balance: '10000000000000',
          selectable_balance: '10000000000000',
          visible: true,
        },
      }),
    ),
    getOffersForAsset: vi.fn(() => Promise.resolve({ offers: [] })),
  },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));
vi.mock('./TokenSelector', () => ({ TokenSelector: () => null }));
vi.mock('./NftSelector', () => ({ NftSelector: () => null }));
vi.mock('./OptionSelector', () => ({ OptionSelector: () => null }));
vi.mock('@/components/ui/masked-input', () => ({
  TokenAmountInput: (props: {
    value: string;
    onValueChange: (values: { value: string }) => void;
  }) => (
    <input
      aria-label='Amount'
      value={props.value}
      onChange={(e) => props.onValueChange({ value: e.target.value })}
    />
  ),
}));

import { Assets } from '@/state';
import { AssetSelector } from './AssetSelector';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(cleanup);

function renderOffering(assets: Assets, fee = '0') {
  const setAssets = vi.fn();
  render(
    <I18nProvider i18n={i18n}>
      <AssetSelector
        offering
        prefix='offer'
        assets={assets}
        setAssets={setAssets}
        fee={fee}
      />
    </I18nProvider>,
  );
  return setAssets;
}

const tokens = (assetId: string | null, amount: string): Assets => ({
  tokens: [{ asset_id: assetId, amount }],
  nfts: [],
  options: [],
});

describe('AssetSelector offered tokens', () => {
  it('shows the spendable balance under the row', async () => {
    renderOffering(tokens('cat1', ''));
    expect(await screen.findByText(/500 CAT spendable/)).toBeTruthy();
  });

  it('flags an amount above the spendable balance', async () => {
    renderOffering(tokens('cat1', '501'));
    expect(
      await screen.findByText('More than your spendable balance'),
    ).toBeTruthy();
  });

  it('counts the network fee against an XCH amount', async () => {
    renderOffering(tokens(null, '10'), '0.5');
    expect(
      await screen.findByText(
        'More than your spendable balance after the network fee',
      ),
    ).toBeTruthy();
  });

  it('fills in the spendable balance less the fee with Max', async () => {
    const setAssets = renderOffering(tokens(null, ''), '0.5');
    await screen.findByText(/XCH spendable/);

    fireEvent.click(
      screen.getByRole('button', { name: 'Use maximum spendable balance' }),
    );

    expect(setAssets).toHaveBeenCalledWith(tokens(null, '9.5'));
  });
});
