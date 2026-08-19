import { Blocks, HeartHandshake, PanelsTopLeft } from 'lucide-react';

import { MobileNavSheet } from '@/components/MobileNavSheet';
import { Button } from '@/components/ui/button';

interface Props {
  openAppCount: number;
  overviewOpen: boolean;
  activeAppHasDonation: boolean;
  onPrepareNavigation: () => void | Promise<void>;
  onFinishNavigation: () => void | Promise<void>;
  onOpenApps: () => void;
  onOpenOverview: () => void;
  onOpenDonation: () => void;
}

export function MobileAppTray({
  openAppCount,
  overviewOpen,
  activeAppHasDonation,
  onPrepareNavigation,
  onFinishNavigation,
  onOpenApps,
  onOpenOverview,
  onOpenDonation,
}: Props) {
  return (
    <div className='z-30 flex h-9 shrink-0 items-center justify-around gap-1 border-t bg-background/95 px-2 backdrop-blur'>
      <MobileNavSheet
        beforeOpen={onPrepareNavigation}
        afterClose={onFinishNavigation}
        compact
      />

      <Button
        type='button'
        variant='ghost'
        size='icon'
        className='h-8 w-10 shrink-0'
        onClick={onOpenApps}
        aria-label='Show apps'
      >
        <Blocks className='h-5 w-5' aria-hidden='true' />
      </Button>

      <Button
        type='button'
        variant={overviewOpen ? 'secondary' : 'ghost'}
        size='icon'
        className='relative h-8 w-10 shrink-0'
        onClick={onOpenOverview}
        aria-label={`Show ${openAppCount} open app${openAppCount === 1 ? '' : 's'}`}
      >
        <span className='relative'>
          <PanelsTopLeft className='h-5 w-5' aria-hidden='true' />
          <span className='absolute -right-3 -top-2 flex min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] leading-4 text-primary-foreground'>
            {openAppCount}
          </span>
        </span>
      </Button>

      {activeAppHasDonation ? (
        <Button
          type='button'
          variant='ghost'
          size='icon'
          className='h-8 w-10 shrink-0 text-amber-500 hover:bg-amber-500/10 hover:text-amber-600'
          onClick={onOpenDonation}
          aria-label='Support developer'
        >
          <HeartHandshake className='h-5 w-5' aria-hidden='true' />
        </Button>
      ) : null}
    </div>
  );
}
