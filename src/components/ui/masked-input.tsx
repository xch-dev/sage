import * as React from 'react';
import { Input, InputProps } from './input';
import { NumericFormat, NumericFormatProps } from 'react-number-format';

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
      {...props}
    />
  ),
);

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