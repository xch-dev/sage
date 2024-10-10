import { Cog, Images, LogOut, Wallet as WalletIcon } from 'lucide-react';
import { PropsWithChildren, useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';

import icon from '@/icon.png';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { usePeers } from '@/contexts/PeerContext';

export default function Layout(props: PropsWithChildren<object>) {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const { peers } = usePeers();

  const walletState = useWalletState();
  const isSynced = useMemo(
    () => walletState.sync.synced_coins === walletState.sync.total_coins,
    [walletState.sync.synced_coins, walletState.sync.total_coins],
  );

  const peerMaxHeight =
    peers?.reduce((max, peer) => {
      return Math.max(max, peer.peak_height);
    }, 0) || 0;

  useEffect(() => {
    commands.activeWallet().then((wallet) => {
      if (wallet.status === 'error') {
        return;
      }
      if (wallet.data) return setWallet(wallet.data);
      navigate('/');
    });
  }, [navigate]);

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  return (
    <>
      <div className='grid h-screen w-screen md:grid-cols-[280px_1fr]'>
        <div className='hidden border-r bg-muted/40 md:block overflow-y-auto'>
          <div className='flex h-full max-h-screen flex-col gap-2'>
            <div className='flex h-14 items-center px-4 lg:h-[60px] lg:px-6'>
              <Link to='/' className='flex items-center gap-2 font-semibold'>
                <img src={icon} className='h-6 w-6' />
                <span className=''>{wallet?.name}</span>
              </Link>
            </div>
            <div className='flex-1 flex flex-col justify-between pb-4'>
              <nav className='grid items-start px-2 text-sm font-medium lg:px-4'>
                <Link
                  to={'/wallet'}
                  className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
                >
                  <WalletIcon className='h-4 w-4' />
                  Assets
                </Link>
                <Link
                  to={'/nfts'}
                  className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
                >
                  <Images className='h-4 w-4' />
                  NFTs
                </Link>
              </nav>

              <nav className='grid px-2 text-sm font-medium lg:px-4'>
                <Link
                  to='/peers'
                  className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
                >
                  <span
                    className={
                      'inline-flex h-2 w-2 m-1 rounded-full' +
                      ' ' +
                      (isSynced ? 'bg-emerald-600' : 'bg-yellow-600')
                    }
                  ></span>
                  {isSynced ? (
                    <>
                      {peers?.length} peers,
                      {peerMaxHeight
                        ? ` ${peerMaxHeight} peak`
                        : ' connecting...'}
                    </>
                  ) : (
                    `Syncing ${walletState.sync.synced_coins} / ${walletState.sync.total_coins} coins`
                  )}
                </Link>
                <Link
                  to={'/settings'}
                  className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
                >
                  <Cog className='h-4 w-4' />
                  Settings
                </Link>
                <button
                  onClick={logout}
                  className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
                >
                  <LogOut className='h-4 w-4' />
                  Logout
                </button>
              </nav>
            </div>
          </div>
        </div>
        <div className='flex flex-col h-screen overflow-auto'>
          {props.children}
        </div>
      </div>
    </>
  );
}
