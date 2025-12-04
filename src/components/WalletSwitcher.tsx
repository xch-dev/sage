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
import { clearState, loginAndUpdateState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { LogOut, WalletIcon } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface WalletSwitcherProps {
  isCollapsed?: boolean;
  logout: () => void;
}

export function WalletSwitcher({ isCollapsed, logout }: WalletSwitcherProps) {
  const navigate = useNavigate();
  const {
    wallet: currentWallet,
    setWallet,
    setIsSwitching,
    isSwitching,
  } = useWallet();
  const { addError } = useErrors();
  const [wallets, setWallets] = useState<
    { name: string; fingerprint: number; emoji: string | null }[]
  >([]);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);
  const [isHovering, setIsHovering] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    const fetchWallets = async () => {
      try {
        const data = await commands.getKeys({});
        setWallets(
          data.keys
            .map((key) => ({
              name: key.name,
              fingerprint: key.fingerprint,
              emoji: key.emoji,
            }))
            .sort((a, b) => a.name.localeCompare(b.name)),
        );
      } catch (error) {
        addError(error as CustomError);
      } finally {
        setLoading(false);
      }
    };

    fetchWallets();
  }, [addError]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleSwitchWallet = async (fingerprint: number) => {
    if (isSwitching) {
      return;
    }

    try {
      // Start switching: clear wallet, state, and set switching state
      setIsSwitching(true);
      setWallet(null);
      clearState();

      // Wait for fade-out transition to complete
      await new Promise((resolve) => setTimeout(resolve, 300));

      // Load new wallet data while still blurred
      await loginAndUpdateState(fingerprint);
      const data = await commands.getKey({});

      // Set new wallet data while still blurred
      setWallet(data.key);

      // Wait a moment for the new data to be set, then fade in
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Now fade in the new wallet
      setIsSwitching(false);
      navigate('/wallet');
    } catch (error) {
      setIsSwitching(false);
      addError(error as CustomError);
      navigate('/');
    }
  };

  const handleMouseEnter = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsHovering(true);
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    setIsHovering(false);
    timeoutRef.current = setTimeout(() => {
      setIsOpen(false);
    }, 150);
  };

  const className = isCollapsed ? 'h-5 w-5' : 'h-4 w-4';
  const baseClassName = `flex items-center gap-3 transition-all ${
    isCollapsed ? 'justify-center p-2 rounded-full' : 'px-2 rounded-lg py-1.5'
  } text-lg md:text-base text-muted-foreground hover:text-primary`;

  // If only one wallet, show simple logout button
  if (!loading && wallets.length <= 1) {
    return (
      <button
        type='button'
        className={baseClassName}
        onClick={logout}
        disabled={isSwitching}
        aria-label={isCollapsed ? t`Logout` : undefined}
      >
        <LogOut className={className} aria-hidden='true' />
        {!isCollapsed && <Trans>Logout</Trans>}
      </button>
    );
  }

  const trigger = (
    <button
      type='button'
      className={baseClassName}
      aria-label={isCollapsed ? t`Wallet switcher` : undefined}
      {...(isMobile
        ? {}
        : {
            onMouseEnter: handleMouseEnter,
            onMouseLeave: handleMouseLeave,
          })}
    >
      <LogOut className={className} aria-hidden='true' />
      {!isCollapsed && <Trans>Wallets</Trans>}
    </button>
  );

  const dropdownContent = (
    <DropdownMenuContent
      align='end'
      className='w-56'
      {...(isMobile
        ? {}
        : {
            onMouseEnter: handleMouseEnter,
            onMouseLeave: handleMouseLeave,
          })}
    >
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
            disabled={
              isSwitching || currentWallet?.fingerprint === wallet.fingerprint
            }
            className='grid grid-cols-[auto_1fr_auto] items-center gap-3'
          >
            <div className='w-6 flex items-center justify-center'>
              {wallet.emoji ? (
                <span className='text-lg' role='img' aria-label='Wallet emoji'>
                  {wallet.emoji}
                </span>
              ) : (
                <WalletIcon
                  className='h-5 w-5 text-muted-foreground'
                  aria-hidden='true'
                />
              )}
            </div>
            <span className='truncate'>{wallet.name}</span>
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
        disabled={isSwitching}
        className='text-destructive focus:text-destructive'
      >
        <LogOut className={className} aria-hidden='true' />
        <Trans>Logout</Trans>
      </DropdownMenuItem>
    </DropdownMenuContent>
  );

  // If multiple wallets, show dropdown menu
  // On mobile, use click to open/close. On desktop, use hover.
  const dropdown = (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen} modal={false}>
      <Tooltip open={isCollapsed && !isHovering && !isOpen ? undefined : false}>
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
