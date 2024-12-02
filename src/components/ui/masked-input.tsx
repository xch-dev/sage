import * as React from 'react';
import { IMaskMixin } from 'react-imask';
import { Input, InputProps } from './input';

const MaskedInput = IMaskMixin(
  ({ inputRef, ...props }: { inputRef: React.Ref<HTMLInputElement> }) => (
    <Input {...props} ref={inputRef} />
  ),
);

MaskedInput.displayName = 'MaskedInput';

// Extended Masked Input for XCH inputs
interface XchInputProps extends InputProps {
  decimals?: number;
}

const TokenAmountInput = React.forwardRef<HTMLInputElement, XchInputProps>(
  ({ decimals = 12, type = 'text', ...props }, ref) => (
    <MaskedInput
      placeholder='0.00'
      {...props}
      type={type}
      inputRef={ref}
      mask={Number}
      radix='.'
      scale={decimals}
    />
  ),
);

TokenAmountInput.displayName = 'TokenAmountInput';

export { MaskedInput, TokenAmountInput };
