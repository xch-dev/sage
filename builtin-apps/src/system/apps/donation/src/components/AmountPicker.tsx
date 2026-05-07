import { RefreshCw } from 'lucide-react';
import { useMemo } from 'react';
import type { DonationMode } from '../utils';
import { formatXchAmount, USD_PRESETS } from '../utils';

interface Props {
  mode: DonationMode;
  setMode: (mode: DonationMode) => void;
  usdInput: string;
  setUsdInput: (value: string) => void;
  xchInput: string;
  setXchInput: (value: string) => void;
  priceUsd: number | null;
  priceLoading: boolean;
  onRefreshPrice: () => void;
  derivedXch: number | null;
  derivedUsd: number | null;
}

function numericInputEquals(a: string, b: string): boolean {
  const aNumber = Number(a.trim());
  const bNumber = Number(b.trim());

  return (
    a.trim() !== '' &&
    b.trim() !== '' &&
    Number.isFinite(aNumber) &&
    Number.isFinite(bNumber) &&
    aNumber === bNumber
  );
}

export function AmountPicker({
  mode,
  setMode,
  usdInput,
  setUsdInput,
  xchInput,
  setXchInput,
  priceUsd,
  priceLoading,
  onRefreshPrice,
  derivedXch,
  derivedUsd,
}: Props) {
  const presets = useMemo(() => {
    if (mode === 'usd') {
      return USD_PRESETS.map((usd) => ({
        key: `usd-${usd}`,
        label: `$${usd}`,
        value: String(usd),
        disabled: false,
      }));
    }

    return USD_PRESETS.map((usd) => {
      const xch = priceUsd ? usd / priceUsd : null;
      const formatted = xch ? formatXchAmount(xch) : null;

      return {
        key: `xch-${usd}`,
        label: formatted ? `${formatted} XCH` : `$${usd}`,
        value: formatted ?? '',
        disabled: !formatted,
      };
    });
  }, [mode, priceUsd]);

  const showPresets = mode === 'usd' || priceUsd !== null;
  const activeInput = mode === 'usd' ? usdInput : xchInput;

  return (
    <div>
      <div className='mb-2 flex items-center justify-between gap-3'>
        <div className='text-sm font-medium'>Amount</div>

        <div className='flex items-center gap-1.5 text-xs text-muted-foreground'>
          {priceLoading ? (
            'Loading XCH price…'
          ) : priceUsd !== null ? (
            `1 XCH ≈ $${priceUsd.toFixed(2)}`
          ) : (
            <>
              <span>XCH price unavailable</span>
              <button
                type='button'
                onClick={onRefreshPrice}
                className='rounded-md p-1 transition-colors hover:bg-muted hover:text-foreground'
                aria-label='Refresh XCH price'
              >
                <RefreshCw className='h-3.5 w-3.5' />
              </button>
            </>
          )}
        </div>
      </div>

      {showPresets ? (
        <div className='mb-2 grid grid-cols-4 gap-2'>
          {presets.map((preset) => {
            const selected =
              !preset.disabled && numericInputEquals(activeInput, preset.value);

            return (
              <button
                key={preset.key}
                type='button'
                disabled={preset.disabled}
                onClick={() => {
                  if (mode === 'usd') {
                    setUsdInput(preset.value);
                  } else {
                    setXchInput(preset.value);
                  }
                }}
                className={[
                  'rounded-lg border px-2 py-1.5 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50',
                  selected
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border hover:bg-muted',
                ].join(' ')}
              >
                {preset.label}
              </button>
            );
          })}
        </div>
      ) : null}

      <div className='flex items-center gap-2'>
        <div className='flex items-center gap-1 rounded-lg border bg-background p-1'>
          <button
            type='button'
            disabled={priceUsd === null}
            onClick={() => setMode('usd')}
            className={[
              'rounded-md px-3 py-1.5 text-sm font-medium',
              mode === 'usd'
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
              priceUsd === null ? 'cursor-not-allowed opacity-50' : '',
            ].join(' ')}
          >
            $
          </button>

          <button
            type='button'
            onClick={() => setMode('xch')}
            className={[
              'rounded-md px-3 py-1.5 text-sm font-medium',
              mode === 'xch'
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
            ].join(' ')}
          >
            XCH
          </button>
        </div>

        <div className='flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-border px-3 py-2'>
          <input
            value={mode === 'usd' ? usdInput : xchInput}
            onChange={(event) => {
              if (mode === 'usd') {
                setUsdInput(event.target.value);
              } else {
                setXchInput(event.target.value);
              }
            }}
            placeholder={mode === 'usd' ? '10.00' : '0.025'}
            inputMode='decimal'
            className='min-w-0 flex-1 bg-transparent text-right text-sm outline-none'
          />
          <span className='text-xs font-medium text-muted-foreground'>
            {mode === 'usd' ? 'USD' : 'XCH'}
          </span>
        </div>
      </div>

      <div className='mt-1 text-right text-xs text-muted-foreground'>
        {mode === 'usd'
          ? derivedXch !== null
            ? `≈ ${formatXchAmount(derivedXch)} XCH`
            : '—'
          : derivedUsd !== null
            ? `≈ $${derivedUsd.toFixed(2)}`
            : '—'}
      </div>
    </div>
  );
}
