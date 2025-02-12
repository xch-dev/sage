import * as React from 'react';
import { Input, InputProps } from './input';
import { NumericFormat } from 'react-number-format';

interface MaskedInputProps extends InputProps {
  inputRef?: React.Ref<HTMLInputElement>;
  onValueChange?: (values: { floatValue?: number }) => void;
  allowLeadingZeros?: boolean;
  allowDecimal?: boolean;
  allowNegative?: boolean;
  allowThousands?: boolean;
  allowLeadingZeroScale?: boolean;
  allowDecimalScale?: boolean;
  allowLeadingZeroWidth?: boolean;
  allowLeadingZeroWidthScale?: boolean;
  type?: 'text';
  decimalScale?: number;
}

const MaskedInput = React.forwardRef<HTMLInputElement, MaskedInputProps>(
  ({ inputRef, type = 'text', onValueChange, ...props }, ref) => (
    <NumericFormat
      onValueChange={onValueChange}
      customInput={Input}
      getInputRef={inputRef || ref}
      displayType='input'
      type={type}
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
      decimalScale={12}
      allowLeadingZeros={false}
      allowDecimal={true}
      allowNegative={false}
      allowThousands={true}
      allowLeadingZeroScale={false}
      allowDecimalScale={true}
      allowLeadingZeroWidth={false}
      allowLeadingZeroWidthScale={false}
    />
  ),
);

TokenAmountInput.displayName = 'TokenAmountInput';

export { MaskedInput, TokenAmountInput };
