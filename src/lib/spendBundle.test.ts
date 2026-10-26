import { describe, expect, it } from 'vitest';
import {
  IDENTITY_SIGNATURE,
  isUnsigned,
  parseSpendBundle,
  SpendBundleParseError,
  stringifySpendBundle,
} from './spendBundle';

const parent = 'aa'.repeat(32);
const puzzleHash = 'bb'.repeat(32);

const coinSpend = {
  coin: {
    parent_coin_info: `0x${parent}`,
    puzzle_hash: `0x${puzzleHash}`,
    amount: 1000,
  },
  puzzle_reveal: '0xff01',
  solution: '0x80',
};

describe('parseSpendBundle', () => {
  it('parses the Sage format', () => {
    const bundle = parseSpendBundle(
      JSON.stringify({
        coin_spends: [coinSpend],
        aggregated_signature: IDENTITY_SIGNATURE,
      }),
    );

    expect(bundle.coin_spends[0].coin.amount).toBe('1000');
    expect(bundle.coin_spends[0].coin.parent_coin_info).toBe(`0x${parent}`);
    expect(isUnsigned(bundle.aggregated_signature)).toBe(true);
  });

  it('accepts camelCase, unprefixed hex and a spend_bundle wrapper', () => {
    const bundle = parseSpendBundle(
      JSON.stringify({
        spend_bundle: {
          coinSpends: [
            {
              coin: {
                parentCoinInfo: parent.toUpperCase(),
                puzzleHash,
                amount: '5',
              },
              puzzleReveal: 'ff01',
              solution: '80',
            },
          ],
        },
      }),
    );

    expect(bundle.coin_spends[0].coin.parent_coin_info).toBe(`0x${parent}`);
    expect(bundle.coin_spends[0].puzzle_reveal).toBe('0xff01');
    expect(bundle.aggregated_signature).toBe(IDENTITY_SIGNATURE);
  });

  it('keeps precision of large amounts', () => {
    const text = `{"coin_spends":[{"coin":{"parent_coin_info":"${parent}","puzzle_hash":"${puzzleHash}","amount":18446744073709551615},"puzzle_reveal":"ff01","solution":"80"}]}`;

    expect(parseSpendBundle(text).coin_spends[0].coin.amount).toBe(
      '18446744073709551615',
    );
  });

  it('rejects invalid input with clear errors', () => {
    expect(() => parseSpendBundle('{')).toThrow(SpendBundleParseError);
    expect(() => parseSpendBundle('{"coin_spends": []}')).toThrow(
      'no coin spends',
    );
    expect(() =>
      parseSpendBundle(
        JSON.stringify({
          coin_spends: [
            { ...coinSpend, coin: { ...coinSpend.coin, puzzle_hash: '0x12' } },
          ],
        }),
      ),
    ).toThrow('coin_spends[0].coin.puzzle_hash must be 32 bytes long');
    expect(() =>
      parseSpendBundle(
        JSON.stringify({
          coin_spends: [coinSpend],
          aggregated_signature: `0x${'ab'.repeat(95)}`,
        }),
      ),
    ).toThrow('aggregated_signature must be 96 bytes long');
  });

  it('rejects decimal and exponent amounts as amounts, not JSON', () => {
    for (const value of ['1.0', '1e3', '-1']) {
      const text = `{"coin_spends":[{"coin":{"parent_coin_info":"${parent}","puzzle_hash":"${puzzleHash}","amount":${value}},"puzzle_reveal":"ff01","solution":"80"}]}`;

      expect(() => parseSpendBundle(text)).toThrow(
        'coin_spends[0].coin.amount must be a non-negative integer',
      );
    }
  });
});

describe('isUnsigned', () => {
  it('detects the identity signature', () => {
    expect(isUnsigned(IDENTITY_SIGNATURE)).toBe(true);
    expect(isUnsigned(IDENTITY_SIGNATURE.slice(2).toUpperCase())).toBe(true);
    expect(isUnsigned(`0x${'ab'.repeat(96)}`)).toBe(false);
  });
});

describe('stringifySpendBundle', () => {
  it('round trips', () => {
    const bundle = parseSpendBundle(
      JSON.stringify({ coin_spends: [coinSpend] }),
    );

    expect(parseSpendBundle(stringifySpendBundle(bundle))).toEqual(bundle);
    expect(stringifySpendBundle(bundle)).toContain('"amount": 1000');
  });
});
