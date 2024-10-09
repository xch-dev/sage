import { Cog, Images, LogOut, Wallet as WalletIcon } from 'lucide-react';
import { PropsWithChildren, useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';

import icon from '@/icon.png';
import { logoutAndUpdateState } from '@/state';

export default function Layout(props: PropsWithChildren<object>) {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

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
      <div className='grid h-screen w-screen md:grid-cols-[220px_1fr] lg:grid-cols-[280px_1fr]'>
        <div className='hidden border-r bg-muted/40 md:block overflow-y-auto'>
          <div className='flex h-full max-h-screen flex-col gap-2'>
            <div className='flex h-14 items-center px-4 lg:h-[60px] lg:px-6'>
              <Link href='/' className='flex items-center gap-2 font-semibold'>
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
                  Wallet
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
            {/* <div className='mt-auto p-4'>
              <Card x-chunk='dashboard-02-chunk-0'>
                <CardHeader className='p-2 pt-0 md:p-4'>
                  <CardTitle>Upgrade to Pro</CardTitle>
                  <CardDescription>
                    Unlock all features and get unlimited access to our support
                    team.
                  </CardDescription>
                </CardHeader>
                <CardContent className='p-2 pt-0 md:p-4 md:pt-0'>
                  <Button size='sm' className='w-full'>
                    Upgrade
                  </Button>
                </CardContent>
              </Card>
            </div> */}
          </div>
        </div>
        <div className='flex flex-col h-screen'>{props.children}</div>
      </div>
    </>
  );
}
