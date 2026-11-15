// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from '@testing-library/react';
import { bech32m } from 'bech32';
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
    getAllCats: vi.fn(() => Promise.resolve({ cats: [] })),
    getOffersForAsset: vi.fn(() => Promise.resolve({ offers: [] })),
    getNfts: vi.fn(() =>
      Promise.resolve({
        nfts: [
          {
            launcher_id: 'nft1first',
            name: 'First NFT',
            metadata_updater_kind: null,
          },
          {
            launcher_id: 'nft1second',
            name: 'Second NFT',
            metadata_updater_kind: null,
          },
        ],
        total: 2,
      }),
    ),
    getNft: vi.fn(() => Promise.resolve({ nft: null })),
    getOption: vi.fn(() => Promise.resolve({ option: null })),
    getOptions: vi.fn(() =>
      Promise.resolve({
        options: [
          {
            launcher_id: 'option1live',
            name: 'Live option',
            expiration_seconds: 4102444800,
            underlying_asset: { kind: 'token', ticker: 'CAT', precision: 3 },
            underlying_amount: 1000,
            strike_asset: { kind: 'token', ticker: 'XCH', precision: 12 },
            strike_amount: 1000000000000,
          },
          {
            launcher_id: 'option1expired',
            name: 'Expired option',
            expiration_seconds: 1,
            underlying_asset: { kind: 'token', ticker: 'CAT', precision: 3 },
            underlying_amount: 1000,
            strike_asset: { kind: 'token', ticker: 'XCH', precision: 12 },
            strike_amount: 1000000000000,
          },
        ],
      }),
    ),
    getNftThumbnail: vi.fn(() => Promise.resolve({ thumbnail: null })),
  },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));
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

import { commands, TokenRecord } from '@/bindings';
import { Assets } from '@/state';
import { AssetSelector } from './AssetSelector';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
  // cmdk and Radix use these, and jsdom doesn't have them.
  globalThis.ResizeObserver ??= class {
    observe() {
      return undefined;
    }
    unobserve() {
      return undefined;
    }
    disconnect() {
      return undefined;
    }
  };
  Element.prototype.scrollIntoView ??= vi.fn();
});

afterEach(cleanup);

function renderOffering(assets: Assets, fee = '0', offering = true) {
  const setAssets = vi.fn();
  render(
    <I18nProvider i18n={i18n}>
      <AssetSelector
        offering={offering}
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

function renderSelector(props: {
  assets: Assets;
  offering?: boolean;
  emptyMessage?: string;
}) {
  const setAssets = vi.fn();
  render(
    <I18nProvider i18n={i18n}>
      <AssetSelector
        offering={props.offering}
        prefix='test'
        assets={props.assets}
        setAssets={setAssets}
        emptyMessage={props.emptyMessage}
        fee='0'
      />
    </I18nProvider>,
  );
  return setAssets;
}

const noAssets: Assets = { tokens: [], nfts: [], options: [] };

describe('AssetSelector balance edges', () => {
  it('accepts an amount equal to the spendable balance', async () => {
    renderOffering(tokens('cat1', '500'));
    await screen.findByText(/500 CAT spendable/);
    expect(screen.queryByText(/More than your spendable balance/)).toBeNull();
  });

  it('only takes the network fee off XCH', async () => {
    renderOffering(tokens('cat1', '500'), '0.5');
    await screen.findByText(/500 CAT spendable/);
    expect(screen.queryByText(/More than your spendable balance/)).toBeNull();
  });
});

describe('AssetSelector requested tokens', () => {
  it("doesn't show a balance, Max or a balance error", async () => {
    renderSelector({ assets: tokens('cat1', '501') });
    // Give any balance lookup the chance to finish.
    await act(async () => undefined);

    expect(screen.queryByText(/spendable/)).toBeNull();
    expect(
      screen.queryByRole('button', { name: 'Use maximum spendable balance' }),
    ).toBeNull();
    expect(screen.queryByText(/More than your spendable balance/)).toBeNull();
  });
});

describe('AssetSelector layout', () => {
  it('shows the empty message only while nothing has been added', () => {
    renderSelector({ assets: noAssets, emptyMessage: 'Nothing yet' });
    expect(screen.getByText('Nothing yet')).toBeTruthy();

    cleanup();
    renderSelector({
      assets: { ...noAssets, nfts: [''] },
      emptyMessage: 'Nothing yet',
    });
    expect(screen.queryByText('Nothing yet')).toBeNull();
  });

  it('stores a pasted NFT ID without surrounding whitespace', () => {
    const setAssets = renderSelector({
      assets: { ...noAssets, nfts: [''] },
    });

    fireEvent.change(screen.getByLabelText('NFT ID'), {
      target: { value: '  nft1example\n' },
    });
    expect(setAssets).toHaveBeenCalledWith({
      ...noAssets,
      nfts: ['nft1example'],
    });
  });
});

const empty: Assets = { tokens: [], nfts: [], options: [] };

describe('AssetSelector pickers', () => {
  it('adds a token row only once a token is picked', async () => {
    const setAssets = renderOffering(empty);

    fireEvent.click(screen.getByRole('button', { name: 'Token' }));
    fireEvent.click(await screen.findByRole('option', { name: /Cat/ }));

    expect(setAssets).toHaveBeenCalledTimes(1);
    expect(setAssets).toHaveBeenCalledWith(tokens('cat1', ''));
  });

  it("won't offer a token twice", async () => {
    renderOffering(tokens('cat1', '1'));

    fireEvent.click(screen.getByRole('button', { name: 'Token' }));
    const option = await screen.findByRole('option', { name: /Cat/ });
    expect(option.getAttribute('aria-disabled')).toBe('true');
  });

  it('requests a CAT by a pasted asset ID', async () => {
    const assetId = 'ab'.repeat(32);
    const setAssets = renderOffering(empty, '0', false);

    fireEvent.click(screen.getByRole('button', { name: 'Token' }));
    fireEvent.change(
      await screen.findByPlaceholderText('Search tokens or paste an asset ID'),
      { target: { value: assetId } },
    );
    fireEvent.click(
      await screen.findByRole('option', { name: /Request this asset ID/ }),
    );

    expect(setAssets).toHaveBeenCalledWith(tokens(assetId, ''));
  });

  it('toggles offered NFTs from the picker', async () => {
    const setAssets = renderOffering({ ...empty, nfts: ['nft1first'] });

    fireEvent.click(screen.getByRole('button', { name: 'NFT' }));
    fireEvent.click(await screen.findByRole('option', { name: /Second NFT/ }));
    expect(setAssets).toHaveBeenLastCalledWith({
      ...empty,
      nfts: ['nft1first', 'nft1second'],
    });

    fireEvent.click(screen.getByRole('option', { name: /First NFT/ }));
    expect(setAssets).toHaveBeenLastCalledWith(empty);
  });

  it('adds unexpired options from the picker', async () => {
    const setAssets = renderOffering(empty);

    fireEvent.click(screen.getByRole('button', { name: 'Option' }));
    const option = await screen.findByRole('option', { name: /Live option/ });
    expect(screen.getByText('1 CAT for 1 XCH', { exact: false })).toBeTruthy();
    expect(screen.queryByRole('option', { name: /Expired option/ })).toBeNull();

    fireEvent.click(option);
    expect(setAssets).toHaveBeenLastCalledWith({
      ...empty,
      options: ['option1live'],
    });
  });

  it('shows requestable tokens a page at a time, held tokens first', async () => {
    const cat = (i: number, balance = '0') =>
      ({
        asset_id: i.toString(16).padStart(64, '0'),
        name: `Token ${i.toString().padStart(3, '0')}`,
        ticker: `T${i}`,
        precision: 3,
        balance,
        selectable_balance: balance,
        visible: true,
      }) as unknown as TokenRecord;
    const cats = Array.from({ length: 120 }, (_, i) => cat(i + 1));
    cats[80] = cat(81, '5000');
    vi.mocked(commands.getAllCats).mockResolvedValue({ cats });

    try {
      renderOffering(empty, '0', false);
      fireEvent.click(screen.getByRole('button', { name: 'Token' }));
      await screen.findByRole('option', { name: /Token 001/ });

      const options = () => screen.getAllByRole('option');
      expect(options()).toHaveLength(50);
      expect(options()[0].textContent).toMatch(/^Chia/);
      expect(options()[1].textContent).toMatch(/Token 081/);

      fireEvent.click(screen.getByRole('button', { name: 'Show more' }));
      expect(options()).toHaveLength(100);
    } finally {
      vi.mocked(commands.getAllCats).mockResolvedValue({ cats: [] });
    }
  });

  it('puts an exact ticker match first', async () => {
    const token = (assetId: string, name: string, ticker: string) =>
      ({
        asset_id: assetId.repeat(64),
        name,
        ticker,
        precision: 3,
        balance: '0',
        selectable_balance: '0',
        visible: true,
      }) as unknown as TokenRecord;
    vi.mocked(commands.getAllCats).mockResolvedValue({
      cats: [
        token('a', 'Another SBX fork', 'SBXX'),
        token('b', 'Spacebucks', 'SBX'),
      ],
    });

    try {
      renderOffering(empty, '0', false);
      fireEvent.click(screen.getByRole('button', { name: 'Token' }));
      fireEvent.change(
        await screen.findByPlaceholderText(
          'Search tokens or paste an asset ID',
        ),
        { target: { value: 'sbx' } },
      );
      await screen.findByRole('option', { name: /Spacebucks/ });
      expect(screen.getAllByRole('option')[0].textContent).toMatch(
        /^Spacebucks/,
      );
    } finally {
      vi.mocked(commands.getAllCats).mockResolvedValue({ cats: [] });
    }
  });
});

describe('AssetSelector row edits', () => {
  it('changes a row to another token and clears its amount', async () => {
    const setAssets = renderOffering(tokens('cat1', '5'));

    fireEvent.click(screen.getByRole('button', { name: 'Change token' }));
    fireEvent.click(await screen.findByRole('option', { name: /^Chia/ }));
    expect(setAssets).toHaveBeenCalledWith(tokens(null, ''));
  });

  it('leaves a row alone when its own token is picked again', async () => {
    const setAssets = renderOffering(tokens('cat1', '5'));

    fireEvent.click(screen.getByRole('button', { name: 'Change token' }));
    fireEvent.click(await screen.findByRole('option', { name: /Cat/ }));
    expect(setAssets).not.toHaveBeenCalled();
  });

  it('flags a requested option ID that is not valid', () => {
    renderSelector({ assets: { ...noAssets, options: ['option1bad'] } });
    expect(screen.getByText("This isn't a valid option ID.")).toBeTruthy();
  });

  it("doesn't flag an empty or valid requested option ID", () => {
    const validId = bech32m.encode(
      'option',
      bech32m.toWords(new Uint8Array(32).fill(3)),
    );
    renderSelector({ assets: { ...noAssets, options: ['', validId] } });
    expect(screen.queryByText("This isn't a valid option ID.")).toBeNull();
  });

  it('stores a pasted option ID without surrounding whitespace', () => {
    const setAssets = renderSelector({
      assets: { ...noAssets, options: [''] },
    });

    fireEvent.change(screen.getByLabelText('Option ID'), {
      target: { value: ' option1example ' },
    });
    expect(setAssets).toHaveBeenCalledWith({
      ...noAssets,
      options: ['option1example'],
    });
  });
});
