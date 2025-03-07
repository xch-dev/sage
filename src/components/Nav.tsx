import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { usePeers } from '@/hooks/usePeers';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowLeftRight,
  BookUser,
  Cog,
  Images,
  LogOut,
  MonitorCheck,
  MonitorCog,
  ShoppingCart,
  SquareUserRound,
  WalletIcon,
} from 'lucide-react';
import { PropsWithChildren, useMemo } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Separator } from './ui/separator';

interface NavProps {
  isCollapsed?: boolean;
}

export function TopNav({ isCollapsed }: NavProps) {
  const className = isCollapsed ? 'h-5 w-5' : 'h-4 w-4';

  return (
    <nav
      className={`grid font-medium ${isCollapsed ? 'gap-2' : ''}`}
      role='navigation'
      aria-label={t`Main navigation`}
    >
      <Separator className='mb-3' role='presentation' />
      <NavLink
        url={'/wallet'}
        isCollapsed={isCollapsed}
        message={<Trans>Wallet</Trans>}
        ariaCurrent='page'
      >
        <WalletIcon className={className} aria-hidden='true' />
      </NavLink>
      <NavLink
        url={'/nfts'}
        isCollapsed={isCollapsed}
        message={<Trans>NFTs</Trans>}
      >
        <Images className={className} />
      </NavLink>
      <NavLink
        url={'/dids'}
        isCollapsed={isCollapsed}
        message={<Trans>Profiles</Trans>}
      >
        <SquareUserRound className={className} />
      </NavLink>
      <NavLink
        url={'/offers'}
        isCollapsed={isCollapsed}
        message={<Trans>Offers</Trans>}
      >
        <ShoppingCart className={className} />
      </NavLink>
      <NavLink
        url={'/wallet/addresses'}
        isCollapsed={isCollapsed}
        message={<Trans>Addresses</Trans>}
      >
        <BookUser className={className} />
      </NavLink>
      <NavLink
        url={'/transactions'}
        isCollapsed={isCollapsed}
        message={<Trans>Transactions</Trans>}
      >
        <ArrowLeftRight className={className} />
      </NavLink>
    </nav>
  );
}

export function BottomNav({ isCollapsed }: NavProps) {
  const navigate = useNavigate();

  const { peers } = usePeers();
  const peerCount = peers?.length || 0;

  const walletState = useWalletState();
  const syncedCoins = walletState.sync.synced_coins;
  const totalCoins = walletState.sync.total_coins;
  const isSynced = useMemo(
    () => walletState.sync.synced_coins === walletState.sync.total_coins,
    [walletState.sync.synced_coins, walletState.sync.total_coins],
  );

  const peerMaxHeight =
    peers?.reduce((max, peer) => {
      return Math.max(max, peer.peak_height);
    }, 0) || 0;

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  const className = isCollapsed ? 'h-5 w-5' : 'h-4 w-4';

  return (
    <nav
      className={`grid font-medium ${isCollapsed ? 'gap-2' : ''}`}
      role='navigation'
      aria-label={t`Secondary navigation`}
    >
      <NavLink
        url={'/peers'}
        isCollapsed={isCollapsed}
        message={
          isSynced ? (
            <>
              {peerMaxHeight ? (
                <Trans>{peerCount} peers synced</Trans>
              ) : (
                <Trans>Connecting...</Trans>
              )}
            </>
          ) : (
            <Trans>
              Syncing {syncedCoins} / {totalCoins}
            </Trans>
          )
        }
        customTooltip={
          <>
            {peerCount} {peerCount === 1 ? t`peer` : t`peers`}{' '}
            {isSynced ? (
              peerMaxHeight ? (
                <Trans>synced to peak {peerMaxHeight}</Trans>
              ) : (
                <Trans>connecting...</Trans>
              )
            ) : (
              <Trans>
                with {syncedCoins} / {totalCoins} coins synced
              </Trans>
            )}
          </>
        }
      >
        {isSynced && peerMaxHeight > 0 ? (
          <MonitorCheck
            className={`${className} text-emerald-600`}
            aria-hidden='true'
          />
        ) : (
          <MonitorCog
            className={`${className} text-yellow-600`}
            aria-hidden='true'
          />
        )}
      </NavLink>

      <NavLink
        url={'/settings'}
        isCollapsed={isCollapsed}
        message={<Trans>Settings</Trans>}
      >
        <Cog className={className} aria-hidden='true' />
      </NavLink>

      <NavLink
        url={logout}
        isCollapsed={isCollapsed}
        message={<Trans>Logout</Trans>}
      >
        <LogOut className={className} aria-hidden='true' />
      </NavLink>
    </nav>
  );
}

interface NavLinkProps extends PropsWithChildren {
  url: string | (() => void);
  isCollapsed?: boolean;
  message: React.ReactNode;
  customTooltip?: React.ReactNode;
  ariaCurrent?: 'page' | 'step' | 'location' | 'date' | 'time' | true | false;
}

function NavLink({
  url,
  children,
  isCollapsed,
  message,
  customTooltip,
  ariaCurrent,
}: NavLinkProps) {
  const className = `flex items-center gap-3 rounded-lg py-1.5 text-muted-foreground transition-all hover:text-primary ${
    isCollapsed ? 'justify-center' : 'px-2'
  } text-lg md:text-base`;

  const link =
    typeof url === 'string' ? (
      <Link
        to={url}
        className={className}
        aria-current={ariaCurrent}
        aria-label={isCollapsed ? message?.toString() : undefined}
      >
        {children}
        {!isCollapsed && message}
      </Link>
    ) : (
      <button
        type='button'
        onClick={url}
        className={className}
        aria-label={isCollapsed ? message?.toString() : undefined}
      >
        {children}
        {!isCollapsed && message}
      </button>
    );

  if (isCollapsed || customTooltip) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{link}</TooltipTrigger>
        <TooltipContent side='right' role='tooltip' aria-live='polite'>
          {customTooltip || message}
        </TooltipContent>
      </Tooltip>
    );
  }

  return link;
}
