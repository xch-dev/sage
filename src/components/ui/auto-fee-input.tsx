import { FeeTierName } from '@/bindings';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectSeparator,
  SelectTrigger,
} from '@/components/ui/select';
import { useFeeRates } from '@/hooks/useFeeRates';
import {
  autoFeeAtRate,
  autoFeeAtTier,
  autoFeeRate,
  autoFeeRateFor,
  autoFeeTier,
  FEE_TIERS,
  feeTierLabel,
  feeTierTime,
  formatRate,
  tierFeeRate,
} from '@/lib/fees';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Sparkles, TriangleAlert } from 'lucide-react';
import * as React from 'react';
import { NumericFormat } from 'react-number-format';

/** The select value of an automatic fee at a rate the user enters. */
const CUSTOM_RATE = 'custom';

interface AutoFeeInputProps {
  /** An automatic fee, at a tier or a rate the user enters (see `isAutoFee`). */
  value: string;
  onValueChange: (value: string) => void;
  onKeyDown?: React.KeyboardEventHandler<HTMLInputElement>;
  heightClass: string;
  /** Shown at the end of the input, to switch to entering an amount. */
  modeButton: React.ReactNode;
  className?: string;
}

/**
 * The fee input of a fee calculated from the transaction's cost: the speed to pay for, with the
 * rate each speed is estimated at, or a rate the user enters.
 */
export function AutoFeeInput({
  value,
  onValueChange,
  onKeyDown,
  heightClass,
  modeButton,
  className,
}: AutoFeeInputProps) {
  const feeRates = useFeeRates();
  const tier = autoFeeTier(value);
  // What's typed in the rate input, which can be empty while it's edited.
  const enteredRate = autoFeeRate(value);
  const effectiveRate = autoFeeRateFor(value, feeRates);
  const minimumRate = feeRates?.minimum_fee_per_cost;
  const belowMinimum =
    tier === null &&
    effectiveRate !== null &&
    minimumRate !== undefined &&
    effectiveRate < minimumRate;
  // Without estimates, such as on a network that Coinset doesn't serve, every tier pays the
  // recommended rate. Until the rates load, it isn't known yet.
  const unestimated = feeRates !== null && feeRates.tiers.length === 0;
  const shownRate = effectiveRate !== null ? formatRate(effectiveRate) : null;
  const tierRate = (name: FeeTierName) =>
    feeRates ? formatRate(tierFeeRate(feeRates, name)) : null;

  const selectSpeed = (selected: string) => {
    if (selected === CUSTOM_RATE) {
      // Start from the rate it was paying, so it can be adjusted from there.
      if (tier !== null) onValueChange(autoFeeAtRate(shownRate ?? ''));
      return;
    }

    onValueChange(autoFeeAtTier(selected as FeeTierName));
  };

  return (
    <div className={className}>
      <div className='flex'>
        <div
          className={`flex flex-1 min-w-0 items-center justify-between gap-1 rounded-md rounded-r-none border border-input bg-muted/40 pr-1.5 text-sm focus-within:ring-1 focus-within:ring-ring ${heightClass}`}
        >
          <Select value={tier ?? CUSTOM_RATE} onValueChange={selectSpeed}>
            {/* The speed is never cut off; the rate beside it gives way instead. */}
            <SelectTrigger
              aria-label={t`Fee speed`}
              className='h-full w-auto shrink-0 gap-1 border-0 bg-transparent pl-3 pr-1 shadow-none focus:ring-0'
            >
              <div className='flex items-center gap-1.5'>
                {/* A custom rate has its input to show it's automatic, and needs the room. */}
                {tier !== null && (
                  <Sparkles className='h-3.5 w-3.5 shrink-0 text-muted-foreground' />
                )}
                <span>{tier !== null ? feeTierLabel(tier) : t`Custom`}</span>
              </div>
            </SelectTrigger>
            <SelectContent className='min-w-[15rem]'>
              {FEE_TIERS.map((name) => {
                const rate = tierRate(name);
                return (
                  <SelectItem key={name} value={name}>
                    <span className='flex flex-col'>
                      <span>{feeTierLabel(name)}</span>
                      <span className='text-xs text-muted-foreground'>
                        {unestimated ? t`Recommended rate` : feeTierTime(name)}
                        {rate !== null && (
                          <>
                            {' · '}
                            <Trans>{rate} mojos/cost</Trans>
                          </>
                        )}
                      </span>
                    </span>
                  </SelectItem>
                );
              })}
              <SelectSeparator />
              <SelectItem value={CUSTOM_RATE}>
                <span className='flex flex-col'>
                  <span>
                    <Trans>Custom rate</Trans>
                  </span>
                  <span className='text-xs text-muted-foreground'>
                    <Trans>Enter mojos per cost</Trans>
                  </span>
                </span>
              </SelectItem>
              {unestimated && (
                <p className='max-w-[15rem] px-2 pb-1.5 pt-1 text-xs text-muted-foreground'>
                  <Trans>
                    Fee estimates aren&apos;t available, so every speed pays the
                    recommended rate.
                  </Trans>
                </p>
              )}
            </SelectContent>
          </Select>
          {tier !== null ? (
            <span
              className='min-w-0 truncate text-xs text-muted-foreground'
              aria-label={t`Fee rate in mojos per cost`}
            >
              {shownRate !== null ? <Trans>{shownRate} mojos/cost</Trans> : '…'}
            </span>
          ) : (
            // A label, so that clicking the unit edits the rate too.
            <label className='flex min-w-0 cursor-text items-center gap-1 text-xs text-muted-foreground'>
              <NumericFormat
                aria-label={t`Fee rate in mojos per cost`}
                value={enteredRate ?? ''}
                placeholder={feeRates ? formatRate(feeRates.fee_per_cost) : ''}
                onValueChange={(values, sourceInfo) => {
                  if (sourceInfo.source === 'prop') return;
                  onValueChange(autoFeeAtRate(values.value));
                }}
                onKeyDown={onKeyDown}
                decimalScale={3}
                allowNegative={false}
                allowLeadingZeros={false}
                className={`h-7 w-12 min-w-0 shrink rounded-sm border border-input bg-background px-1.5 text-left text-sm text-foreground focus:outline-none ${belowMinimum ? 'border-amber-500 text-amber-600 dark:text-amber-400' : ''}`}
              />
              <span className='shrink-0 whitespace-nowrap'>
                <Trans>mojos/cost</Trans>
              </span>
            </label>
          )}
        </div>
        {modeButton}
      </div>
      {belowMinimum && (
        <p
          role='alert'
          className='mt-1.5 flex items-start gap-1 text-xs text-amber-600 dark:text-amber-400'
        >
          <TriangleAlert className='mt-px h-3.5 w-3.5 shrink-0' />
          <Trans>
            Below the minimum of {minimumRate} mojos per cost. The transaction
            may never be confirmed.
          </Trans>
        </p>
      )}
    </div>
  );
}
