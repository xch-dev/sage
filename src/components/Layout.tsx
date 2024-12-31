import { Cog, LogOut } from 'lucide-react';
import { PropsWithChildren, useMemo } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Plural, Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';

import { useInsets } from '@/contexts/SafeAreaContext';
import useInitialization from '@/hooks/useInitialization';
import { usePeers } from '@/hooks/usePeers';
import { useWallet } from '@/hooks/useWallet';
import icon from '@/icon.png';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { Nav } from './Nav';

export default function Layout(props: PropsWithChildren<object>) {
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

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  return (
    <div className='grid h-screen w-screen md:grid-cols-[250px_1fr]'>
      <div className='hidden border-r bg-muted/40 md:block overflow-y-auto'>
        <div className='flex h-full max-h-screen flex-col gap-2'>
          <div className='flex h-14 items-center pt-2 px-5'>
            <Link
              to='/wallet'
              className='flex items-center gap-2 font-semibold'
            >
              <img src={icon} className='h-8 w-8' alt={t`Wallet icon`} />
              <span className='text-lg'>{wallet?.name}</span>
            </Link>
          </div>
          <div className='flex-1 flex flex-col justify-between pb-4'>
            <nav className='grid items-start px-3 text-sm font-medium'>
              <Nav />
            </nav>

            <nav className='grid px-3 text-sm font-medium'>
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
                {isSynced ? (
                  <>
                    <Plural
                      value={peerCount}
                      one={'# peer'}
                      other={'# peers'}
                    />{' '}
                    {peerMaxHeight
                      ? t`at peak ${peerMaxHeight}`
                      : t`connecting...`}
                  </>
                ) : (
                  <Trans>
                    Syncing {syncedCoins} / {totalCoins}
                  </Trans>
                )}
              </Link>
              <Link
                to={'/settings'}
                className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
              >
                <Cog className='h-4 w-4' aria-hidden='true' />
                <Trans>Settings</Trans>
              </Link>
              <button
                onClick={logout}
                className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
              >
                <LogOut className='h-4 w-4' aria-hidden='true' />
                <Trans>Logout</Trans>
              </button>
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
  );
}
