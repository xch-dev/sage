const MOJO_DECIMAL_PLACES = 12;
const MOJOS_PER_XCH = 10n ** BigInt(MOJO_DECIMAL_PLACES);
const MAX_FEE_MOJOS = (1n << 64n) - 1n;

export interface ParsedFee {
  mojos: string;
  xch: string;
}

function formatXch(mojos: bigint) {
  const whole = mojos / MOJOS_PER_XCH;
  const fraction = (mojos % MOJOS_PER_XCH)
    .toString()
    .padStart(MOJO_DECIMAL_PLACES, '0')
    .replace(/0+$/, '');

  return fraction.length > 0 ? `${whole}.${fraction}` : whole.toString();
}

export function parseMojos(value: string): ParsedFee | null {
  if (!/^\d+$/.test(value)) return null;

  const canonicalValue = value.replace(/^0+/, '') || '0';
  if (canonicalValue.length > 20) return null;

  const mojos = BigInt(canonicalValue);
  if (mojos > MAX_FEE_MOJOS) return null;

  return {
    mojos: mojos.toString(),
    xch: formatXch(mojos),
  };
}

export function parseXchFee(value: string): ParsedFee | null {
  const match = /^(0|[1-9]\d*)(?:\.(\d{0,12}))?$/.exec(value);
  if (!match) return null;
  if (match[1].length > 20) return null;

  const whole = BigInt(match[1]);
  const fraction = BigInt((match[2] ?? '').padEnd(MOJO_DECIMAL_PLACES, '0'));
  const mojos = whole * MOJOS_PER_XCH + fraction;
  if (mojos > MAX_FEE_MOJOS) return null;

  return {
    mojos: mojos.toString(),
    xch: formatXch(mojos),
  };
}
