import { bls12_381 } from '@noble/curves/bls12-381.js';
import { describe, expect, it } from 'vitest';
import {
  aggregateSignatures,
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

function signature(seed: number) {
  const { longSignatures } = bls12_381;
  const sk = new Uint8Array(32).fill(seed);
  const message = longSignatures.hash(new Uint8Array([seed]));
  return `0x${longSignatures.Signature.toHex(longSignatures.sign(message, sk))}`;
}

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
          aggregated_signature: `0x${'ab'.repeat(96)}`,
        }),
      ),
    ).toThrow('not a valid BLS signature');
  });
});

describe('aggregateSignatures', () => {
  it('treats the identity signature as a no-op', () => {
    const sig = signature(1);

    expect(isUnsigned(sig)).toBe(false);
    expect(aggregateSignatures([IDENTITY_SIGNATURE, sig])).toBe(sig);
  });

  it('is order independent', () => {
    const a = signature(1);
    const b = signature(2);

    expect(aggregateSignatures([a, b])).toBe(aggregateSignatures([b, a]));
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
