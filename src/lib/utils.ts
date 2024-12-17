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
