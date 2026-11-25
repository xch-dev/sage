import { PricedTokenRecord } from '@/types/TokenViewProps';
import { i18n } from '@lingui/core';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { exportTokens } from './exportTokens';

const exportText = vi.fn();

vi.mock('@/lib/exportText', () => ({
  exportText: (...args: unknown[]) => exportText(...args),
}));

vi.mock('react-toastify', () => ({
  toast: { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() },
}));

const xch = {
  name: 'Chia',
  ticker: 'XCH',
  asset_id: null,
  balance: '1500000000000',
  icon_url: 'https://example.com/xch.png',
  balanceInFiat: 30,
  priceInFiat: 20,
} as unknown as PricedTokenRecord;

async function exportedRows(currency: string | null) {
  await exportTokens([xch], currency);
  const csv = exportText.mock.calls[0][0] as string;
  return csv.split('\n').map((line) => line.split(','));
}

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

beforeEach(() => {
  exportText.mockReset();
  exportText.mockResolvedValue(true);
});

describe('exportTokens', () => {
  it('includes the value and price in the currency', async () => {
    const [headers, row] = await exportedRows('EUR');

    expect(headers).toEqual([
      'Name',
      'Ticker',
      'Asset ID',
      'Balance',
      'Balance (EUR)',
      'Price (EUR)',
      'Icon URL',
      'Type',
    ]);
    expect(row).toEqual([
      'Chia',
      'XCH',
      '',
      '1500000000000',
      '30',
      '20',
      'https://example.com/xch.png',
      'XCH',
    ]);
  });

  it('leaves out the value and price without a currency', async () => {
    const [headers, row] = await exportedRows(null);

    expect(headers).toEqual([
      'Name',
      'Ticker',
      'Asset ID',
      'Balance',
      'Icon URL',
      'Type',
    ]);
    expect(row).toEqual([
      'Chia',
      'XCH',
      '',
      '1500000000000',
      'https://example.com/xch.png',
      'XCH',
    ]);
  });
});
