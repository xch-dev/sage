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

export function toMojos(amount: string, decimals: number): string {
  return BigNumber(amount).multipliedBy(BigNumber(10).pow(decimals)).toString();
}

export function toDecimal(amount: string | number, decimals: number): string {
  return BigNumber(amount).dividedBy(BigNumber(10).pow(decimals)).toString();
}
