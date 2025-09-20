import { AssetKind, NftRecord } from '@/bindings';
import { t } from '@lingui/core/macro';
import { bech32m } from 'bech32';
import BigNumber from 'bignumber.js';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function dbg<T>(value: T): T {
  console.log(value);
  return value;
}

/**
 * Deep merge two theme objects, with values from the second theme overriding values from the first
 */
export function deepMerge<T>(target: T, source: Partial<T>): T {
  const result = { ...target };

  for (const key in source) {
    if (
      source[key] &&
      typeof source[key] === 'object' &&
      !Array.isArray(source[key])
    ) {
      result[key] = deepMerge(
        (result[key] as Record<string, unknown>) || {},
        source[key] as Record<string, unknown>,
      ) as T[Extract<keyof T, string>];
    } else {
      result[key] = source[key] as T[Extract<keyof T, string>];
    }
  }

  return result;
}

export function emptyNftRecord(nftId: string): NftRecord {
  return {
    launcher_id: nftId,
    name: `${nftId.slice(0, 8)}...${nftId.slice(-4)}`,
    icon_url: null,
    collection_id: null,
    collection_name: null,
    minter_did: null,
    owner_did: null,
    visible: false,
    sensitive_content: false,
    created_height: null,
    coin_id: '',
    address: '',
    royalty_address: '',
    royalty_ten_thousandths: 0,
    data_uris: [],
    data_hash: null,
    metadata_uris: [],
    metadata_hash: null,
    license_uris: [],
    license_hash: null,
    edition_number: null,
    edition_total: null,
    created_timestamp: null,
    special_use_type: null,
  };
}
export function getAssetDisplayName(
  name: string | null,
  ticker: string | null,
  kind: AssetKind,
) {
  return (
    name ??
    ticker ??
    (kind === 'token'
      ? t`Unknown CAT`
      : kind === 'option'
        ? 'Untitled Option'
        : t`Untitled ${kind.toUpperCase()}`)
  );
}

export function formatTimestamp(
  timestamp: number | null,
  dateStyle = 'medium',
  timeStyle: string = dateStyle,
): string {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000); // Convert from Unix timestamp to JavaScript timestamp
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: dateStyle as 'full' | 'long' | 'medium' | 'short',
    timeStyle: timeStyle as 'full' | 'long' | 'medium' | 'short',
  }).format(date);
}

export function formatAddress(
  address: string,
  chars = 8,
  trailingChars: number = chars,
): string {
  const cleanAddress = address.startsWith('0x') ? address.slice(2) : address;

  if (chars + trailingChars >= cleanAddress.length) {
    return address;
  }

  return `${cleanAddress.slice(0, chars)}...${cleanAddress.slice(-trailingChars)}`;
}

export function formatUsdPrice(price: number): string {
  if (price < 0.01) {
    return '< 0.01¢';
  } else if (price < 1) {
    return `${(price * 100).toFixed(2)}¢`;
  } else {
    return `$${price.toFixed(2)}`;
  }
}

export function toMojos(amount: string, precision: number): string {
  return BigNumber(amount)
    .multipliedBy(BigNumber(10).pow(precision))
    .toString();
}

export function toDecimal(amount: string | number, precision: number): string {
  return fromMojos(amount, precision).toString();
}

export function fromMojos(
  amount: string | number | BigNumber,
  precision: number,
): BigNumber {
  return BigNumber(amount).dividedBy(BigNumber(10).pow(precision));
}

export interface AddressInfo {
  puzzleHash: string;
  prefix: string;
}

export function toAddress(puzzleHash: string, prefix: string): string {
  return bech32m.encode(
    prefix,
    bech32m.toWords(fromHex(sanitizeHex(puzzleHash))),
  );
}

export function addressInfo(address: string): AddressInfo {
  const { words, prefix } = bech32m.decode(address);
  return {
    puzzleHash: toHex(Uint8Array.from(bech32m.fromWords(words))),
    prefix,
  };
}

export function puzzleHash(address: string): string {
  const info = addressInfo(address);
  return info.puzzleHash;
}

export function isValidAddress(address: string, prefix: string): boolean {
  try {
    const info = addressInfo(address);
    return info.puzzleHash.length === 64 && info.prefix === prefix;
  } catch {
    return false;
  }
}

export function isValidUrl(str: string) {
  try {
    // only allow http(s) schemes, not file, ftp, wss etc
    const trimmed = str.trimStart().toLowerCase();
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) {
      const url = new URL(str);
      // since this is used for nft links, we don't want to allow localhost,
      // or 127.0.0.1 to prevent links to local resources
      return (
        url.hostname.toLowerCase() !== 'localhost' &&
        url.hostname !== '127.0.0.1'
      );
    }
  } catch {
    return false;
  }
}

export function isValidAssetId(assetId: string): boolean {
  return /^[a-fA-F0-9]{64}$/.test(assetId);
}

function sanitizeHex(hex: string): string {
  return hex.replace(/0x/i, '');
}

const HEX_STRINGS = '0123456789abcdef';
const MAP_HEX: Record<string, number> = {
  '0': 0,
  '1': 1,
  '2': 2,
  '3': 3,
  '4': 4,
  '5': 5,
  '6': 6,
  '7': 7,
  '8': 8,
  '9': 9,
  a: 10,
  b: 11,
  c: 12,
  d: 13,
  e: 14,
  f: 15,
  A: 10,
  B: 11,
  C: 12,
  D: 13,
  E: 14,
  F: 15,
};

export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => HEX_STRINGS[b >> 4] + HEX_STRINGS[b & 15])
    .join('');
}

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(Math.floor(hex.length / 2));
  let i;
  for (i = 0; i < bytes.length; i++) {
    const a = MAP_HEX[hex[i * 2]];
    const b = MAP_HEX[hex[i * 2 + 1]];
    if (a === undefined || b === undefined) {
      break;
    }
    bytes[i] = (a << 4) | b;
  }
  return i === bytes.length ? bytes : bytes.slice(0, i);
}

export function decodeHexMessage(hexMessage: string): string {
  return new TextDecoder().decode(fromHex(sanitizeHex(hexMessage)));
}

export function isHex(str: string): boolean {
  return /^(0x)?[0-9a-fA-F]+$/.test(str);
}
