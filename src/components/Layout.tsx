import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useInsets } from '@/contexts/SafeAreaContext';
import useInitialization from '@/hooks/useInitialization';
import { usePeers } from '@/hooks/usePeers';
import { useWallet } from '@/hooks/useWallet';
import icon from '@/icon.png';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { Cog, LogOut, PanelLeft, PanelLeftClose } from 'lucide-react';
import { PropsWithChildren, useMemo } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { Nav } from './Nav';

const SIDEBAR_COLLAPSED_STORAGE_KEY = 'sage-wallet-sidebar-collapsed';

type LayoutProps = PropsWithChildren<object> & {
  transparentBackground?: boolean;
};

export default function Layout(props: LayoutProps) {
  const navigate = useNavigate();
  const insets = useInsets();

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

  const initialized = useInitialization();
  const wallet = useWallet(initialized);

  const [isCollapsed, setIsCollapsed] = useLocalStorage<boolean>(
    SIDEBAR_COLLAPSED_STORAGE_KEY,
    false,
  );

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  const bottomNav = (
    <>
      <Link
        to='/peers'
        className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
        aria-label={t`Network status`}
      >
        <span
          className={
            'inline-flex h-3 w-3 m-0.5 rounded-full' +
            ' ' +
            (isSynced ? 'bg-emerald-600' : 'bg-yellow-600')
          }
          aria-hidden='true'
        ></span>
        {!isCollapsed &&
          (isSynced ? (
            <>
              <Plural value={peerCount} one={'# peer'} other={'# peers'} />{' '}
              {peerMaxHeight ? t`at peak ${peerMaxHeight}` : t`connecting...`}
            </>
          ) : (
            <Trans>
              Syncing {syncedCoins} / {totalCoins}
            </Trans>
          ))}
      </Link>
      <Link
        to={'/settings'}
        className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
      >
        <Cog className='h-4 w-4' aria-hidden='true' />
        {!isCollapsed && <Trans>Settings</Trans>}
      </Link>
      <button
        type='button'
        onClick={logout}
        aria-label={t`Logout`}
        className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
      >
        <LogOut className='h-4 w-4' aria-hidden='true' />
        {!isCollapsed && <Trans>Logout</Trans>}
      </button>
    </>
  );

  const bottomNavWithTooltips = isCollapsed ? (
    <>
      <Tooltip>
        <TooltipTrigger asChild>
          <Link
            to='/peers'
            className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
            aria-label={t`Network status`}
          >
            <span
              className={
                'inline-flex h-3 w-3 m-0.5 rounded-full' +
                ' ' +
                (isSynced ? 'bg-emerald-600' : 'bg-yellow-600')
              }
              aria-hidden='true'
            ></span>
          </Link>
        </TooltipTrigger>
        <TooltipContent side='right'>
          {isSynced ? (
            <>
              {peerCount} {peerCount === 1 ? t`peer` : t`peers`}{' '}
              {peerMaxHeight ? (
                <Trans>at peak {peerMaxHeight}</Trans>
              ) : (
                <Trans>(connecting...)</Trans>
              )}
            </>
          ) : (
            <Trans>
              Syncing {syncedCoins} / {totalCoins}
            </Trans>
          )}
        </TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Link
            to={'/settings'}
            className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
          >
            <Cog className='h-4 w-4' aria-hidden='true' />
          </Link>
        </TooltipTrigger>
        <TooltipContent side='right'>
          <Trans>Settings</Trans>
        </TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <button
            type='button'
            onClick={logout}
            aria-label={t`Logout`}
            className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
          >
            <LogOut className='h-4 w-4' aria-hidden='true' />
          </button>
        </TooltipTrigger>
        <TooltipContent side='right'>
          <Trans>Logout</Trans>
        </TooltipContent>
      </Tooltip>
    </>
  ) : (
    bottomNav
  );

  const walletIcon = (
    <Link to='/wallet' className='flex items-center gap-2 font-semibold'>
      <img src={icon} className='h-8 w-8' alt={t`Wallet icon`} />
      <span
        className={`text-lg transition-opacity duration-300 ${
          isCollapsed ? 'opacity-0 hidden' : 'opacity-100'
        }`}
      >
        {wallet?.name}
      </span>
    </Link>
  );

  const walletIconWithTooltip = isCollapsed ? (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link to='/wallet' className='flex items-center gap-2 font-semibold'>
          <img src={icon} className='h-8 w-8' alt={t`Wallet icon`} />
        </Link>
      </TooltipTrigger>
      <TooltipContent side='right'>{wallet?.name ?? t`Wallet`}</TooltipContent>
    </Tooltip>
  ) : (
    walletIcon
  );

  return (
    <TooltipProvider>
      <div className='grid h-screen w-screen md:grid-cols-[auto_1fr]'>
        <div
          className={`hidden border-r bg-muted/40 md:flex flex-col transition-all duration-300 ${
            isCollapsed ? 'w-[60px]' : 'w-[250px]'
          }`}
        >
          <div className='flex h-full max-h-screen flex-col gap-2'>
            <div className='flex h-14 items-center pt-2 px-5 justify-between'>
              {walletIconWithTooltip}
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    type='button'
                    onClick={() => setIsCollapsed(!isCollapsed)}
                    className='text-muted-foreground hover:text-primary transition-colors'
                    aria-label={
                      isCollapsed ? t`Expand sidebar` : t`Collapse sidebar`
                    }
                  >
                    {isCollapsed ? (
                      <PanelLeft className='h-5 w-5' />
                    ) : (
                      <PanelLeftClose className='h-5 w-5' />
                    )}
                  </button>
                </TooltipTrigger>
                <TooltipContent side='right'>
                  {isCollapsed ? t`Expand sidebar` : t`Collapse sidebar`}
                </TooltipContent>
              </Tooltip>
            </div>

            <div className='flex-1 flex flex-col justify-between pb-4'>
              <nav
                className={`grid items-start px-3 text-sm font-medium ${
                  isCollapsed ? 'px-2' : 'px-3'
                }`}
              >
                <Nav isCollapsed={isCollapsed} />
              </nav>

              <nav
                className={`grid text-sm font-medium ${
                  isCollapsed ? 'px-2' : 'px-3'
                }`}
              >
                {bottomNavWithTooltips}
              </nav>
            </div>
          </div>
        </div>
        <div
          className='flex flex-col h-screen overflow-hidden'
          style={{
            paddingTop:
              insets.top !== 0 ? `${insets.top}px` : 'env(safe-area-inset-top)',
          }}
        >
          {props.children}
        </div>
      </div>
    </TooltipProvider>
  );
}
