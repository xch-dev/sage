import { PropsWithChildren } from 'react';
import { Button } from './ui/button';
import { ChevronLeft, Menu } from 'lucide-react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { Sheet, SheetContent, SheetTrigger } from './ui/sheet';
import { Cog, Images, LogOut, Wallet as WalletIcon } from 'lucide-react';
import icon from '@/icon.png';
import { logoutAndUpdateState } from '@/state';
import { useWallet } from '@/hooks/useWallet';
import { usePeers } from '@/contexts/PeerContext';

export default function Header(
  props: PropsWithChildren<{ title: string; back?: () => void }>,
) {
  const navigate = useNavigate();
  const location = useLocation();

  const { wallet } = useWallet();
  const { peers } = usePeers();

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
    <header className='flex h-14 items-center gap-4 border-b bg-muted/40 px-4 lg:h-[60px] lg:px-6'>
      <Sheet>
        {!hasBackButton && (
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
          <div className='flex h-14 items-center px-4 lg:h-[60px] lg:px-6'>
            <Link to='/' className='flex items-center gap-2 font-semibold'>
              <img src={icon} className='h-6 w-6' alt='Wallet icon' />
              <span className=''>{wallet?.name}</span>
            </Link>
          </div>
          <nav className='grid gap-2 text-lg font-medium'>
            <Link
              to='/wallet'
              className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
            >
              <WalletIcon className='h-5 w-5' />
              Wallet
            </Link>
            <Link
              to='/nfts'
              className='mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2 text-muted-foreground hover:text-foreground'
            >
              <Images className='h-5 w-5' />
              NFTs
            </Link>
          </nav>
          <nav className='mt-auto grid gap-2 text-lg font-medium'>
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
      <div className='w-full flex-1'>
        {hasBackButton && (
          <Button
            variant='outline'
            size='icon'
            onClick={() => (props.back ? props.back() : navigate(-1))}
            className='mr-4'
          >
            <ChevronLeft className='h-4 w-4' />
          </Button>
        )}
        <span className='text-xl font-semibold'>{props.title}</span>
      </div>
      <Link to='/peers'>
        <Button
          variant='ghost'
          className='flex items-center gap-1.5 text-sm text-muted-foreground font-bold'
        >
          <span
            className={
              'inline-flex h-2 w-2 rounded-full' +
              ' ' +
              (peerMaxHeight > 0 ? 'bg-emerald-600' : 'bg-yellow-600')
            }
          ></span>
          {peers?.length} peers,
          {peerMaxHeight ? ` ${peerMaxHeight} peak` : ' connecting...'}
        </Button>
      </Link>
    </header>
  );
}
