import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { useInsets } from '@/contexts/SafeAreaContext';
import { useWallet } from '@/contexts/WalletContext';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  ArrowLeftRightIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  DatabaseIcon,
  GlobeIcon,
  InfoIcon,
  LucideIcon,
  PaletteIcon,
  PlugIcon,
  ShieldCheckIcon,
  SlidersHorizontalIcon,
  WalletIcon,
} from 'lucide-react';
import { ReactNode } from 'react';
import {
  Link,
  Navigate,
  useLocation,
  useNavigate,
  useParams,
} from 'react-router-dom';
import { AboutSettings } from './settings/AboutSettings';
import { AppearanceSettings } from './settings/AppearanceSettings';
import { ConnectionSettings } from './settings/ConnectionSettings';
import { GeneralSettings } from './settings/GeneralSettings';
import { NetworkSettings } from './settings/NetworkSettings';
import { SecuritySettings } from './settings/SecuritySettings';
import { TransactionSettings } from './settings/TransactionSettings';
import { WalletSettings, WalletSyncSettings } from './settings/WalletSettings';

type SettingsGroup = 'app' | 'wallet' | 'about';

interface SettingsLinkState {
  fromIndex?: boolean;
}

interface SettingsPage {
  id: string;
  group: SettingsGroup;
  label: string;
  description: string;
  icon: LucideIcon;
  render: (fingerprint: number | null) => ReactNode;
}

function settingsPages(isMobile: boolean): SettingsPage[] {
  return [
    {
      id: 'general',
      group: 'app',
      label: t`General`,
      description: t`Language`,
      icon: SlidersHorizontalIcon,
      render: () => <GeneralSettings />,
    },
    {
      id: 'appearance',
      group: 'app',
      label: t`Appearance`,
      description: t`Theme`,
      icon: PaletteIcon,
      render: () => <AppearanceSettings />,
    },
    {
      id: 'security',
      group: 'app',
      label: t`Security`,
      description: t`Biometrics and keychain`,
      icon: ShieldCheckIcon,
      render: () => <SecuritySettings />,
    },
    {
      id: 'transactions',
      group: 'app',
      label: t`Transactions`,
      description: t`Fees, clawback, expiration and cold wallets`,
      icon: ArrowLeftRightIcon,
      render: () => <TransactionSettings />,
    },
    {
      id: 'network',
      group: 'app',
      label: t`Network & Sync`,
      description: t`Network, peers and syncing`,
      icon: GlobeIcon,
      render: () => <NetworkSettings />,
    },
    {
      id: 'connections',
      group: 'app',
      label: t`Connected Apps`,
      description: isMobile
        ? t`WalletConnect`
        : t`WalletConnect and RPC server`,
      icon: PlugIcon,
      render: () => <ConnectionSettings />,
    },
    {
      id: 'wallet',
      group: 'wallet',
      label: t`Wallet`,
      description: t`Name, network override and change address`,
      icon: WalletIcon,
      render: (fingerprint) =>
        fingerprint !== null && <WalletSettings fingerprint={fingerprint} />,
    },
    {
      id: 'storage',
      group: 'wallet',
      label: t`Sync & Storage`,
      description: t`Derivation index, database and resync`,
      icon: DatabaseIcon,
      render: (fingerprint) =>
        fingerprint !== null && (
          <WalletSyncSettings fingerprint={fingerprint} />
        ),
    },
    {
      id: 'about',
      group: 'about',
      label: t`About`,
      description: t`Version and logs`,
      icon: InfoIcon,
      render: () => <AboutSettings />,
    },
  ];
}

export default function Settings() {
  const { wallet } = useWallet();
  const navigate = useNavigate();
  const location = useLocation();
  const insets = useInsets();
  const { section } = useParams<{ section?: string }>();

  // Wallet pages only exist while a wallet is logged in.
  const isMobile = platform() === 'ios' || platform() === 'android';
  const pages = settingsPages(isMobile).filter(
    (page) => page.group !== 'wallet' || wallet,
  );

  const activePage = section ? pages.find((page) => page.id === section) : null;

  if (activePage === undefined) {
    return <Navigate to='/settings' replace />;
  }

  const groups: { group: SettingsGroup; label?: string }[] = [
    { group: 'app', label: t`App` },
    { group: 'wallet', label: wallet?.name || t`Wallet` },
    { group: 'about' },
  ];

  // Going back from a page opened from the list returns to that list entry, rather than adding
  // another one, so the system back button doesn't reopen the page.
  const backToIndex = () => {
    if ((location.state as SettingsLinkState | null)?.fromIndex) {
      navigate(-1);
    } else {
      navigate('/settings', { replace: true });
    }
  };

  const header = activePage ? (
    <Header title={activePage.label} back={backToIndex} />
  ) : (
    <Header
      title={t`Settings`}
      back={!wallet ? () => navigate('/') : undefined}
    />
  );

  return (
    <Layout>
      {header}
      <Container
        className='max-w-3xl'
        style={{
          paddingBottom: insets.bottom ? `${insets.bottom}px` : undefined,
        }}
      >
        {activePage ? (
          <div className='flex flex-col gap-4'>
            {/* The header's back button is hidden on mobile. */}
            <button
              type='button'
              onClick={backToIndex}
              className='md:hidden self-start flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground'
            >
              <ChevronLeftIcon className='h-4 w-4' aria-hidden='true' />
              <Trans>Settings</Trans>
            </button>
            {activePage.render(wallet?.fingerprint ?? null)}
          </div>
        ) : (
          <nav
            aria-label={t`Settings sections`}
            className='flex flex-col gap-6'
          >
            {groups.map(({ group, label }) => {
              const groupPages = pages.filter((page) => page.group === group);
              if (groupPages.length === 0) return null;

              return (
                <div key={group} className='flex flex-col gap-2'>
                  {label && (
                    <h2 className='px-3 text-xs font-medium uppercase tracking-wide text-muted-foreground truncate'>
                      {label}
                    </h2>
                  )}
                  <div className='flex flex-col rounded-md border bg-card text-card-foreground divide-y overflow-hidden'>
                    {groupPages.map((page) => {
                      const Icon = page.icon;

                      return (
                        <Link
                          key={page.id}
                          to={`/settings/${page.id}`}
                          state={
                            { fromIndex: true } satisfies SettingsLinkState
                          }
                          className='flex items-center gap-3 p-3 transition-colors hover:bg-muted/50'
                        >
                          <div className='rounded-md bg-muted p-2'>
                            <Icon className='h-4 w-4' aria-hidden='true' />
                          </div>
                          <div className='flex-1 min-w-0'>
                            <div className='text-sm font-medium truncate'>
                              {page.label}
                            </div>
                            <div className='text-sm text-muted-foreground truncate'>
                              {page.description}
                            </div>
                          </div>
                          <ChevronRightIcon
                            className='h-4 w-4 text-muted-foreground shrink-0'
                            aria-hidden='true'
                          />
                        </Link>
                      );
                    })}
                  </div>
                </div>
              );
            })}
          </nav>
        )}
      </Container>
    </Layout>
  );
}
