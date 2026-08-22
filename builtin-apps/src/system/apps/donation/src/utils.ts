import type { AppIcon } from 'sage-app-ui';
import type { SageAppIconView } from 'sage-system-app-sdk';

export type DonationMode = 'usd' | 'xch';

export const DEFAULT_USD = '10';
export const DEFAULT_XCH = '0.05';
export const DEFAULT_FEE_XCH = '0.0001';
export const USD_PRESETS = [2, 5, 10, 20];

export function getTargetAppId() {
  return new URL(window.location.href).searchParams.get('appId');
}

export function xchToMojos(xch: number): string {
  return String(Math.floor(xch * 1_000_000_000_000));
}

export function parsePositiveNumber(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) return null;

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

export function formatXchAmount(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '0';

  for (let decimals = 0; decimals <= 12; decimals++) {
    const rounded = value.toFixed(decimals);
    const roundedNumber = Number(rounded);

    if (roundedNumber <= 0) continue;

    const relativeError = Math.abs(roundedNumber - value) / value;

    if (relativeError < 0.0005) {
      return rounded.replace(/\.?0+$/, '');
    }
  }

  return value.toFixed(12).replace(/\.?0+$/, '');
}

export function appIconFromInline(
  icon: SageAppIconView | null | undefined,
): AppIcon | null {
  if (!icon) return null;

  return {
    kind: 'bytes',
    icon: {
      bytes: icon.bytes,
      mime: icon.mime,
    },
  };
}

export function inlineImageSrc(
  icon: SageAppIconView | null | undefined,
): string | null {
  if (!icon) return null;

  const bytes = new Uint8Array(icon.bytes);
  let binary = '';

  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return `data:${icon.mime};base64,${btoa(binary)}`;
}
