import { useDefaultFee } from '@/hooks/useDefaultFee';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import * as React from 'react';
import {
  NumericFormat,
  NumericFormatProps,
  SourceInfo,
} from 'react-number-format';
import { toast } from 'react-toastify';
import { Input, InputProps } from './input';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Trans } from '@lingui/react/macro';
import { ArrowUpToLine, PencilLine, Sparkles } from 'lucide-react';
import { AUTO_FEE, getFeeRate, isAutoFee } from '@/lib/fees';

interface MaskedInputProps extends NumericFormatProps<InputProps> {
  inputRef?: React.Ref<HTMLInputElement>;
}

const MaskedInput = React.forwardRef<HTMLInputElement, MaskedInputProps>(
  ({ inputRef, type = 'text', onValueChange, value, ...props }, ref) => (
    <NumericFormat
      onValueChange={onValueChange}
      customInput={Input}
      getInputRef={inputRef || ref}
      displayType='input'
      type={type}
      value={value}
      onPaste={(e: React.ClipboardEvent<HTMLInputElement>) => {
        const pastedText = e.clipboardData.getData('text');
        if (!isLocaleNumber(pastedText)) {
          e.preventDefault();
          toast.error(t`Invalid number ${pastedText}`);
          return;
        }
      }}
      {...props}
    />
  ),
);

function isLocaleNumber(stringNumber: string, locale?: string): boolean {
  try {
    // Use navigator.language as fallback if locale is not provided
    const userLocale = locale || navigator.language;

    // Get the decimal separator for this locale
    const decimalSeparator = Intl.NumberFormat(userLocale)
      .format(1.1)
      .replace(/\p{Number}/gu, '');

    // convert decimal separator to period
    const normalizedNumber = stringNumber.replace(
      new RegExp(`\\${decimalSeparator}`),
      '.',
    );

    // Check if it's a valid number and not NaN
    const parsedNumber = Number(normalizedNumber);
    return !isNaN(parsedNumber) && isFinite(parsedNumber);
  } catch {
    // Return false if there's any error in the parsing process
    return false;
  }
}

MaskedInput.displayName = 'MaskedInput';

// Extended Masked Input for XCH inputs
interface TokenInputProps extends MaskedInputProps {
  precision?: number;
  ticker?: string | null;
  maxValue?: string;
  maxButtonClassName?: string;
  hideMaxButton?: boolean;
}

const TokenAmountInput = React.forwardRef<HTMLInputElement, TokenInputProps>(
  (
    {
      precision = 12,
      ticker = null,
      maxValue,
      maxButtonClassName,
      hideMaxButton = false,
      onChange,
      onValueChange,
      ...props
    },
    ref,
  ) => {
    const walletState = useWalletState();

    const handleMaxClick = () => {
      if (!maxValue) return;
      if (onChange) {
        onChange({
          target: { value: maxValue },
        } as React.ChangeEvent<HTMLInputElement>);
      }
      if (onValueChange) {
        (
          onValueChange as (values: {
            floatValue: number | undefined;
            formattedValue: string;
            value: string;
          }) => void
        )({
          floatValue: parseFloat(maxValue),
          formattedValue: maxValue,
          value: maxValue,
        });
      }
    };

    // Extract height class from className (e.g. 'h-12') to apply to the button
    const heightClass = props.className?.match(/\bh-\d+\b/)?.[0] || 'h-9';

    if (hideMaxButton) {
      return (
        <div className='relative'>
          <MaskedInput
            placeholder='0.00'
            onChange={onChange}
            onValueChange={onValueChange}
            {...props}
            type='text'
            inputRef={ref}
            decimalScale={precision}
            allowLeadingZeros={true}
            allowNegative={false}
          />
          {ticker && (
            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
              <span className='text-muted-foreground text-sm'>
                {ticker === 'xch' ? walletState.sync.unit.ticker : ticker}
              </span>
            </div>
          )}
        </div>
      );
    }

    return (
      <div className='flex'>
        <div className='relative flex-1'>
          <MaskedInput
            placeholder='0.00'
            onChange={onChange}
            onValueChange={onValueChange}
            {...props}
            type='text'
            inputRef={ref}
            decimalScale={precision}
            allowLeadingZeros={true}
            allowNegative={false}
            className={`!rounded-r-none ${props.className || ''}`}
          />
          {ticker && (
            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
              <span className='text-muted-foreground text-sm'>
                {ticker === 'xch' ? walletState.sync.unit.ticker : ticker}
              </span>
            </div>
          )}
        </div>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant='outline'
                type='button'
                tabIndex={-1}
                className={`!border-l-0 !rounded-l-none flex-shrink-0 ${heightClass} w-10 px-0 ${maxButtonClassName || ''}`}
                onClick={handleMaxClick}
                disabled={!maxValue}
                aria-label={t`Use maximum spendable balance`}
              >
                <ArrowUpToLine className='h-4 w-4' />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <Trans>Use maximum spendable balance</Trans>
              <div className='text-xs text-muted-foreground'>
                <Trans>Fees subtracted when applicable</Trans>
              </div>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
    );
  },
);

TokenAmountInput.displayName = 'TokenAmountInput';

// Integer input that only accepts positive integers
interface IntegerInputProps extends MaskedInputProps {
  min?: number;
  max?: number;
}

const IntegerInput = React.forwardRef<HTMLInputElement, IntegerInputProps>(
  ({ min = 0, max, ...props }, ref) => (
    <MaskedInput
      placeholder='0'
      {...props}
      type='text'
      inputRef={ref}
      decimalScale={0}
      allowLeadingZeros={false}
      allowNegative={false}
      isAllowed={(values) => {
        const { floatValue } = values;
        if (floatValue === undefined) return true;

        if (min !== undefined && floatValue < min) return false;
        if (max !== undefined && floatValue > max) return false;

        return true;
      }}
    />
  ),
);

IntegerInput.displayName = 'IntegerInput';

// Fee input that starts from the default fee. Unless `allowAuto` is false, it can
// also be set to calculate the fee automatically, which gives it the value `AUTO_FEE`.
interface FeeAmountInputProps extends Omit<TokenInputProps, 'value'> {
  value?: string;
  className?: string;
  /** Whether the fee can be calculated automatically. */
  allowAuto?: boolean;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onValueChange?: (
    values: {
      floatValue: number | undefined;
      value: string;
    },
    // Absent when the input fills in the default fee or switches modes.
    sourceInfo?: SourceInfo,
  ) => void;
}

const FeeAmountInput = React.forwardRef<HTMLInputElement, FeeAmountInputProps>(
  (
    { value, className, allowAuto = true, onChange, onValueChange, ...props },
    ref,
  ) => {
    const { fee: storedDefaultFee } = useDefaultFee();
    const defaultFee =
      !allowAuto && isAutoFee(storedDefaultFee) ? '' : storedDefaultFee;
    const hasSetInitialValue = React.useRef(false);
    const feeRate = useFeeRate();

    const setValue = React.useCallback(
      (newValue: string) => {
        if (onChange) {
          onChange({
            target: { value: newValue },
          } as React.ChangeEvent<HTMLInputElement>);
        }
        if (onValueChange) {
          onValueChange({
            floatValue: isAutoFee(newValue) ? undefined : Number(newValue),
            value: newValue,
          });
        }
      },
      [onChange, onValueChange],
    );

    // Start from the default fee if the input first appears empty. Only on
    // mount: a value cleared later (by the user, or by its owner resetting
    // it) shouldn't be replaced with the default.
    React.useEffect(() => {
      if (hasSetInitialValue.current) return;
      hasSetInitialValue.current = true;
      if (!value && defaultFee) {
        setValue(defaultFee);
      }
    }, [defaultFee, setValue, value]);

    const currentValue = value ?? defaultFee;
    const heightClass = className?.match(/\bh-\d+\b/)?.[0] || 'h-9';

    const modeButton = (auto: boolean) => (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant='outline'
              type='button'
              className={`!border-l-0 !rounded-l-none flex-shrink-0 ${heightClass} w-10 px-0`}
              onClick={() => setValue(auto ? '' : AUTO_FEE)}
              aria-label={
                auto
                  ? t`Enter a custom fee`
                  : t`Calculate the fee automatically`
              }
            >
              {auto ? (
                <PencilLine className='h-4 w-4' />
              ) : (
                <Sparkles className='h-4 w-4' />
              )}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            {auto ? (
              <Trans>Enter a custom fee</Trans>
            ) : (
              <Trans>Calculate the fee automatically</Trans>
            )}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );

    if (allowAuto && isAutoFee(currentValue)) {
      return (
        <div className='flex'>
          <div
            role='status'
            aria-label={t`Network fee is calculated automatically`}
            className={`flex flex-1 min-w-0 items-center justify-between gap-2 rounded-md rounded-r-none border border-input bg-muted/40 px-3 text-sm ${heightClass}`}
          >
            <span className='flex items-center gap-1.5 truncate'>
              <Sparkles className='h-3.5 w-3.5 shrink-0 text-muted-foreground' />
              <Trans>Automatic</Trans>
            </span>
            {feeRate !== null && (
              <span className='text-muted-foreground text-xs truncate'>
                <Trans>{feeRate} mojos per cost</Trans>
              </span>
            )}
          </div>
          {modeButton(true)}
        </div>
      );
    }

    const input = (
      <TokenAmountInput
        aria-label={t`Network fee amount`}
        {...props}
        ref={ref}
        value={isAutoFee(currentValue) ? '' : currentValue}
        onChange={onChange}
        onValueChange={onValueChange}
        placeholder={t`Enter network fee`}
        className={`pr-12 ${allowAuto ? '!rounded-r-none' : ''} ${className || ''}`}
        ticker='xch'
        precision={12}
        hideMaxButton={true}
      />
    );

    if (!allowAuto) {
      return <div className='relative'>{input}</div>;
    }

    return (
      <div className='flex'>
        <div className='relative flex-1 min-w-0'>{input}</div>
        {modeButton(false)}
      </div>
    );
  },
);

/** The fee rate automatic fees are priced at, once it's loaded. */
function useFeeRate(): number | null {
  const [feeRate, setFeeRate] = React.useState<number | null>(null);

  React.useEffect(() => {
    let cancelled = false;

    getFeeRate()
      .then((rate) => {
        if (!cancelled) setFeeRate(rate);
      })
      .catch(() => {
        // The rate is only shown as a hint, and the request that uses it reports the error.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return feeRate;
}

FeeAmountInput.displayName = 'FeeAmountInput';

export { FeeAmountInput, IntegerInput, MaskedInput, TokenAmountInput };
