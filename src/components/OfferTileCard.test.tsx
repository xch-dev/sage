// @vitest-environment jsdom

import { Asset, OfferAsset, OfferRecord } from '@/bindings';
import { i18n } from '@lingui/core';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';
import { OfferTileCard, offerSummaryLabel } from './OfferTileCard';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(cleanup);

const NOW = Date.UTC(2026, 0, 10, 12, 0, 0);
const NOW_SECONDS = Math.floor(NOW / 1000);

function asset(overrides: Partial<Asset>): Asset {
  return {
    asset_id: 'a'.repeat(64),
    name: null,
    ticker: null,
    precision: 3,
    icon_url: null,
    description: null,
    is_sensitive_content: false,
    is_visible: true,
    revocation_address: null,
    kind: 'token',
    ...overrides,
  };
}

function offered(assetValue: Asset, amount: number): OfferAsset {
  return {
    asset: assetValue,
    amount,
    royalty: 0,
    nft_royalty: null,
    option_assets: null,
  };
}

const sbx = offered(asset({ name: 'Spacebucks', ticker: 'SBX' }), 1_500_000);
const nft = offered(
  asset({
    asset_id: 'nft1x',
    name: 'Chia Friends #42',
    kind: 'nft',
    precision: 0,
  }),
  1,
);

function record(overrides: Partial<OfferRecord['summary']> = {}): OfferRecord {
  return {
    offer_id: 'b'.repeat(64),
    offer: 'offer1…',
    status: 'active',
    creation_timestamp: NOW_SECONDS - 2 * 3600,
    summary: {
      fee: 0,
      maker: [sbx],
      taker: [nft],
      expiration_height: null,
      expiration_timestamp: NOW_SECONDS + 3 * 86400,
      ...overrides,
    },
  };
}

describe('OfferTileCard', () => {
  it('shows compact token amounts and NFT names without an amount', () => {
    render(<OfferTileCard record={record()} content={null} now={NOW} />);

    expect(screen.getByText('1.5K')).toBeTruthy();
    expect(screen.getByText('SBX')).toBeTruthy();
    expect(screen.getByText('Chia Friends #42')).toBeTruthy();
    expect(screen.queryByText('1')).toBeNull();
  });

  it('shows the status as text next to its icon', () => {
    render(<OfferTileCard record={record()} content={null} now={NOW} />);
    expect(screen.getByText('Active')).toBeTruthy();
  });

  it('collapses extra assets on a side into +N', () => {
    const extra = offered(
      asset({ asset_id: 'c'.repeat(64), ticker: 'MRMT' }),
      5,
    );
    const third = offered(
      asset({ asset_id: 'd'.repeat(64), ticker: 'PURE' }),
      7,
    );
    render(
      <OfferTileCard
        record={record({ maker: [sbx, extra, third] })}
        content={null}
        now={NOW}
      />,
    );
    expect(screen.getByText('+2')).toBeTruthy();
  });

  it('shows relative expiry and age', () => {
    render(<OfferTileCard record={record()} content={null} now={NOW} />);
    expect(screen.getByText('in 3d')).toBeTruthy();
    expect(screen.getByText('2h ago')).toBeTruthy();
  });

  it('shows the block height for height-based expiry', () => {
    render(
      <OfferTileCard
        record={record({
          expiration_timestamp: null,
          expiration_height: 123456,
        })}
        content={null}
        now={NOW}
      />,
    );
    expect(screen.getByTitle('Expires at block 123456')).toBeTruthy();
  });

  it('shows no expiry for offers that never expire', () => {
    render(
      <OfferTileCard
        record={record({ expiration_timestamp: null })}
        content={null}
        now={NOW}
      />,
    );
    expect(screen.queryByText('in 3d')).toBeNull();
    expect(screen.queryByTitle(/Expires at block/)).toBeNull();
  });

  it('renders the selection checkbox only in multi-select', () => {
    const { rerender } = render(
      <OfferTileCard record={record()} content={null} now={NOW} />,
    );
    expect(screen.queryByRole('checkbox')).toBeNull();

    rerender(
      <OfferTileCard
        record={record()}
        content={null}
        now={NOW}
        selectionState={[true, () => undefined]}
      />,
    );
    expect(screen.getByRole('checkbox')).toBeTruthy();
  });
});

describe('offerSummaryLabel', () => {
  it('describes the trade for screen readers', () => {
    expect(offerSummaryLabel(record(), NOW)).toBe(
      'Active offer: 1,500 SBX for Chia Friends #42, expires in 3d',
    );
  });

  it('omits expiry when the offer never expires', () => {
    expect(offerSummaryLabel(record({ expiration_timestamp: null }), NOW)).toBe(
      'Active offer: 1,500 SBX for Chia Friends #42',
    );
  });
});
