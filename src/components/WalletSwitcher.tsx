import { commands, KeyInfo } from '@/bindings';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
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
import { ChevronDown, WalletIcon } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTheme } from 'theme-o-rama';
import iconDark from '@/icon-dark.png';
import iconLight from '@/icon-light.png';

interface WalletSwitcherProps {
  isCollapsed?: boolean;
  wallet?: KeyInfo;
}

export function WalletSwitcher({ isCollapsed, wallet }: WalletSwitcherProps) {
  const navigate = useNavigate();
  const { currentTheme } = useTheme();
  const { setWallet, setIsSwitching, isSwitching } = useWallet();
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
      setIsSwitching(true);
      setWallet(null);
      clearState();

      await new Promise((resolve) => setTimeout(resolve, 300));

      await loginAndUpdateState(fingerprint);
      const data = await commands.getKey({});

      setWallet(data.key);

      await new Promise((resolve) => setTimeout(resolve, 50));

      setIsSwitching(false);
      navigate('/wallet');
    } catch (error) {
      setIsSwitching(false);
      addError(error as CustomError);
      navigate('/');
    }
  };

  const handleMouseEnter = () => {
    if (isMobile) return;
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsHovering(true);
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    if (isMobile) return;
    setIsHovering(false);
    timeoutRef.current = setTimeout(() => {
      setIsOpen(false);
    }, 150);
  };

  // Filter out current wallet from the list
  const otherWallets = wallets.filter(
    (w) => w.fingerprint !== wallet?.fingerprint,
  );

  // Don't show dropdown if there are no other wallets
  const hasOtherWallets = !loading && otherWallets.length > 0;

  // When collapsed, show only the wallet icon/emoji
  if (isCollapsed) {
    if (!hasOtherWallets) {
      // No dropdown needed, just show a tooltip with wallet name
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type='button'
              className={`flex items-center gap-2 font-semibold font-heading ${!wallet ? 'opacity-50 pointer-events-none' : ''}`}
            >
              {wallet?.emoji ? (
                <span className='text-xl' role='img' aria-label='Wallet emoji'>
                  {wallet.emoji}
                </span>
              ) : (
                <img
                  src={
                    currentTheme?.mostLike === 'light' ? iconDark : iconLight
                  }
                  className='h-6 w-6'
                  alt={t`Wallet icon`}
                />
              )}
            </button>
          </TooltipTrigger>
          <TooltipContent side='right'>
            {wallet?.name ?? t`Wallet`}
          </TooltipContent>
        </Tooltip>
      );
    }

    // Show dropdown when collapsed with other wallets available
    return (
      <DropdownMenu open={isOpen} onOpenChange={setIsOpen} modal={false}>
        <Tooltip open={!isHovering && !isOpen ? undefined : false}>
          <TooltipTrigger asChild>
            <DropdownMenuTrigger asChild>
              <button
                type='button'
                className={`flex items-center gap-2 font-semibold font-heading ${!wallet ? 'opacity-50 pointer-events-none' : ''}`}
                onMouseEnter={handleMouseEnter}
                onMouseLeave={handleMouseLeave}
              >
                {wallet?.emoji ? (
                  <span
                    className='text-xl'
                    role='img'
                    aria-label='Wallet emoji'
                  >
                    {wallet.emoji}
                  </span>
                ) : (
                  <img
                    src={
                      currentTheme?.mostLike === 'light' ? iconDark : iconLight
                    }
                    className='h-6 w-6'
                    alt={t`Wallet icon`}
                  />
                )}
              </button>
            </DropdownMenuTrigger>
          </TooltipTrigger>
          <TooltipContent side='right'>
            {wallet?.name ?? t`Wallet`}
          </TooltipContent>
        </Tooltip>
        <DropdownMenuContent
          align='start'
          side='right'
          className='w-56'
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          <div className='px-2 py-1.5 text-sm font-semibold text-muted-foreground'>
            <Trans>Switch Wallet</Trans>
          </div>
          <DropdownMenuSeparator />
          <div className='max-h-[75vh] overflow-y-auto'>
            {otherWallets.map((w) => (
              <DropdownMenuItem
                key={w.fingerprint}
                onClick={() => handleSwitchWallet(w.fingerprint)}
                disabled={isSwitching}
                className='grid grid-cols-[auto_1fr] items-center gap-3 cursor-pointer'
              >
                <div className='w-6 flex items-center justify-center'>
                  {w.emoji ? (
                    <span
                      className='text-lg'
                      role='img'
                      aria-label='Wallet emoji'
                    >
                      {w.emoji}
                    </span>
                  ) : (
                    <WalletIcon
                      className='h-5 w-5 text-muted-foreground'
                      aria-hidden='true'
                    />
                  )}
                </div>
                <span className='truncate'>{w.name}</span>
              </DropdownMenuItem>
            ))}
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // Non-collapsed view
  if (!hasOtherWallets) {
    // No dropdown needed, just show wallet name as a link
    return (
      <button
        type='button'
        className={`flex items-center gap-2 font-semibold font-heading ${!wallet ? 'opacity-50 pointer-events-none' : ''}`}
        onClick={() => navigate('/wallet')}
      >
        {wallet?.emoji ? (
          <span className='text-xl' role='img' aria-label='Wallet emoji'>
            {wallet.emoji}
          </span>
        ) : (
          <img
            src={currentTheme?.mostLike === 'light' ? iconDark : iconLight}
            className='h-6 w-6'
            alt={t`Wallet icon`}
          />
        )}
        <span className='text-lg'>{wallet?.name ?? t`Wallet`}</span>
      </button>
    );
  }

  // Show full dropdown with wallet name and chevron
  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen} modal={false}>
      <DropdownMenuTrigger asChild>
        <button
          type='button'
          className={`flex items-center gap-2 font-semibold font-heading group ${!wallet ? 'opacity-50 pointer-events-none' : ''}`}
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          {wallet?.emoji ? (
            <span className='text-xl' role='img' aria-label='Wallet emoji'>
              {wallet.emoji}
            </span>
          ) : (
            <img
              src={currentTheme?.mostLike === 'light' ? iconDark : iconLight}
              className='h-6 w-6'
              alt={t`Wallet icon`}
            />
          )}
          <span className='text-lg'>{wallet?.name ?? t`Wallet`}</span>
          <ChevronDown
            className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
            aria-hidden='true'
          />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align='start'
        className='w-56'
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <div className='px-2 py-1.5 text-sm font-semibold text-muted-foreground'>
          <Trans>Switch Wallet</Trans>
        </div>
        <DropdownMenuSeparator />
        <div className='max-h-[75vh] overflow-y-auto'>
          {otherWallets.map((w) => (
            <DropdownMenuItem
              key={w.fingerprint}
              onClick={() => handleSwitchWallet(w.fingerprint)}
              disabled={isSwitching}
              className='grid grid-cols-[auto_1fr] items-center gap-3 cursor-pointer'
            >
              <div className='w-6 flex items-center justify-center'>
                {w.emoji ? (
                  <span
                    className='text-lg'
                    role='img'
                    aria-label='Wallet emoji'
                  >
                    {w.emoji}
                  </span>
                ) : (
                  <WalletIcon
                    className='h-5 w-5 text-muted-foreground'
                    aria-hidden='true'
                  />
                )}
              </div>
              <span className='truncate'>{w.name}</span>
            </DropdownMenuItem>
          ))}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
