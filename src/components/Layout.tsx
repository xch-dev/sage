import { WalletRecord } from '@/bindings';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useInsets } from '@/contexts/SafeAreaContext';
import { useWallet } from '@/contexts/WalletContext';
import { t } from '@lingui/core/macro';
import { PanelLeft, PanelLeftClose } from 'lucide-react';
import { PropsWithChildren } from 'react';
import { useLocation } from 'react-router-dom';
import { Insets } from 'tauri-plugin-safe-area-insets';
import { useTheme } from 'theme-o-rama';
import { useLocalStorage } from 'usehooks-ts';
import { BottomNav, TopNav } from './Nav';
import { WalletSwitcher } from './WalletSwitcher';

function WalletTransitionWrapper({
  children,
  props,
  insets,
}: PropsWithChildren<{ props: LayoutProps; insets: Insets }>) {
  const { isSwitching, wallet } = useWallet();

  // Only show content if we have a wallet or we're not switching
  // This prevents old wallet data from showing during transition
  const shouldShow = wallet !== null || !isSwitching;

  return (
    <div
      className={`transition-all duration-300 ${
        !shouldShow
          ? 'opacity-0 blur-sm pointer-events-none'
          : 'opacity-100 blur-0'
      } flex flex-col h-screen overflow-hidden ${
        props.transparentBackground ? 'bg-transparent' : 'bg-background'
      }`}
      style={{
        paddingBottom: insets.bottom
          ? `${insets.bottom}px`
          : 'env(safe-area-inset-bottom)',
      }}
    >
      {shouldShow ? children : null}
    </div>
  );
}

const SIDEBAR_COLLAPSED_STORAGE_KEY = 'sage-wallet-sidebar-collapsed';

type LayoutProps = PropsWithChildren<object> & {
  transparentBackground?: boolean;
  wallet?: WalletRecord;
};

export function FullLayout(props: LayoutProps) {
  const { wallet } = props;
  const { currentTheme } = useTheme();
  const insets = useInsets();

  const [isCollapsed, setIsCollapsed] = useLocalStorage<boolean>(
    SIDEBAR_COLLAPSED_STORAGE_KEY,
    false,
  );

  return (
    <TooltipProvider>
      <div className='grid h-screen w-screen md:grid-cols-[auto_1fr]'>
        <div
          className={`hidden md:flex flex-col transition-all duration-300 ${
            isCollapsed ? 'w-[60px]' : 'w-[250px]'
          } ${currentTheme?.sidebar ? '' : 'border-r bg-muted/40'}`}
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
          role='complementary'
          aria-label={t`Sidebar navigation`}
        >
          <div className='bg-background flex h-full max-h-screen flex-col gap-2'>
            <div
              className={`flex h-14 items-center pt-2 px-5 ${isCollapsed ? 'flex-col gap-2 justify-center' : 'justify-between'}`}
            >
              <WalletSwitcher isCollapsed={isCollapsed} wallet={wallet} />
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    type='button'
                    onClick={() => setIsCollapsed(!isCollapsed)}
                    className='text-muted-foreground hover:text-primary transition-colors'
                    aria-label={
                      isCollapsed ? t`Expand sidebar` : t`Collapse sidebar`
                    }
                    aria-expanded={!isCollapsed}
                  >
                    {isCollapsed ? (
                      <PanelLeft className='h-5 w-5' aria-hidden='true' />
                    ) : (
                      <PanelLeftClose className='h-5 w-5' aria-hidden='true' />
                    )}
                  </button>
                </TooltipTrigger>
                <TooltipContent side='right' role='tooltip'>
                  {isCollapsed ? t`Expand sidebar` : t`Collapse sidebar`}
                </TooltipContent>
              </Tooltip>
            </div>

            <div className='flex-1 flex flex-col justify-between pb-4'>
              <div
                className={`grid items-start px-3 text-sm font-medium font-body ${
                  isCollapsed ? 'justify-center' : 'px-3'
                }`}
              >
                <TopNav isCollapsed={isCollapsed} />
              </div>

              <div
                className={`grid text-sm font-medium font-body ${
                  isCollapsed ? 'justify-center' : 'px-3'
                }`}
              >
                <BottomNav isCollapsed={isCollapsed} />
              </div>
            </div>
          </div>
        </div>
        <WalletTransitionWrapper props={props} insets={insets}>
          <div
            className='bg-background'
            style={{
              height:
                insets.top !== 0
                  ? `${insets.top + 8}px`
                  : 'env(safe-area-inset-top)',
            }}
          />
          {props.children}
        </WalletTransitionWrapper>
      </div>
    </TooltipProvider>
  );
}

function MinimalLayout(props: LayoutProps) {
  const insets = useInsets();

  return (
    <div className='flex flex-col h-screen w-screen'>
      <div
        className={`flex flex-col h-screen overflow-hidden ${
          props.transparentBackground ? 'bg-transparent' : 'bg-background'
        }`}
        style={{
          paddingBottom: insets.bottom
            ? `${insets.bottom}px`
            : 'env(safe-area-inset-bottom)',
        }}
      >
        <div
          className='bg-background'
          style={{
            height:
              insets.top !== 0
                ? `${insets.top + 8}px`
                : 'env(safe-area-inset-top)',
          }}
        />
        {props.children}
      </div>
    </div>
  );
}

export default function Layout(props: LayoutProps) {
  const { wallet } = useWallet();
  const location = useLocation();

  if (
    !wallet &&
    (location.pathname === '/settings' || location.pathname === '/themes')
  ) {
    return <MinimalLayout {...props} />;
  }

  return <FullLayout {...props} wallet={wallet || undefined} />;
}
