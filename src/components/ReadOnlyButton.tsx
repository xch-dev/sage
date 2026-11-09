import { useWallet } from '@/contexts/WalletContext';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Trans } from '@lingui/react/macro';
import { ComponentProps, ReactNode } from 'react';
import { cn } from '@/lib/utils';

type ButtonProps = ComponentProps<typeof Button> & {
  /** Set for actions that must be signed inline and cannot be exported as an
   *  unsigned transaction (making, taking, or swapping via an offer). These
   *  stay disabled on cold wallets even when the user has opted in to unsigned
   *  transactions. */
  requiresSigning?: boolean;
  /** Overrides the tooltip shown while the button is disabled. */
  disabledTooltip?: ReactNode;
};

export function ReadOnlyButton({
  children,
  onClick,
  requiresSigning = false,
  disabledTooltip,
  ...props
}: ButtonProps) {
  const { isReadOnly, isTransactionDisabled } = useWallet();
  const disabled = requiresSigning ? isReadOnly : isTransactionDisabled;

  if (disabled) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <span className='inline-flex cursor-not-allowed'>
            <Button
              {...props}
              disabled
              className={cn(props.className, 'pointer-events-none')}
            >
              {children}
            </Button>
          </span>
        </TooltipTrigger>
        <TooltipContent>
          {disabledTooltip ??
            (requiresSigning ? (
              <Trans>Cold wallets cannot sign offers</Trans>
            ) : (
              <Trans>Not available for read-only wallets</Trans>
            ))}
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    <Button onClick={onClick} {...props}>
      {children}
    </Button>
  );
}
