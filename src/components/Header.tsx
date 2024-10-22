import { usePeers } from '@/contexts/PeerContext';
import { useWallet } from '@/hooks/useWallet';
import icon from '@/icon.png';
import { logoutAndUpdateState, useWalletState } from '@/state';
import { ChevronLeft, Cog, LogOut, Menu } from 'lucide-react';
import { PropsWithChildren, ReactNode, useMemo } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { navItems } from './Nav';
import { Button } from './ui/button';
import { Separator } from './ui/separator';
import { Sheet, SheetContent, SheetTrigger } from './ui/sheet';

export default function Header(
  props: PropsWithChildren<{
    title: string;
    back?: () => void;
    children?: ReactNode;
  }>,
) {
  const navigate = useNavigate();
  const location = useLocation();

  const { wallet } = useWallet();
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

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  const hasBackButton = props.back || location.pathname.split('/').length > 2;

  return (
    <header className='flex items-center gap-4 px-4 lg:px-6 pt-2 sticky top-0 bg-background z-10'>
      <Sheet>
        {hasBackButton ? (
          <Button
            variant='outline'
            size='icon'
            onClick={() => (props.back ? props.back() : navigate(-1))}
            className='md:hidden text-muted-foreground'
          >
            <ChevronLeft className='h-5 w-5' />
            <span className='sr-only'>Back</span>
          </Button>
        ) : (
          <SheetTrigger asChild>
            <Button
              variant='outline'
              size='icon'
              className='shrink-0 md:hidden'
            >
              <Menu className='h-5 w-5' />
              <span className='sr-only'>Toggle navigation menu</span>
            </Button>
          </SheetTrigger>
        )}
        <SheetContent side='left' className='flex flex-col'>
          <div className='flex h-14 items-center lg:h-[60px]'>
            <Link to='/' className='flex items-center gap-2 font-semibold'>
              <img src={icon} className='h-6 w-6' alt='Wallet icon' />
              <span className=''>{wallet?.name}</span>
            </Link>
          </div>
          <nav className='grid gap-2 text-lg font-medium'>
            {navItems.map((item, i) => {
              switch (item.type) {
                case 'link': {
                  const Icon = item.icon;
                  return (
                    <Link
                      key={i}
                      to={item.url}
                      className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
                    >
                      <Icon className='h-5 w-5' />
                      {item.label}
                    </Link>
                  );
                }
                case 'separator': {
                  return <Separator className='my-2' />;
                }
              }
            })}
          </nav>
          <nav className='mt-auto grid gap-2 text-lg font-medium'>
            <Link
              to='/peers'
              className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
            >
              <span
                className={
                  'inline-flex h-3 w-3 m-1 rounded-full' +
                  ' ' +
                  (isSynced ? 'bg-emerald-600' : 'bg-yellow-600')
                }
              ></span>
              {isSynced ? (
                <>
                  {peers?.length} peers,
                  {peerMaxHeight ? ` ${peerMaxHeight} peak` : ' connecting...'}
                </>
              ) : (
                `Syncing ${walletState.sync.synced_coins} / ${walletState.sync.total_coins} coins`
              )}
            </Link>
            <Link
              to='/settings'
              className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
            >
              <Cog className='h-5 w-5' />
              Settings
            </Link>
            <button
              onClick={logout}
              className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
            >
              <LogOut className='h-5 w-5' />
              Logout
            </button>
          </nav>
        </SheetContent>
      </Sheet>
      <div className='flex-1 md:mt-2 flex items-center md:block'>
        {hasBackButton ? (
          <>
            <Button
              variant='link'
              size='sm'
              onClick={() => (props.back ? props.back() : navigate(-1))}
              className='hidden md:flex px-0 text-muted-foreground'
            >
              <ChevronLeft className='h-4 w-4 mr-1' />
              Back
            </Button>
          </>
        ) : (
          <div className='md:h-8'></div>
        )}
        <div className='flex-1 flex justify-between items-center gap-4 md:h-9 md:my-2'>
          <h1 className='text-xl font-bold tracking-tight md:text-3xl'>
            {props.title}
          </h1>
          <div className='hidden md:block'>{props.children}</div>
        </div>
      </div>
    </header>
  );
}
