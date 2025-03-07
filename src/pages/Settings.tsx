import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { PasteInput } from '@/components/PasteInput';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { IntegerInput } from '@/components/ui/masked-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useLanguage } from '@/contexts/LanguageContext';
import { useWallet } from '@/contexts/WalletContext';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { clearState, fetchState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { getVersion } from '@tauri-apps/api/app';
import { platform } from '@tauri-apps/plugin-os';
import { TrashIcon, WalletIcon } from 'lucide-react';
import { useContext, useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { ping } from 'tauri-plugin-nfc-debug-api';
import { DarkModeContext } from '../App';
import {
  commands,
  KeyInfo,
  Network,
  NetworkConfig,
  WalletConfig,
} from '../bindings';
import { isValidU32 } from '../validation';

export default function Settings() {
  const { wallet } = useWallet();
  const [version, setVersion] = useState<string | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  // Get tab from URL or default to 'general'
  const currentTab = searchParams.get('tab') || 'general';

  // Update URL when tab changes
  const handleTabChange = (value: string) => {
    setSearchParams({ tab: value });
  };

  return (
    <Layout>
      <Header
        title={t`Settings`}
        back={!wallet ? () => navigate('/') : undefined}
        alwaysShowChildren
      >
        <div className='flex items-center justify-center gap-2 text-md text-muted-foreground'>
          <Trans>Version {version}</Trans>
        </div>
      </Header>
      <Container className='max-w-3xl'>
        <div className='flex flex-col gap-4'>
          <Tabs
            value={currentTab}
            onValueChange={handleTabChange}
            className='w-full'
          >
            <div className='flex flex-col gap-2'>
              <div className='flex items-center justify-between'>
                <TabsList className='w-full md:w-auto inline-flex h-9 items-center justify-start rounded-lg bg-muted p-1 text-muted-foreground'>
                  <TabsTrigger
                    value='general'
                    className='flex-1 md:flex-none rounded-md px-3 py-1 text-sm font-medium'
                  >
                    <Trans>General</Trans>
                  </TabsTrigger>

                  <TabsTrigger
                    value='wallet'
                    className='flex-1 md:flex-none rounded-md px-3 py-1 text-sm font-medium'
                  >
                    <Trans>Wallet</Trans>
                  </TabsTrigger>

                  <TabsTrigger
                    value='network'
                    className='flex-1 md:flex-none rounded-md px-3 py-1 text-sm font-medium'
                  >
                    <Trans>Network</Trans>
                  </TabsTrigger>

                  <TabsTrigger
                    value='advanced'
                    className='flex-1 md:flex-none rounded-md px-3 py-1 text-sm font-medium'
                  >
                    <Trans>Advanced</Trans>
                  </TabsTrigger>
                </TabsList>
              </div>
            </div>

            <div className='mt-4'>
              <TabsContent value='general'>
                <div className='grid gap-4'>
                  <WalletConnectSettings />
                  <GlobalSettings />
                </div>
              </TabsContent>

              <TabsContent value='wallet'>
                {wallet ? (
                  <WalletSettings wallet={wallet} />
                ) : (
                  <Card>
                    <CardContent className='flex flex-col items-center justify-center gap-4 py-12'>
                      <div className='rounded-full bg-muted p-3'>
                        <WalletIcon className='h-6 w-6 text-muted-foreground' />
                      </div>
                      <div className='text-center'>
                        <h3 className='font-medium'>
                          <Trans>No Wallet Connected</Trans>
                        </h3>
                        <p className='text-sm text-muted-foreground'>
                          <Trans>Connect a wallet to manage its settings</Trans>
                        </p>
                      </div>
                      <Button onClick={() => navigate('/')}>
                        <Trans>Connect Wallet</Trans>
                      </Button>
                    </CardContent>
                  </Card>
                )}
              </TabsContent>

              <TabsContent value='network'>
                <NetworkSettings />
              </TabsContent>

              <TabsContent value='advanced'>
                <div className='grid gap-6'>
                  {!isMobile && <RpcSettings />}
                  {isMobile && <TangemDebug />}
                </div>
              </TabsContent>
            </div>
          </Tabs>
        </div>
      </Container>
    </Layout>
  );
}

interface SettingsSectionProps {
  title: string;
  children: React.ReactNode;
}

function SettingsSection({ title, children }: SettingsSectionProps) {
  return (
    <div className='divide-y rounded-md border bg-neutral-100 dark:bg-neutral-900'>
      <div className='p-3'>
        <h3 className='text-sm font-medium'>{title}</h3>
      </div>
      <div className='divide-y'>{children}</div>
    </div>
  );
}

interface SettingItemProps {
  label: string;
  description?: string;
  control: React.ReactNode;
  children?: React.ReactNode;
}

function SettingItem({
  label,
  description,
  control,
  children,
}: SettingItemProps) {
  return (
    <div className='p-3'>
      <div className='flex flex-col sm:flex-row sm:items-center justify-between gap-x-4 gap-y-2'>
        <div className='space-y-1'>
          <Label className='text-sm font-medium'>{label}</Label>
          {description && (
            <div className='text-sm text-muted-foreground'>{description}</div>
          )}
        </div>
        <div className='flex sm:justify-end'>{control}</div>
      </div>
      {children}
    </div>
  );
}

function GlobalSettings() {
  const { dark, setDark } = useContext(DarkModeContext);
  const { locale, changeLanguage } = useLanguage();
  const { expiry, setExpiry } = useDefaultOfferExpiry();

  return (
    <SettingsSection title={t`Preferences`}>
      <SettingItem
        label={t`Dark Mode`}
        description={t`Switch between light and dark theme`}
        control={<Switch checked={dark} onCheckedChange={setDark} />}
      />

      <SettingItem
        label={t`Language`}
        description={t`Choose your preferred language`}
        control={
          <Select value={locale} onValueChange={changeLanguage}>
            <SelectTrigger className='w-[140px]'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='en-US'>English</SelectItem>
              <SelectItem value='de-DE'>Deutsch</SelectItem>
              <SelectItem value='zh-CN'>中文</SelectItem>
            </SelectContent>
          </Select>
        }
      />

      <SettingItem
        label={t`Default Offer Expiry`}
        description={t`Set a default expiration time for new offers`}
        control={
          <Switch
            checked={expiry.enabled}
            onCheckedChange={(checked) => {
              setExpiry({
                ...expiry,
                enabled: checked,
                days: checked ? '1' : '',
              });
            }}
          />
        }
      >
        {expiry.enabled && (
          <div className='grid grid-cols-3 gap-4 mt-2'>
            <TimeInput
              label={t`Days`}
              value={expiry.days}
              onChange={(value) => setExpiry({ ...expiry, days: value })}
            />
            <TimeInput
              label={t`Hours`}
              value={expiry.hours}
              onChange={(value) => setExpiry({ ...expiry, hours: value })}
            />
            <TimeInput
              label={t`Minutes`}
              value={expiry.minutes}
              onChange={(value) => setExpiry({ ...expiry, minutes: value })}
            />
          </div>
        )}
      </SettingItem>
    </SettingsSection>
  );
}

interface TimeInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

function TimeInput({ label, value, onChange }: TimeInputProps) {
  return (
    <div className='space-y-2'>
      <Label className='text-sm text-muted-foreground'>{label}</Label>
      <IntegerInput
        value={value}
        placeholder='0'
        min={0}
        onValueChange={(values) => onChange(values.value)}
      />
    </div>
  );
}

function WalletConnectSettings() {
  const { pair, sessions, disconnect, connecting } = useWalletConnect();
  const [uri, setUri] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  const { handleScanOrPaste } = useScannerOrClipboard((scanResValue) => {
    setUri(scanResValue);
  });

  const handlePair = async () => {
    try {
      setError(null);
      await pair(uri);
      setUri('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect');
    }
  };

  return (
    <SettingsSection title={t`WalletConnect`}>
      {sessions.length > 0 ? (
        sessions.map((session) => (
          <div
            key={session.topic}
            className='px-4 py-4 flex items-center justify-between gap-4'
          >
            <div className='flex gap-3 items-center'>
              <img
                src={session.peer?.metadata?.icons?.[0] ?? ''}
                alt={session.peer?.metadata?.name ?? t`Unknown App`}
                className='h-8 w-8 rounded-full'
              />
              <span className='font-medium'>
                {session.peer?.metadata?.name ?? t`Unknown App`}
              </span>
            </div>
            <Button
              variant='destructive'
              size='icon'
              onClick={() => disconnect(session.topic)}
            >
              <TrashIcon className='h-4 w-4' />
            </Button>
          </div>
        ))
      ) : (
        <div className='p-3 text-sm text-muted-foreground'>
          <Trans>No active sessions</Trans>
        </div>
      )}

      <div className='p-3'>
        <div className='flex flex-col gap-2'>
          <div className='flex gap-2 items-center'>
            <PasteInput
              value={uri}
              placeholder={t`WalletConnect URI`}
              onChange={(e) => setUri(e.target.value)}
              onEndIconClick={handleScanOrPaste}
              disabled={connecting}
            />

            <Button onClick={handlePair} disabled={connecting} size='sm'>
              {connecting ? (
                <Trans>Connecting...</Trans>
              ) : (
                <Trans>Connect</Trans>
              )}
            </Button>
          </div>

          {error && <span className='text-sm text-destructive'>{error}</span>}
        </div>
      </div>
    </SettingsSection>
  );
}

function NetworkSettings() {
  const { addError } = useErrors();

  const [discoverPeers, setDiscoverPeers] = useState<boolean | null>(null);
  const [targetPeersText, setTargetPeers] = useState<string | null>(null);
  const [networkId, setNetworkId] = useState<string | null>(null);
  const [networks, setNetworks] = useState<Record<string, Network>>({});

  const targetPeers =
    targetPeersText === null ? null : parseInt(targetPeersText);

  const invalidTargetPeers =
    targetPeers === null || !isValidU32(targetPeers, 1);

  const [config, setConfig] = useState<NetworkConfig | null>(null);

  useEffect(() => {
    commands.networkConfig().then(setConfig).catch(addError);
    commands
      .getNetworks({})
      .then((data) => setNetworks(data.networks))
      .catch(addError);
  }, [addError]);

  return (
    <SettingsSection title={t`Network`}>
      <SettingItem
        label={t`Discover Peers`}
        description={t`Automatically discover and connect to peers`}
        control={
          <Switch
            checked={discoverPeers ?? config?.discover_peers ?? true}
            onCheckedChange={(checked) => {
              commands
                .setDiscoverPeers({ discover_peers: checked })
                .catch(addError)
                .finally(() => setDiscoverPeers(checked));
            }}
          />
        }
      />

      <SettingItem
        label={t`Target Peers`}
        description={t`Number of peers to maintain connections with`}
        control={
          <Input
            type='number'
            className='w-[120px]'
            value={targetPeersText ?? config?.target_peers ?? 500}
            disabled={!(discoverPeers ?? config?.discover_peers)}
            onChange={(event) => setTargetPeers(event.target.value)}
            onBlur={() => {
              if (invalidTargetPeers) return;

              if (targetPeers !== config?.target_peers) {
                if (config) {
                  setConfig({ ...config, target_peers: targetPeers });
                }
                commands
                  .setTargetPeers({ target_peers: targetPeers })
                  .catch(addError);
              }
            }}
          />
        }
      />

      <SettingItem
        label={t`Network ID`}
        description={t`Choose the network to connect to`}
        control={
          <Select
            value={networkId ?? config?.network_id ?? 'mainnet'}
            onValueChange={(networkId) => {
              if (networkId !== config?.network_id) {
                if (config) {
                  setConfig({ ...config, network_id: networkId });
                }
                clearState();
                commands
                  .setNetworkId({ network_id: networkId })
                  .catch(addError)
                  .finally(() => {
                    fetchState();
                    setNetworkId(networkId);
                  });
              }
            }}
          >
            <SelectTrigger
              id='network'
              aria-label='Select network'
              className='w-[140px]'
            >
              <SelectValue placeholder={<Trans>Select network</Trans>} />
            </SelectTrigger>
            <SelectContent>
              {Object.keys(networks).map((networkId, i) => (
                <SelectItem key={i} value={networkId}>
                  {networkId}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        }
      />
    </SettingsSection>
  );
}

function RpcSettings() {
  const { addError } = useErrors();

  const [isRunning, setIsRunning] = useState(false);
  const [runOnStartup, setRunOnStartup] = useState(false);

  useEffect(() => {
    // Get initial state
    Promise.all([commands.isRpcRunning(), commands.getRpcRunOnStartup()])
      .then(([running, startup]) => {
        setIsRunning(running);
        setRunOnStartup(startup);
      })
      .catch(addError);

    // Poll RPC status
    const interval = setInterval(() => {
      commands.isRpcRunning().then(setIsRunning).catch(addError);
    }, 1000);

    return () => clearInterval(interval);
  }, [addError]);

  const start = () => {
    commands
      .startRpcServer()
      .catch(addError)
      .then(() => setIsRunning(true));
  };

  const stop = () => {
    commands
      .stopRpcServer()
      .catch(addError)
      .then(() => setIsRunning(false));
  };

  const toggleRunOnStartup = (checked: boolean) => {
    commands
      .setRpcRunOnStartup(checked)
      .catch(addError)
      .then(() => setRunOnStartup(checked));
  };

  return (
    <SettingsSection title={t`RPC Server`}>
      <div className='p-3'>
        <div className='flex items-center justify-between'>
          <div className='flex items-center gap-3'>
            <div
              className={`h-3 w-3 rounded-full ${
                isRunning ? 'bg-green-500' : 'bg-red-500'
              }`}
            />
            <span className='font-medium'>
              {isRunning ? <Trans>Running</Trans> : <Trans>Stopped</Trans>}
            </span>
          </div>
          <Button
            variant={isRunning ? 'destructive' : 'default'}
            size='sm'
            onClick={isRunning ? stop : start}
          >
            {isRunning ? <Trans>Stop</Trans> : <Trans>Start</Trans>}
          </Button>
        </div>
      </div>

      <SettingItem
        label={t`Run on startup`}
        description={t`Automatically start the RPC server when the app launches`}
        control={
          <Switch checked={runOnStartup} onCheckedChange={toggleRunOnStartup} />
        }
      />
    </SettingsSection>
  );
}

function TangemDebug() {
  const { addError } = useErrors();

  const [result, setResult] = useState<string | null>(null);

  return (
    <div>
      <Button
        onClick={() => ping('Hello, world!').then(setResult).catch(addError)}
      >
        Ping
      </Button>
      {result}
    </div>
  );
}

function WalletSettings(props: { wallet: KeyInfo }) {
  const { addError } = useErrors();

  const [name, setName] = useState(props.wallet.name);
  const [deriveAutomatically, setDeriveAutomatically] = useState<
    boolean | null
  >(true);
  const [derivationBatchSizeText, setDerivationBatchSize] = useState<
    string | null
  >(null);

  const derivationBatchSize =
    derivationBatchSizeText === null ? null : parseInt(derivationBatchSizeText);

  const invalidDerivationBatchSize =
    derivationBatchSize === null || !isValidU32(derivationBatchSize, 1);

  const [config, setConfig] = useState<WalletConfig | null>(null);

  useEffect(() => {
    commands
      .walletConfig(props.wallet.fingerprint)
      .then(setConfig)
      .catch(addError);
  }, [props.wallet.fingerprint, addError]);

  return (
    <SettingsSection title={t`Wallet`}>
      <SettingItem
        label={t`Wallet Name`}
        description={t`Give your wallet a memorable name`}
        control={
          <Input
            type='text'
            className='w-[200px]'
            value={name}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => {
              if (name !== config?.name) {
                if (config) {
                  setConfig({ ...config, name });
                }
                if (name)
                  commands
                    .renameKey({
                      fingerprint: props.wallet.fingerprint,
                      name,
                    })
                    .catch(addError);
              }
            }}
          />
        }
      />

      <SettingItem
        label={t`Generate Addresses`}
        description={t`Automatically generate new addresses as needed`}
        control={
          <Switch
            checked={
              deriveAutomatically ?? config?.derive_automatically ?? true
            }
            onCheckedChange={(checked) => {
              commands
                .setDeriveAutomatically({
                  fingerprint: props.wallet.fingerprint,
                  derive_automatically: checked,
                })
                .catch(addError)
                .finally(() => setDeriveAutomatically(checked));
            }}
          />
        }
      />

      <SettingItem
        label={t`Address Batch Size`}
        description={t`Number of addresses to generate at once`}
        control={
          <Input
            type='number'
            className='w-[120px]'
            value={
              derivationBatchSizeText ?? config?.derivation_batch_size ?? 500
            }
            disabled={!(deriveAutomatically ?? config?.derive_automatically)}
            onChange={(event) => setDerivationBatchSize(event.target.value)}
            onBlur={() => {
              if (invalidDerivationBatchSize) return;

              if (derivationBatchSize !== config?.derivation_batch_size) {
                if (config) {
                  setConfig({
                    ...config,
                    derivation_batch_size: derivationBatchSize,
                  });
                }
                commands
                  .setDerivationBatchSize({
                    fingerprint: props.wallet.fingerprint,
                    derivation_batch_size: derivationBatchSize,
                  })
                  .catch(addError);
              }
            }}
          />
        }
      />
    </SettingsSection>
  );
}
