import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { AnimatePresence, motion } from 'framer-motion';
import { ChevronLeft } from 'lucide-react';
import { PropsWithChildren, ReactNode } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Button } from './ui/button';
import { MobileNavSheet } from '@/components/MobileNavSheet.tsx';

const headerPaginationVariants = {
  enter: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20, transition: { duration: 0.15 } },
};

export default function Header(
  props: PropsWithChildren<{
    title: string | ReactNode;
    back?: () => void;
    mobileActionItems?: ReactNode;
    children?: ReactNode;
    paginationControls?: ReactNode;
    alwaysShowChildren?: boolean;
    style?: React.CSSProperties;
  }>,
) {
  const navigate = useNavigate();
  const location = useLocation();

  const hasBackButton = props.back || location.pathname.split('/').length > 2;
  const isMobile = platform() === 'ios' || platform() === 'android';

  return (
    <header
      className={`flex items-center gap-4 px-4 md:px-6 sticky top-0 z-10 ${
        !isMobile ? 'pt-2' : 'pb-2 pt-2'
      }`}
      role='banner'
      style={props.style}
    >
      <MobileNavSheet />
      <div className='flex-1 md:mt-1 flex items-center md:block'>
        <div className={`${!hasBackButton ? 'invisible' : ''}`}>
          <Button
            variant='link'
            size='sm'
            onClick={() => (props.back ? props.back() : navigate(-1))}
            className='hidden md:flex px-0 text-muted-foreground'
          >
            <ChevronLeft className='h-4 w-4 mr-1' aria-hidden='true' />
            <Trans>Back</Trans>
          </Button>
        </div>
        <div className='flex-1 flex justify-between items-center gap-4 md:h-8 md:my-1'>
          <div className='flex items-center gap-4'>
            <h1 className='text-xl font-bold tracking-tight md:text-3xl font-heading truncate'>
              {props.title}
            </h1>
            <AnimatePresence mode='wait'>
              {props.paginationControls && (
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  animate={headerPaginationVariants.enter}
                  exit={headerPaginationVariants.exit}
                  className='ml-4'
                >
                  {props.paginationControls}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
          <div className='flex items-center gap-2'>
            <div className={props.alwaysShowChildren ? '' : 'hidden md:block'}>
              {props.children}
            </div>
            {props.mobileActionItems && isMobile && props.mobileActionItems}
          </div>
        </div>
      </div>
    </header>
  );
}
