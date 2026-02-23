import { useDefaultFee } from '@/hooks/useDefaultFee';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import * as React from 'react';
import { NumericFormat, NumericFormatProps } from 'react-number-format';
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
import { ArrowUpToLine } from 'lucide-react';

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
              >
                <ArrowUpToLine className='h-4 w-4' />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <Trans>Use maximum spendable balance</Trans>
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

// Fee input that uses the default fee value as initial value
interface FeeAmountInputProps extends Omit<TokenInputProps, 'value'> {
  value?: string;
  className?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onValueChange?: (values: {
    floatValue: number | undefined;
    value: string;
  }) => void;
}

const FeeAmountInput = React.forwardRef<HTMLInputElement, FeeAmountInputProps>(
  ({ value, className, onChange, onValueChange, ...props }, ref) => {
    const { fee: defaultFee } = useDefaultFee();
    const hasSetInitialValue = React.useRef(false);

    // Set initial value when component mounts
    React.useEffect(() => {
      if (!value && !hasSetInitialValue.current) {
        hasSetInitialValue.current = true;
        if (onChange) {
          onChange({
            target: { value: defaultFee },
          } as React.ChangeEvent<HTMLInputElement>);
        }
        if (onValueChange) {
          onValueChange({ floatValue: Number(defaultFee), value: defaultFee });
        }
      }
    }, [defaultFee, onChange, onValueChange, value]);

    return (
      <div className='relative'>
        <TokenAmountInput
          {...props}
          ref={ref}
          value={value ?? defaultFee}
          onChange={onChange}
          onValueChange={onValueChange}
          placeholder={t`Enter network fee`}
          aria-label={t`Network fee amount`}
          className={`pr-12 ${className || ''}`}
          ticker='xch'
          precision={12}
          hideMaxButton={true}
        />
      </div>
    );
  },
);

FeeAmountInput.displayName = 'FeeAmountInput';

export { FeeAmountInput, IntegerInput, MaskedInput, TokenAmountInput };
