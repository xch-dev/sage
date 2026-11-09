// @vitest-environment jsdom

import { TransactionSummary, Unit } from '@/bindings';
import { i18n } from '@lingui/core';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { calculateTransaction } from './AdvancedTransactionSummary';

vi.mock('@/state', () => ({ useWalletState: vi.fn() }));

const xch = { ticker: 'XCH', precision: 12 } as Unit;
const asset = { asset_id: null, kind: 'token' };

function output(coin_id: string, amount: number) {
  return { coin_id, amount, address: 'xch1', receiving: true, burning: false };
}

function input(coin_id: string, amount: number, outputs: unknown[]) {
  return { coin_id, amount, address: 'xch1', asset, outputs };
}

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en-US', messages: {} });
});

describe('calculateTransaction', () => {
  it('does not report coins created and spent in the transaction as spent', () => {
    const summary = {
      fee: 0,
      handles: [],
      inputs: [
        input('base', 1, [output('settlement', 0), output('change', 1)]),
        input('settlement', 0, [output('payment', 5000)]),
        input('payment', 5000, []),
      ],
    } as unknown as TransactionSummary;

    const { spent, created } = calculateTransaction(xch, summary);

    expect(spent.map((coin) => coin.coinId)).toEqual(['base']);
    expect(created).toHaveLength(1);
  });
});
