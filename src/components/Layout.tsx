import { Cog, LogOut } from 'lucide-react';
import { PropsWithChildren, useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';

import { usePeers } from '@/contexts/PeerContext';
import icon from '@/icon.png';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { navItems } from './Nav';
import { Separator } from './ui/separator';

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
              <Link
                to='/wallet'
                className='flex items-center gap-2 font-semibold'
              >
                <img src={icon} className='h-6 w-6' />
                {wallet?.name}
              </Link>
            </div>
            <div className='flex-1 flex flex-col justify-between pb-4'>
              <nav className='grid items-start px-2 text-sm font-medium lg:px-4'>
                {navItems.map((item, i) => {
                  switch (item.type) {
                    case 'link': {
                      const Icon = item.icon;
                      return (
                        <NavLink key={i} url={item.url}>
                          <Icon className='h-4 w-4' />
                          {item.label}
                        </NavLink>
                      );
                    }
                    case 'separator': {
                      return <Separator className='my-2' key={i} />;
                    }
                  }
                })}
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
                      {peers?.length} peers
                      {peerMaxHeight
                        ? ` | peak ${peerMaxHeight}`
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

interface NavLinkProps {
  url: string;
}

function NavLink({ url, children }: PropsWithChildren<NavLinkProps>) {
  return (
    <Link
      to={url}
      className='flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
    >
      {children}
    </Link>
  );
}
