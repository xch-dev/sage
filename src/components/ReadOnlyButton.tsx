import { useWallet } from '@/contexts/WalletContext';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Trans } from '@lingui/react/macro';
import { ComponentProps } from 'react';
import { cn } from '@/lib/utils';

type ButtonProps = ComponentProps<typeof Button>;

export function ReadOnlyButton({ children, onClick, ...props }: ButtonProps) {
  const { isReadOnly } = useWallet();

  if (isReadOnly) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <span className='inline-flex cursor-not-allowed'>
            <Button
              disabled
              {...props}
              className={cn(props.className, 'pointer-events-none')}
            >
              {children}
            </Button>
          </span>
        </TooltipTrigger>
        <TooltipContent>
          <Trans>Not available for read-only wallets</Trans>
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
