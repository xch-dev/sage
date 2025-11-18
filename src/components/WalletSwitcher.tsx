import { commands } from '@/bindings';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { loginAndUpdateState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LogOut, WalletIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface WalletSwitcherProps {
  isCollapsed?: boolean;
  logout: () => void;
}

export function WalletSwitcher({ isCollapsed, logout }: WalletSwitcherProps) {
  const navigate = useNavigate();
  const { wallet: currentWallet, setWallet } = useWallet();
  const { addError } = useErrors();
  const [wallets, setWallets] = useState<
    { name: string; fingerprint: number; emoji: string | null }[]
  >([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchWallets = async () => {
      try {
        const data = await commands.getKeys({});
        setWallets(
          data.keys.map((key) => ({
            name: key.name,
            fingerprint: key.fingerprint,
            emoji: key.emoji,
          })),
        );
      } catch (error) {
        addError(error as CustomError);
      } finally {
        setLoading(false);
      }
    };

    fetchWallets();
  }, [addError]);

  const handleSwitchWallet = async (fingerprint: number) => {
    try {
      await loginAndUpdateState(fingerprint);
      const data = await commands.getKey({});
      setWallet(data.key);
      navigate('/wallet');
    } catch (error) {
      if (
        typeof error === 'object' &&
        error !== null &&
        'kind' in error &&
        error.kind === 'database_migration'
      ) {
        // Handle migration error if needed
        addError(error as CustomError);
      } else {
        addError(error as CustomError);
      }
    }
  };

  const className = isCollapsed ? 'h-5 w-5' : 'h-4 w-4';
  const baseClassName = `flex items-center gap-3 transition-all ${
    isCollapsed ? 'justify-center p-2 rounded-full' : 'px-2 rounded-lg py-1.5'
  } text-lg md:text-base text-muted-foreground hover:text-primary`;

  const trigger = (
    <button
      type='button'
      className={baseClassName}
      aria-label={isCollapsed ? t`Wallet switcher` : undefined}
    >
      <LogOut className={className} aria-hidden='true' />
      {!isCollapsed && <Trans>Logout</Trans>}
    </button>
  );

  const dropdownContent = (
    <DropdownMenuContent align='end' className='w-56'>
      <DropdownMenuLabel>
        <Trans>Switch Wallet</Trans>
      </DropdownMenuLabel>
      <DropdownMenuSeparator />
      {loading ? (
        <DropdownMenuItem disabled>
          <Trans>Loading...</Trans>
        </DropdownMenuItem>
      ) : wallets.length === 0 ? (
        <DropdownMenuItem disabled>
          <Trans>No wallets available</Trans>
        </DropdownMenuItem>
      ) : (
        wallets.map((wallet) => (
          <DropdownMenuItem
            key={wallet.fingerprint}
            onClick={() => handleSwitchWallet(wallet.fingerprint)}
            disabled={currentWallet?.fingerprint === wallet.fingerprint}
            className='flex items-center gap-2'
          >
            {wallet.emoji ? (
              <span className='text-lg' role='img' aria-label='Wallet emoji'>
                {wallet.emoji}
              </span>
            ) : (
              <WalletIcon
                className='h-4 w-4 text-muted-foreground'
                aria-hidden='true'
              />
            )}
            <span className='flex-1'>{wallet.name}</span>
            {currentWallet?.fingerprint === wallet.fingerprint && (
              <span className='text-xs text-muted-foreground'>
                <Trans>(current)</Trans>
              </span>
            )}
          </DropdownMenuItem>
        ))
      )}
      <DropdownMenuSeparator />
      <DropdownMenuItem
        onClick={logout}
        className='text-destructive focus:text-destructive'
      >
        <LogOut className={className} aria-hidden='true' />
        <Trans>Logout</Trans>
      </DropdownMenuItem>
    </DropdownMenuContent>
  );

  const dropdown = (
    <DropdownMenu>
      <Tooltip>
        <TooltipTrigger asChild>
          <DropdownMenuTrigger asChild>{trigger}</DropdownMenuTrigger>
        </TooltipTrigger>
        {isCollapsed && (
          <TooltipContent side='right' role='tooltip' aria-live='polite'>
            <Trans>Wallet switcher</Trans>
          </TooltipContent>
        )}
      </Tooltip>
      {dropdownContent}
    </DropdownMenu>
  );

  return dropdown;
}
