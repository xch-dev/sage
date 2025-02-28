import { t } from '@lingui/core/macro';
import * as React from 'react';
import { NumericFormat, NumericFormatProps } from 'react-number-format';
import { toast } from 'react-toastify';
import { Input, InputProps } from './input';

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
  } catch (error) {
    // Return false if there's any error in the parsing process
    return false;
  }
}

MaskedInput.displayName = 'MaskedInput';

// Extended Masked Input for XCH inputs
interface XchInputProps extends MaskedInputProps {
  decimals?: number;
}

const TokenAmountInput = React.forwardRef<HTMLInputElement, XchInputProps>(
  ({ decimals = 12, ...props }, ref) => (
    <MaskedInput
      placeholder='0.00'
      {...props}
      type='text'
      inputRef={ref}
      decimalScale={decimals}
      allowLeadingZeros={false}
      allowNegative={false}
    />
  ),
);

TokenAmountInput.displayName = 'TokenAmountInput';

export { MaskedInput, TokenAmountInput };
