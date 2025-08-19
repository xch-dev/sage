import { LoaderCircleIcon } from 'lucide-react';
import * as React from 'react';
import { Button, ButtonProps } from './button';

export interface LoadingButtonProps extends ButtonProps {
  loading?: boolean;
  loadingText?: string;
  children: React.ReactNode;
}

const LoadingButton = React.forwardRef<HTMLButtonElement, LoadingButtonProps>(
  (
    {
      loading = false,
      loadingText,
      children,
      disabled,
      'aria-label': ariaLabel,
      ...props
    },
    ref,
  ) => {
    const getLoadingAriaLabel = () => {
      if (ariaLabel) {
        return `${ariaLabel} - ${loadingText || 'Loading'}`;
      }
      return loadingText || 'Loading';
    };

    return (
      <Button
        ref={ref}
        disabled={loading || disabled}
        aria-label={loading ? getLoadingAriaLabel() : ariaLabel}
        aria-busy={loading}
        {...props}
      >
        {loading && (
          <LoaderCircleIcon
            className='mr-2 h-4 w-4 animate-spin'
            aria-hidden='true'
          />
        )}
        {loading && loadingText ? loadingText : children}
      </Button>
    );
  },
);
LoadingButton.displayName = 'LoadingButton';

export { LoadingButton };
