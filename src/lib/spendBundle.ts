import { bls12_381 } from '@noble/curves/bls12-381.js';
import { CoinSpendJson, SpendBundleJson } from '../bindings';

/** The identity (point at infinity) G2 element, used as an empty signature. */
export const IDENTITY_SIGNATURE = `0xc0${'00'.repeat(95)}`;

export class SpendBundleParseError extends Error {}

type JsonObject = Record<string, unknown>;

function isObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function field(obj: JsonObject, snake: string, camel: string): unknown {
  return obj[snake] ?? obj[camel];
}

function hex(value: unknown, name: string, bytes?: number): string {
  if (typeof value !== 'string') {
    throw new SpendBundleParseError(`${name} must be a hex string`);
  }

  const stripped = value.trim().replace(/^0x/i, '').toLowerCase();

  if (!/^[0-9a-f]*$/.test(stripped) || stripped.length % 2 !== 0) {
    throw new SpendBundleParseError(`${name} is not valid hex`);
  }

  if (bytes !== undefined && stripped.length !== bytes * 2) {
    throw new SpendBundleParseError(`${name} must be ${bytes} bytes long`);
  }

  if (bytes === undefined && stripped.length === 0) {
    throw new SpendBundleParseError(`${name} is empty`);
  }

  return `0x${stripped}`;
}

function amount(value: unknown, name: string): string {
  const text = typeof value === 'number' ? value.toString() : value;

  if (typeof text !== 'string' || !/^\d+$/.test(text.trim())) {
    throw new SpendBundleParseError(`${name} must be a non-negative integer`);
  }

  return text.trim();
}

function parseCoinSpend(value: unknown, index: number): CoinSpendJson {
  const name = `coin_spends[${index}]`;

  if (!isObject(value)) {
    throw new SpendBundleParseError(`${name} must be an object`);
  }

  const coin = value.coin;

  if (!isObject(coin)) {
    throw new SpendBundleParseError(`${name}.coin must be an object`);
  }

  return {
    coin: {
      parent_coin_info: hex(
        field(coin, 'parent_coin_info', 'parentCoinInfo'),
        `${name}.coin.parent_coin_info`,
        32,
      ),
      puzzle_hash: hex(
        field(coin, 'puzzle_hash', 'puzzleHash'),
        `${name}.coin.puzzle_hash`,
        32,
      ),
      amount: amount(coin.amount, `${name}.coin.amount`),
    },
    puzzle_reveal: hex(
      field(value, 'puzzle_reveal', 'puzzleReveal'),
      `${name}.puzzle_reveal`,
    ),
    solution: hex(value.solution, `${name}.solution`),
  };
}

/**
 * Parses a JSON spend bundle, accepting snake_case or camelCase keys,
 * hex with or without a 0x prefix, and an optional `spend_bundle` wrapper
 * (as used by the Chia `push_tx` RPC). A missing signature is treated as
 * the identity signature.
 */
export function parseSpendBundle(text: string): SpendBundleJson {
  // Quote bare integer amounts so values above 2^53 keep their precision.
  const quoted = text.replace(/("amount"\s*:\s*)(\d+)/g, '$1"$2"');

  let json: unknown;

  try {
    json = JSON.parse(quoted);
  } catch (error) {
    throw new SpendBundleParseError(
      `Invalid JSON: ${error instanceof Error ? error.message : String(error)}`,
    );
  }

  if (isObject(json)) {
    const wrapped = field(json, 'spend_bundle', 'spendBundle');
    if (isObject(wrapped)) json = wrapped;
  }

  if (!isObject(json)) {
    throw new SpendBundleParseError('Spend bundle must be a JSON object');
  }

  const coinSpends = field(json, 'coin_spends', 'coinSpends');

  if (!Array.isArray(coinSpends)) {
    throw new SpendBundleParseError('Missing coin_spends array');
  }

  if (coinSpends.length === 0) {
    throw new SpendBundleParseError('Spend bundle has no coin spends');
  }

  const signature = field(json, 'aggregated_signature', 'aggregatedSignature');

  const aggregatedSignature =
    signature === undefined || signature === null || signature === ''
      ? IDENTITY_SIGNATURE
      : hex(signature, 'aggregated_signature', 96);

  try {
    bls12_381.longSignatures.Signature.fromHex(aggregatedSignature.slice(2));
  } catch {
    throw new SpendBundleParseError(
      'aggregated_signature is not a valid BLS signature',
    );
  }

  return {
    coin_spends: coinSpends.map(parseCoinSpend),
    aggregated_signature: aggregatedSignature,
  };
}

export function isUnsigned(signature: string): boolean {
  return bls12_381.longSignatures.Signature.fromHex(
    signature.replace(/^0x/i, ''),
  ).is0();
}

export function aggregateSignatures(signatures: string[]): string {
  const { Signature, aggregateSignatures } = bls12_381.longSignatures;

  const aggregated = aggregateSignatures(
    signatures.map((signature) =>
      Signature.fromHex(signature.replace(/^0x/i, '')),
    ),
  );

  return `0x${Signature.toHex(aggregated)}`;
}

/** Serializes a spend bundle with numeric amounts, matching Chia tooling. */
export function stringifySpendBundle(spendBundle: SpendBundleJson): string {
  return JSON.stringify(spendBundle, null, 2).replace(
    /"amount": "(\d+)"/g,
    '"amount": $1',
  );
}
