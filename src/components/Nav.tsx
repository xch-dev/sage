import { usePeers } from '@/hooks/usePeers';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  ArrowDownUp,
  ArrowLeftRight,
  BookUser,
  Cog,
  FilePenLine,
  Handshake,
  Images,
  MonitorCheck,
  MonitorCog,
  SquareUserRound,
  WalletIcon,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { NavLink } from './NavLink';
import { Separator } from './ui/separator';
import { WalletSwitcher } from './WalletSwitcher';

interface NavProps {
  isCollapsed?: boolean;
}

export function TopNav({ isCollapsed }: NavProps) {
  const className = isCollapsed ? 'h-5 w-5' : 'h-4 w-4';

  const isIos = platform() === 'ios';

  return (
    <nav
      className={`grid font-medium font-body ${isCollapsed ? 'gap-1' : ''}`}
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
        <SquareUserRound className={className} aria-hidden='true' />
      </NavLink>

      {!isIos && (
        <NavLink
          url={'/options'}
          isCollapsed={isCollapsed}
          message={<Trans>Options</Trans>}
        >
          <FilePenLine className={className} aria-hidden='true' />
        </NavLink>
      )}

      <NavLink
        url={'/offers'}
        isCollapsed={isCollapsed}
        message={<Trans>Offers</Trans>}
      >
        <Handshake className={className} aria-hidden='true' />
      </NavLink>

      {!isIos && (
        <NavLink
          url={'/swap'}
          isCollapsed={isCollapsed}
          message={<Trans>Swap</Trans>}
        >
          <ArrowLeftRight className={className} aria-hidden='true' />
        </NavLink>
      )}

      <NavLink
        url={'/addresses'}
        isCollapsed={isCollapsed}
        message={<Trans>Addresses</Trans>}
      >
        <BookUser className={className} aria-hidden='true' />
      </NavLink>

      <NavLink
        url={'/transactions'}
        isCollapsed={isCollapsed}
        message={<Trans>Transactions</Trans>}
      >
        <ArrowDownUp className={className} />
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
  const checkedFiles = walletState.sync.checked_files;
  const totalFiles = walletState.sync.total_files;

  const coinsSynced =
    walletState.sync.synced_coins === walletState.sync.total_coins;
  const filesSynced =
    walletState.sync.checked_files === walletState.sync.total_files;
  const isSynced = coinsSynced && filesSynced;

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
      className={`grid font-medium font-body ${isCollapsed ? 'gap-1' : ''}`}
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
          ) : coinsSynced ? (
            <Trans>
              Downloading {checkedFiles} / {totalFiles}
            </Trans>
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

      <WalletSwitcher isCollapsed={isCollapsed} logout={logout} />
    </nav>
  );
}
