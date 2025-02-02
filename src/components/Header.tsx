import { useInsets } from '@/contexts/SafeAreaContext';
import useInitialization from '@/hooks/useInitialization';
import { useWallet } from '@/hooks/useWallet';
import icon from '@/icon.png';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { ChevronLeft, Menu } from 'lucide-react';
import { PropsWithChildren, ReactNode } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { BottomNav, TopNav } from './Nav';
import { Button } from './ui/button';
import { Sheet, SheetContent, SheetTrigger } from './ui/sheet';

export default function Header(
  props: PropsWithChildren<{
    title: string | ReactNode;
    back?: () => void;
    mobileActionItems?: ReactNode;
    children?: ReactNode;
  }>,
) {
  const navigate = useNavigate();
  const location = useLocation();
  const insets = useInsets();

  const initialized = useInitialization();
  const wallet = useWallet(initialized);

  const hasBackButton = props.back || location.pathname.split('/').length > 2;
  const isMobile = platform() === 'ios' || platform() === 'android';

  return (
    <header
      className={`flex items-center gap-4 px-4 md:px-6 sticky top-0 bg-background z-10 ${
        !isMobile ? 'pt-4' : 'pb-2 pt-2'
      }`}
      role='banner'
    >
      <Sheet>
        {hasBackButton ? (
          <Button
            variant='outline'
            size='icon'
            onClick={() => (props.back ? props.back() : navigate(-1))}
            className='md:hidden text-muted-foreground'
            aria-label={t`Back`}
          >
            <ChevronLeft className='h-5 w-5 pb' aria-hidden='true' />
          </Button>
        ) : (
          <SheetTrigger asChild>
            <Button
              variant='outline'
              size='icon'
              className='shrink-0 md:hidden'
              aria-label={t`Toggle navigation menu`}
              aria-expanded='false'
              aria-haspopup='dialog'
            >
              <Menu className='h-5 w-5' aria-hidden='true' />
            </Button>
          </SheetTrigger>
        )}
        <SheetContent
          side='left'
          isMobile={isMobile}
          className='flex flex-col'
          role='dialog'
          aria-label={t`Navigation menu`}
          style={{
            paddingTop:
              insets.top !== 0
                ? `${insets.top + 8}px`
                : 'env(safe-area-inset-top)',
            paddingBottom: insets.bottom
              ? `${insets.bottom + 16}px`
              : 'env(safe-area-inset-bottom)',
          }}
        >
          <div className='flex h-14 items-center '>
            <Link
              to='/wallet'
              className='flex items-center gap-2 font-semibold'
              aria-label={t`Go to wallet`}
            >
              <img src={icon} className='h-8 w-8' alt={t`Wallet icon`} />
              <span className='text-lg'>{wallet?.name}</span>
            </Link>
          </div>
          <div className='-mx-2'>
            <TopNav />
          </div>
          <div
            className={`mt-auto grid gap-1 text-md font-medium ${!isMobile ? 'pb-4' : ''}`}
          >
            <BottomNav />
          </div>
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
              <ChevronLeft className='h-4 w-4 mr-1' aria-hidden='true' />
              <Trans>Back</Trans>
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
          {props.mobileActionItems && isMobile && props.mobileActionItems}
        </div>
      </div>
    </header>
  );
}
