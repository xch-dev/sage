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

export function formatAddress(address: string, chars: number = 8): string {
  const cleanAddress = address.startsWith('0x') ? address.slice(2) : address;

  if (chars * 2 >= cleanAddress.length) {
    return address;
  }

  return `${cleanAddress.slice(0, chars)}...${cleanAddress.slice(-chars)}`;
}

export function toMojos(amount: string, decimals: number): string {
  return BigNumber(amount).multipliedBy(BigNumber(10).pow(decimals)).toString();
}

export function toDecimal(amount: string | number, decimals: number): string {
  return BigNumber(amount).dividedBy(BigNumber(10).pow(decimals)).toString();
}

export interface AddressInfo {
  puzzleHash: string;
  prefix: string;
}

export function toAddress(puzzleHash: string, prefix: string): string {
  return bech32m.encode(
    prefix,
    convertBits(fromHex(sanitizeHex(puzzleHash)), 8, 5, true),
  );
}

export function addressInfo(address: string): AddressInfo {
  const { words, prefix } = bech32m.decode(address);
  return {
    puzzleHash: toHex(convertBits(Uint8Array.from(words), 5, 8, false)),
    prefix,
  };
}

export function puzzleHash(address: string): string {
  const info = addressInfo(address);
  return info.puzzleHash;
}

function sanitizeHex(hex: string): string {
  return hex.replace(/0x/i, '');
}

function formatHex(hex: string): string {
  return /^0x/i.test(hex) ? hex : `0x${hex}`;
}

function convertBits(
  bytes: Uint8Array,
  from: number,
  to: number,
  pad: boolean,
): Uint8Array {
  let accumulate = 0;
  let bits = 0;
  const maxv = (1 << to) - 1;
  const result = [];
  for (const value of bytes) {
    const b = value & 0xff;
    if (b < 0 || b >> from > 0) throw new Error('Could not convert bits.');
    accumulate = (accumulate << from) | b;
    bits += from;
    while (bits >= to) {
      bits -= to;
      result.push((accumulate >> bits) & maxv);
    }
  }
  if (pad && bits > 0) {
    result.push((accumulate << (to - bits)) & maxv);
  } else if (bits >= from || ((accumulate << (to - bits)) & maxv) != 0) {
    throw new Error('Could not convert bits.');
  }
  return Uint8Array.from(result);
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

function toHex(bytes: Uint8Array): string {
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
