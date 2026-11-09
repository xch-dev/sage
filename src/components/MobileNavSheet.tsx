import { useInsets } from '@/contexts/SafeAreaContext';
import { useWallet } from '@/contexts/WalletContext';
import { t } from '@lingui/core/macro';
import { platform } from '@tauri-apps/plugin-os';
import { Menu } from 'lucide-react';
import { useTheme } from 'theme-o-rama';
import { BottomNav, TopNav } from './Nav';
import { Button } from './ui/button';
import { Sheet, SheetContent, SheetTrigger } from './ui/sheet';
import { TooltipProvider } from './ui/tooltip';
import { WalletSwitcher } from './WalletSwitcher';

export function MobileNavSheet() {
  const insets = useInsets();
  const { wallet } = useWallet();
  const { currentTheme } = useTheme();

  const isMobile = platform() === 'ios' || platform() === 'android';

  return (
    <Sheet>
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

      <SheetContent
        side='left'
        isMobile={isMobile}
        className={`flex flex-col p-0 border-0 ${
          currentTheme?.backgroundImage ? 'bg-transparent' : ''
        }`}
        role='dialog'
        aria-label={t`Navigation menu`}
        style={{
          marginLeft: '-8px',
          paddingTop:
            insets.top !== 0
              ? `${insets.top + 8}px`
              : 'env(safe-area-inset-top)',
          paddingBottom: insets.bottom
            ? `${insets.bottom + 16}px`
            : 'env(safe-area-inset-bottom)',
          ...(currentTheme?.backgroundImage && {
            backgroundImage: `url(${currentTheme.backgroundImage})`,
            backgroundSize: 'cover',
            backgroundPosition: 'center',
            backgroundRepeat: 'no-repeat',
            backgroundAttachment: 'scroll',
            backgroundColor: 'transparent',
            transform: 'translateZ(0)',
            willChange: 'transform',
          }),
        }}
      >
        <div
          className={`flex flex-col h-full p-6 ${
            currentTheme?.sidebar ? '' : 'bg-muted/40'
          }`}
          style={
            currentTheme?.sidebar
              ? {
                  borderRight: '1px solid var(--sidebar-border)',
                  background: 'var(--sidebar-background)',
                  backdropFilter: 'var(--sidebar-backdrop-filter)',
                  WebkitBackdropFilter: 'var(--sidebar-backdrop-filter-webkit)',
                }
              : {}
          }
        >
          <div className='mt-4 mb-2'>
            <TooltipProvider>
              <WalletSwitcher wallet={wallet ?? undefined} />
            </TooltipProvider>
          </div>

          <TopNav />

          <div
            className={`mt-auto grid gap-1 text-md font-medium font-body ${
              !isMobile ? 'pb-4' : ''
            }`}
          >
            <BottomNav />
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
