import Container from '@/components/Container';
import { ResyncDialog } from '@/components/dialogs/ResyncDialog';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { PasteInput } from '@/components/PasteInput';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, IntegerInput } from '@/components/ui/masked-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { CustomError } from '@/contexts/ErrorContext';
import { useLanguage } from '@/contexts/LanguageContext';
import { useInsets } from '@/contexts/SafeAreaContext';
import { useWallet } from '@/contexts/WalletContext';
import { useBiometric } from '@/hooks/useBiometric';
import { useDefaultClawback } from '@/hooks/useDefaultClawback';
import { useDefaultFee } from '@/hooks/useDefaultFee';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { exportText, ExportType } from '@/lib/exportText';
import {
  clearState,
  fetchState,
  updateSyncStatus,
  useWalletState,
} from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useVirtualizer } from '@tanstack/react-virtual';
import { getVersion } from '@tauri-apps/api/app';
import { platform } from '@tauri-apps/plugin-os';
import {
  DownloadIcon,
  LoaderCircleIcon,
  TrashIcon,
  WalletIcon,
} from 'lucide-react';
import prettyBytes from 'pretty-bytes';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { z } from 'zod';
import {
  commands,
  events,
  GetDatabaseStatsResponse,
  KeyInfo,
  LogFile,
  Network,
  NetworkConfig,
  PerformDatabaseMaintenanceResponse,
  Wallet,
  WalletDefaults,
  WebhookEntry,
} from '../bindings';

import { ThemeSelectorSimple } from '../components/ThemeSelector';
import { isValidU32 } from '../validation';
export default function Settings() {
  const { wallet } = useWallet();
  const [version, setVersion] = useState<string | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const insets = useInsets();

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
      <Container
        className='max-w-3xl'
        style={{
          paddingBottom: insets.bottom ? `${insets.bottom}px` : undefined,
        }}
      >
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
                  <WalletSettings fingerprint={wallet.fingerprint} />
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
                <div className='grid gap-4'>
                  {!isMobile && <RpcSettings />}
                  <WebhooksSettings />
                  <LogViewer />
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
    <div className='divide-y rounded-md border bg-card text-card-foreground overflow-hidden'>
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
  const { addError } = useErrors();
  const { locale, changeLanguage } = useLanguage();
  const { expiry, setExpiry } = useDefaultOfferExpiry();
  const { clawback, setClawback } = useDefaultClawback();
  const { enabled, available, enableIfAvailable, disable } = useBiometric();
  const { setFee } = useDefaultFee();

  const [defaultWalletConfig, setDefaultWalletConfig] =
    useState<WalletDefaults | null>(null);

  useEffect(() => {
    commands.defaultWalletConfig().then(setDefaultWalletConfig).catch(addError);
  }, [addError]);

  const isMobile = platform() === 'ios' || platform() === 'android';

  const toggleBiometric = async (value: boolean) => {
    if (value) {
      await enableIfAvailable();
    } else {
      await disable();
    }
  };

  return (
    <>
      <SettingsSection title={t`Preferences`}>
        <SettingItem
          label={t`Theme`}
          description={t`Choose your preferred theme`}
          control={
            <div className='w-full'>
              {/* Theme selector will be added below */}
            </div>
          }
        />
        <div>
          <ThemeSelectorSimple />
          <div className='m-4'>
            <Link to='/themes'>
              <Button variant='outline' size='sm'>
                <Trans>More Themes...</Trans>
              </Button>
            </Link>
          </div>
        </div>
        {isMobile && (
          <SettingItem
            label={t`Biometric Authentication`}
            description={t`Require biometrics for sensitive actions`}
            control={
              <Switch
                checked={enabled}
                disabled={!available}
                onCheckedChange={toggleBiometric}
              />
            }
          />
        )}
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
                <SelectItem value='es-MX'>Español</SelectItem>
              </SelectContent>
            </Select>
          }
        />
      </SettingsSection>

      <SettingsSection title={t`Transaction Defaults`}>
        <SettingItem
          label={t`Default Fee`}
          description={t`The default fee to use for transactions`}
          control={
            <FeeAmountInput
              onValueChange={(values) =>
                setFee(values.value === '' ? '0' : values.value)
              }
            />
          }
        />
        <SettingItem
          label={t`Default Clawback`}
          description={t`Set a default clawback time for transactions`}
          control={
            <Switch
              checked={clawback.enabled}
              onCheckedChange={(checked) => {
                setClawback({
                  ...clawback,
                  enabled: checked,
                });
              }}
            />
          }
        >
          {clawback.enabled && (
            <div className='grid grid-cols-3 gap-4 mt-2'>
              <TimeInput
                label={t`Days`}
                value={clawback.days}
                onChange={(value) => setClawback({ ...clawback, days: value })}
              />
              <TimeInput
                label={t`Hours`}
                value={clawback.hours}
                onChange={(value) => setClawback({ ...clawback, hours: value })}
              />
              <TimeInput
                label={t`Minutes`}
                value={clawback.minutes}
                onChange={(value) =>
                  setClawback({ ...clawback, minutes: value })
                }
              />
            </div>
          )}
        </SettingItem>
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

      <SettingsSection title={t`Syncing Defaults`}>
        <SettingItem
          label={t`Delta Sync`}
          description={t`A more lightweight sync that only checks unseen blocks`}
          control={
            <Switch
              checked={defaultWalletConfig?.delta_sync ?? true}
              onCheckedChange={(checked) => {
                if (!defaultWalletConfig) return;
                setDefaultWalletConfig({
                  ...defaultWalletConfig,
                  delta_sync: checked,
                });
                commands.setDeltaSync({ delta_sync: checked }).catch(addError);
              }}
            />
          }
        />
      </SettingsSection>
    </>
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
  const [targetPeersText, setTargetPeersText] = useState<string | null>(null);
  const [network, setNetwork] = useState<string | null>(null);
  const [networks, setNetworks] = useState<Network[]>([]);

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
        label={t`Default Network`}
        description={t`Choose the network to connect to`}
        control={
          <Select
            value={network ?? config?.default_network ?? 'mainnet'}
            onValueChange={(name) => {
              if (name !== config?.default_network) {
                if (config) {
                  setConfig({ ...config, default_network: name });
                }
                clearState();
                commands
                  .setNetwork({ name })
                  .catch(addError)
                  .finally(() => {
                    fetchState();
                    setNetwork(name);
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
              {networks.map((network) => (
                <SelectItem key={network.name} value={network.name}>
                  {network.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        }
      />

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
            onChange={(event) => setTargetPeersText(event.target.value)}
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
    </SettingsSection>
  );
}

function LogViewer() {
  const { addError } = useErrors();

  const [logs, setLogs] = useState<LogFile[]>([]);
  const [logName, setLogName] = useState('');
  const [selectedLog, setSelectedLog] = useState<LogFile | null>(null);
  const parentRef = useRef<HTMLDivElement>(null);
  const [logLines, setLogLines] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedLevel, setSelectedLevel] = useState<string>('all');
  const [filteredLines, setFilteredLines] = useState<string[]>([]);
  const [isAtBottom, setIsAtBottom] = useState(true);

  const rowVirtualizer = useVirtualizer({
    count: filteredLines.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24,
    overscan: 5,
  });

  const scrollToBottom = useCallback(() => {
    if (!parentRef.current || !rowVirtualizer) return;
    const scrollHeight = rowVirtualizer.getTotalSize();
    parentRef.current.scrollTop = scrollHeight;
  }, [rowVirtualizer]);

  const checkIfAtBottom = useCallback(() => {
    if (!parentRef.current || !rowVirtualizer) return false;
    const { scrollTop, clientHeight } = parentRef.current;
    const scrollHeight = rowVirtualizer.getTotalSize();
    // Consider "at bottom" if within 10px of the bottom
    return scrollHeight - scrollTop - clientHeight < 10;
  }, [rowVirtualizer]);

  const handleScroll = useCallback(() => {
    setIsAtBottom(checkIfAtBottom());
  }, [checkIfAtBottom]);

  const virtualItems = rowVirtualizer.getVirtualItems();

  // Handle scrolling when log changes or new lines are added
  useEffect(() => {
    const items = virtualItems;
    // Always scroll on initial load of a log, or when at bottom and content changes
    if (
      items.length > 0 &&
      selectedLog &&
      (isAtBottom || parentRef.current?.scrollTop === undefined)
    ) {
      scrollToBottom();
    }
  }, [virtualItems, selectedLog, isAtBottom, scrollToBottom]);

  const updateLogs = useCallback(() => {
    commands
      .getLogs()
      .then((logs) =>
        setLogs(logs.sort((a, b) => b.name.localeCompare(a.name))),
      )
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    updateLogs();

    const interval = setInterval(() => {
      updateLogs();
    }, 2000);

    return () => clearInterval(interval);
  }, [updateLogs]);

  useEffect(() => {
    if (logs.length > 0) {
      const defaultLog = logs[0];
      setLogName(defaultLog.name);
      setSelectedLog(defaultLog);
    }
  }, [logs]);

  useEffect(() => {
    if (selectedLog) {
      setLogLines(selectedLog.text.trim().split('\n'));
    } else {
      setLogLines([]);
    }
  }, [selectedLog]);

  // Filter logs based on search query and selected level
  useEffect(() => {
    const filtered = logLines.filter((line) => {
      const matchesSearch =
        searchQuery === '' ||
        line.toLowerCase().includes(searchQuery.toLowerCase());

      if (!matchesSearch) return false;

      if (selectedLevel === 'all') return true;

      const levelMatch = line.match(
        /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z\s+(\w+)/,
      );
      if (!levelMatch) return false;

      return levelMatch[1].toLowerCase() === selectedLevel.toLowerCase();
    });

    setFilteredLines(filtered);
  }, [logLines, searchQuery, selectedLevel]);

  const handleLogChange = (name: string) => {
    setLogName(name);
    const log = logs.find((l) => l.name === name);
    setSelectedLog(log ?? null);
    setSearchQuery('');
    setSelectedLevel('all');
    setIsAtBottom(true); // Reset isAtBottom when changing logs
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString();
    } catch {
      return timestamp;
    }
  };

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return 'text-red-500';
      case 'WARN':
        return 'text-yellow-600';
      case 'INFO':
        return 'text-blue-600';
      case 'DEBUG':
        return 'text-slate-500';
      default:
        return 'text-muted-foreground';
    }
  };

  const handleExport = () => {
    if (selectedLog) {
      exportText(selectedLog.text, selectedLog.name, ExportType.LOG);
    }
  };

  return (
    <SettingsSection title={t`Log Viewer`}>
      <div className='p-3 space-y-4 max-w-full'>
        <div className='flex flex-col gap-4'>
          <div className='flex flex-col gap-2'>
            <Select value={logName} onValueChange={handleLogChange}>
              <SelectTrigger id='log' aria-label='Select file'>
                <SelectValue placeholder={<Trans>Select log file</Trans>} />
              </SelectTrigger>
              <SelectContent>
                {logs.map((log) => (
                  <SelectItem key={log.name} value={log.name}>
                    {log.name.replace('app.log.', '')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className='flex gap-2'>
              <Input
                type='text'
                placeholder={t`Search logs...`}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />

              <Select value={selectedLevel} onValueChange={setSelectedLevel}>
                <SelectTrigger
                  id='level'
                  aria-label='Select log level'
                  className='w-[120px]'
                >
                  <SelectValue placeholder={<Trans>Log Level</Trans>} />
                </SelectTrigger>

                <SelectContent>
                  <SelectItem value='all'>
                    <Trans>All Levels</Trans>
                  </SelectItem>
                  <SelectItem value='error'>
                    <span className={getLevelColor('ERROR')}>ERROR</span>
                  </SelectItem>
                  <SelectItem value='warn'>
                    <span className={getLevelColor('WARN')}>WARN</span>
                  </SelectItem>
                  <SelectItem value='info'>
                    <span className={getLevelColor('INFO')}>INFO</span>
                  </SelectItem>
                  <SelectItem value='debug'>
                    <span className={getLevelColor('DEBUG')}>DEBUG</span>
                  </SelectItem>
                </SelectContent>
              </Select>

              <Button variant='outline' onClick={handleExport}>
                <DownloadIcon className='h-4 w-4' />
              </Button>
            </div>
          </div>

          {selectedLog && filteredLines.length === 0 && (
            <div className='text-center py-8 text-muted-foreground'>
              <Trans>No matching log entries found</Trans>
            </div>
          )}
        </div>

        {selectedLog && filteredLines.length > 0 && (
          <div
            ref={parentRef}
            style={{ height: 'calc(100vh - 400px)', minHeight: '300px' }}
            className='border rounded-lg bg-muted/30 overflow-auto'
            onScroll={handleScroll}
          >
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const line = filteredLines[virtualRow.index];
                const match = line.match(
                  /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(\w+)\s+(.*)$/,
                );

                return (
                  <div
                    key={virtualRow.index}
                    data-index={virtualRow.index}
                    ref={rowVirtualizer.measureElement}
                    className={`absolute top-0 left-0 w-full hover:bg-muted/50 transition-colors`}
                    style={{
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    {match ? (
                      <div className='flex min-w-max px-2 py-0.5'>
                        <div className='w-[90px] whitespace-nowrap'>
                          <span className='text-xs text-muted-foreground font-mono'>
                            {formatTimestamp(match[1])}
                          </span>
                        </div>
                        <div className='w-[50px] whitespace-nowrap'>
                          <span
                            className={`text-xs font-medium ${getLevelColor(
                              match[2],
                            )}`}
                          >
                            {match[2].padEnd(5, ' ')}
                          </span>
                        </div>
                        <div className='flex-1 whitespace-nowrap'>
                          <span className='text-xs font-mono'>{match[3]}</span>
                        </div>
                      </div>
                    ) : (
                      <div className='px-2 py-0.5 whitespace-nowrap'>
                        <span className='text-xs font-mono'>{line}</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </SettingsSection>
  );
}

function RpcSettings() {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

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

  const start = async () => {
    if (!(await promptIfEnabled())) return;

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

  const toggleRunOnStartup = async (checked: boolean) => {
    if (!(await promptIfEnabled())) return;

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

function WalletSettings({ fingerprint }: { fingerprint: number }) {
  const { addError } = useErrors();

  const walletState = useWalletState();

  const [key, setKey] = useState<KeyInfo | null>(null);
  const [wallet, setWallet] = useState<Wallet | null>(null);
  const [localName, setLocalName] = useState<string>('');
  const [localChangeAddress, setLocalChangeAddress] = useState('');
  const [networks, setNetworks] = useState<Network[]>([]);
  const [deriveOpen, setDeriveOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [resyncOpen, setResyncOpen] = useState(false);
  const [dbStats, setDbStats] = useState<GetDatabaseStatsResponse | null>(null);
  const [loadingStats, setLoadingStats] = useState(false);
  const [maintenanceOpen, setMaintenanceOpen] = useState(false);
  const [maintenanceResults, setMaintenanceResults] =
    useState<PerformDatabaseMaintenanceResponse | null>(null);
  const [performingMaintenance, setPerformingMaintenance] = useState(false);
  const [unhardened, setUnhardened] = useState(true);
  const [hardened, setHardened] = useState(true);

  const saveChangeAddress = (address: string) => {
    const trimmedAddress = address.trim();
    const currentAddress = wallet?.change_address || '';
    if (trimmedAddress !== currentAddress) {
      setChangeAddress(trimmedAddress || null);
    }
  };

  const { handleScanOrPaste: handleScanOrPasteChangeAddress } =
    useScannerOrClipboard((scanResValue) => {
      setLocalChangeAddress(scanResValue);
      saveChangeAddress(scanResValue);
    });

  const fetchDatabaseStats = useCallback(async () => {
    setLoadingStats(true);
    try {
      const stats = await commands.getDatabaseStats({});
      setDbStats(stats);
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setLoadingStats(false);
    }
  }, [addError]);

  const performMaintenance = useCallback(async () => {
    setPerformingMaintenance(true);
    try {
      const results = await commands.performDatabaseMaintenance({
        force_vacuum: false,
      });
      setMaintenanceResults(results);
      setMaintenanceOpen(true);
      // Refresh database stats after maintenance
      await fetchDatabaseStats();
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setPerformingMaintenance(false);
    }
  }, [addError, fetchDatabaseStats]);

  useEffect(() => {
    commands
      .getKey({ fingerprint })
      .then((data) => setKey(data.key))
      .catch(addError);
  }, [addError, fingerprint]);

  useEffect(() => {
    commands
      .getNetworks({})
      .then((data) => setNetworks(data.networks))
      .catch(addError);

    commands
      .walletConfig(fingerprint)
      .then((data) => {
        setWallet(data);
        if (data?.name) setLocalName(data.name);
        if (data?.change_address) {
          setLocalChangeAddress(data.change_address);
        }
      })
      .catch(addError);

    // Fetch database stats when component mounts
    fetchDatabaseStats();
  }, [addError, fingerprint, fetchDatabaseStats]);

  const addNetworkOverride = async () => {
    if (!wallet) return;
    try {
      const config = await commands.networkConfig();
      await commands.setNetworkOverride({
        fingerprint,
        name: config.default_network,
      });
      setWallet({ ...wallet, network: config.default_network });
    } catch (error) {
      addError(error as CustomError);
    }
  };

  const setNetworkOverride = async (name: string | null) => {
    if (!wallet) return;
    clearState();
    try {
      await commands.setNetworkOverride({ fingerprint, name });
      setWallet({ ...wallet, network: name });
    } catch (error) {
      addError(error as CustomError);
    }
    fetchState();
  };

  const setChangeAddress = async (address: string | null) => {
    if (!wallet) return;
    clearState();
    try {
      if (address) {
        const { valid } = await commands.checkAddress({ address });
        if (!valid) {
          addError({
            kind: 'not_found',
            reason: t`Cannot send change to external address. Setting not updated.`,
          });
          return;
        }
      }
      await commands.setChangeAddress({
        fingerprint,
        change_address: address || null,
      });
      setWallet({ ...wallet, change_address: address });
    } catch (error) {
      addError(error as CustomError);
    } finally {
      fetchState();
    }
  };

  const derivationIndex = walletState.sync.unhardened_derivation_index;

  const schema = z.object({
    index: z.string().refine((value) => {
      const num = parseInt(value);

      if (
        isNaN(num) ||
        !isFinite(num) ||
        num < derivationIndex ||
        Math.floor(num) != num
      )
        return false;

      return true;
    }),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      index: derivationIndex.toString(),
    },
  });

  const handler = (values: z.infer<typeof schema>) => {
    setPending(true);

    commands
      .increaseDerivationIndex({
        index: parseInt(values.index),
        hardened: key?.has_secrets && hardened,
        unhardened,
      })
      .then(() => {
        setDeriveOpen(false);
        updateSyncStatus();
      })
      .catch(addError)
      .finally(() => setPending(false));
  };

  return (
    <div className='flex flex-col gap-4'>
      <SettingsSection title={t`Wallet`}>
        <SettingItem
          label={t`Wallet Name`}
          description={t`Give your wallet a memorable name`}
          control={
            <Input
              type='text'
              className='w-[200px]'
              value={localName}
              onChange={(event) => setLocalName(event.target.value)}
              onBlur={() => {
                if (localName === wallet?.name) return;

                commands
                  .renameKey({
                    fingerprint,
                    name: localName,
                  })
                  .then(() => {
                    if (wallet) {
                      setWallet({ ...wallet, name: localName });
                    }
                  })
                  .catch(addError);
              }}
            />
          }
        />

        <SettingItem
          label={t`Override Network`}
          description={t`Override the default network for this wallet`}
          control={
            <Switch
              checked={!!wallet?.network}
              onCheckedChange={(checked) => {
                if (checked) {
                  addNetworkOverride();
                } else {
                  setNetworkOverride(null);
                }
              }}
            />
          }
        >
          {!!wallet?.network && (
            <div className='mt-3'>
              <Select
                value={wallet?.network}
                onValueChange={(name) => {
                  setNetworkOverride(name);
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
                  {networks.map((network) => (
                    <SelectItem key={network.name} value={network.name}>
                      {network.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
        </SettingItem>

        <SettingItem
          label={t`Change Address`}
          description={t`Set a specific address to send change to`}
          control={<div />}
        >
          <div className='mt-3'>
            <PasteInput
              placeholder={t`Enter change address`}
              value={localChangeAddress}
              onChange={(event) => setLocalChangeAddress(event.target.value)}
              onBlur={() => {
                saveChangeAddress(localChangeAddress);
              }}
              onEndIconClick={handleScanOrPasteChangeAddress}
            />
          </div>
        </SettingItem>
      </SettingsSection>

      <SettingsSection title={t`Syncing`}>
        <SettingItem
          label={t`Derivation Index`}
          description={t`The number of addresses managed by this wallet`}
          control={
            <div className='flex items-center gap-3'>
              <span className='text-md'>{derivationIndex}</span>
              <Button
                variant='secondary'
                size='sm'
                onClick={() => setDeriveOpen(true)}
              >
                <Trans>Increase</Trans>
              </Button>
            </div>
          }
        />

        <SettingItem
          label={t`Resync`}
          description={t`Delete and redownload coin data from the network`}
          control={
            <Button
              variant='destructive'
              size='sm'
              onClick={() => setResyncOpen(true)}
            >
              <Trans>Resync</Trans>
            </Button>
          }
        />
      </SettingsSection>

      <SettingsSection title={t`Status`}>
        <SettingItem
          label={t`Database Stats`}
          description={t`Current database statistics and health information`}
          control={
            <div className='flex gap-2'>
              <Button
                variant='outline'
                size='sm'
                disabled={performingMaintenance}
                onClick={performMaintenance}
              >
                {performingMaintenance && (
                  <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
                )}
                {performingMaintenance ? (
                  <Trans>Optimizing...</Trans>
                ) : (
                  <Trans>Optimize</Trans>
                )}
              </Button>
              <Button
                variant='outline'
                size='sm'
                disabled={loadingStats}
                onClick={fetchDatabaseStats}
              >
                {loadingStats && (
                  <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
                )}
                {loadingStats ? (
                  <Trans>Loading...</Trans>
                ) : (
                  <Trans>Refresh</Trans>
                )}
              </Button>
            </div>
          }
        >
          {dbStats && (
            <div className='mt-3 space-y-3'>
              <div className='grid grid-cols-2 gap-4 text-sm'>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Database Size</Trans>
                  </Label>
                  <div className='text-sm'>
                    {prettyBytes(dbStats.database_size_bytes, { locale: true })}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Compactable Space</Trans>
                  </Label>
                  <div className='text-sm'>
                    {prettyBytes(dbStats.free_space_bytes, { locale: true })} (
                    {dbStats.free_percentage.toFixed(1)}%)
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Total Pages</Trans>
                  </Label>
                  <div className='text-sm'>
                    {dbStats.total_pages.toLocaleString()}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Compactable Pages</Trans>
                  </Label>
                  <div className='text-sm'>
                    {dbStats.free_pages.toLocaleString()}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Page Size</Trans>
                  </Label>
                  <div className='text-sm'>
                    {prettyBytes(dbStats.page_size, { locale: true })}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>WAL Pages</Trans>
                  </Label>
                  <div className='text-sm'>
                    {dbStats.wal_pages.toLocaleString()}
                  </div>
                </div>
              </div>
            </div>
          )}
        </SettingItem>
      </SettingsSection>

      <Dialog open={deriveOpen} onOpenChange={setDeriveOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Increase Derivation Index</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Increase the derivation index to generate new addresses. Setting
                this too high can cause issues, and it can&apos;t be reversed
                without resyncing the wallet.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(handler)} className='space-y-4'>
              <FormField
                control={form.control}
                name='index'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Derivation Index</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder={t`Enter derivation index`}
                        aria-label={t`Derivation index`}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <div className='flex items-center gap-2 my-4'>
                <label
                  htmlFor='unhardened'
                  className='text-sm text-muted-foreground'
                >
                  <Trans>Unhardened Keys</Trans>
                </label>
                <Switch
                  id='unhardened'
                  checked={unhardened}
                  onCheckedChange={(value) => setUnhardened(value)}
                />
              </div>

              <div className='flex items-center gap-2 my-4'>
                <label
                  htmlFor='hardened'
                  className='text-sm text-muted-foreground'
                >
                  <Trans>Hardened Keys</Trans>
                </label>
                <Switch
                  id='hardened'
                  checked={hardened}
                  onCheckedChange={(value) => setHardened(value)}
                />
              </div>

              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setDeriveOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button
                  type='submit'
                  disabled={!form.formState.isValid || pending}
                >
                  {pending && (
                    <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
                  )}
                  {pending ? (
                    <Trans>Generating</Trans>
                  ) : (
                    <Trans>Generate</Trans>
                  )}
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <ResyncDialog
        open={resyncOpen}
        setOpen={setResyncOpen}
        submit={async (options) => {
          await commands.resync({ fingerprint, ...options });
        }}
      />

      <Dialog open={maintenanceOpen} onOpenChange={setMaintenanceOpen}>
        <DialogContent className='max-w-md'>
          <DialogHeader>
            <DialogTitle>
              <Trans>Database Maintenance Complete</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Database optimization has completed successfully.</Trans>
            </DialogDescription>
          </DialogHeader>
          {maintenanceResults && (
            <div className='space-y-3'>
              <div className='grid grid-cols-2 gap-4 text-sm'>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Total Duration</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.total_duration_ms}ms
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Pages Vacuumed</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.pages_vacuumed.toLocaleString()}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Analyze Duration</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.analyze_duration_ms}ms
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>WAL Checkpoint Duration</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.wal_checkpoint_duration_ms}ms
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>WAL Pages Checkpointed</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.wal_pages_checkpointed.toLocaleString()}
                  </div>
                </div>
                <div>
                  <Label className='text-xs font-medium text-muted-foreground'>
                    <Trans>Vacuum Duration</Trans>
                  </Label>
                  <div className='text-sm'>
                    {maintenanceResults.vacuum_duration_ms}ms
                  </div>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button onClick={() => setMaintenanceOpen(false)}>
              <Trans>Close</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function WebhooksSettings() {
  const { addError } = useErrors();
  const [webhooks, setWebhooks] = useState<WebhookEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [updatingId, setUpdatingId] = useState<string | null>(null);

  const loadWebhooks = useCallback(() => {
    setLoading(true);
    commands
      .getWebhooks({})
      .then((response) => {
        setWebhooks(response.webhooks);
      })
      .catch(addError)
      .finally(() => setLoading(false));
  }, [addError]);

  const handleToggle = useCallback(
    (webhookId: string, enabled: boolean) => {
      setUpdatingId(webhookId);
      commands
        .updateWebhook({ webhook_id: webhookId, enabled })
        .then(() => {
          loadWebhooks();
        })
        .catch(addError)
        .finally(() => setUpdatingId(null));
    },
    [addError, loadWebhooks],
  );

  useEffect(() => {
    loadWebhooks();
  }, [loadWebhooks]);

  // Refresh webhooks when webhook-specific events occur
  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      // Only refresh on webhook-related events
      if (
        event.payload.type === 'webhooks_changed' ||
        event.payload.type === 'webhook_invoked'
      ) {
        loadWebhooks();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadWebhooks]);

  if (loading) {
    return (
      <SettingsSection title={t`Webhooks`}>
        <div className='p-4 flex items-center justify-center'>
          <LoaderCircleIcon className='h-5 w-5 animate-spin text-muted-foreground' />
        </div>
      </SettingsSection>
    );
  }

  if (webhooks.length === 0) {
    return (
      <SettingsSection title={t`Webhooks`}>
        <div className='p-4 text-center'>
          <div className='text-sm text-muted-foreground'>
            <Trans>No webhooks registered</Trans>
          </div>
          <div className='text-xs text-muted-foreground mt-1'>
            <Trans>
              Register webhooks via the API to receive real-time notifications
              about wallet events
            </Trans>
          </div>
        </div>
      </SettingsSection>
    );
  }

  return (
    <SettingsSection title={t`Webhooks`}>
      {webhooks.map((webhook, index) => {
        // Determine status color and label
        let statusColor = 'bg-gray-400';
        let statusLabel = <Trans>Disabled</Trans>;
        const failures = webhook.consecutive_failures ?? 0;

        if (!webhook.enabled && failures >= 5) {
          // Auto-disabled due to consecutive failures
          statusColor = 'bg-red-500';
          statusLabel = <Trans>Auto-disabled</Trans>;
        } else if (webhook.enabled) {
          // Check health based on delivery attempts
          if (
            webhook.last_delivered_at === null &&
            webhook.last_delivery_attempt_at !== null
          ) {
            // Never successfully delivered, but attempts have been made
            statusColor = 'bg-red-500';
            statusLabel = <Trans>Failing</Trans>;
          } else if (
            webhook.last_delivery_attempt_at !== null &&
            webhook.last_delivered_at !== null &&
            webhook.last_delivery_attempt_at > webhook.last_delivered_at
          ) {
            // Most recent attempt failed
            statusColor = 'bg-yellow-500';
            statusLabel = <Trans>Warning</Trans>;
          } else {
            // Enabled and healthy
            statusColor = 'bg-green-500';
            statusLabel = <Trans>Enabled</Trans>;
          }
        }

        return (
          <div key={webhook.id} className='p-4'>
            <div className='space-y-2'>
              {/* Header with Status, ID, and Toggle */}
              <div className='flex items-center gap-2'>
                <div className={`h-2.5 w-2.5 rounded-full ${statusColor}`} />
                <div className='font-medium text-sm'>{statusLabel}</div>
                <div className='text-xs text-muted-foreground font-mono flex-1'>
                  {webhook.id}
                </div>
                <Switch
                  checked={webhook.enabled}
                  disabled={updatingId === webhook.id}
                  onCheckedChange={(checked) =>
                    handleToggle(webhook.id, checked)
                  }
                />
              </div>

              {/* URL */}
              <div>
                <Label className='text-xs text-muted-foreground'>
                  <Trans>URL</Trans>
                </Label>
                <div className='text-xs break-all font-mono bg-muted px-1.5 py-0.5 rounded mt-0.5'>
                  {webhook.url}
                </div>
              </div>

              {/* Events */}
              <div>
                <Label className='text-xs text-muted-foreground'>
                  <Trans>Events</Trans>
                </Label>
                <div className='text-xs mt-0.5'>
                  {webhook.events === null ? (
                    <span className='text-muted-foreground italic'>
                      <Trans>All events</Trans>
                    </span>
                  ) : webhook.events.length === 0 ? (
                    <span className='text-muted-foreground italic'>
                      <Trans>No events</Trans>
                    </span>
                  ) : (
                    <div className='flex flex-wrap gap-1'>
                      {webhook.events.map((event) => (
                        <span
                          key={event}
                          className='inline-flex items-center px-1.5 py-0.5 rounded-md bg-muted text-xs font-mono'
                        >
                          {event}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Delivery timestamps and failure count */}
              {(webhook.last_delivered_at ||
                webhook.last_delivery_attempt_at ||
                failures > 0) && (
                <div className='grid grid-cols-2 gap-2'>
                  {webhook.last_delivered_at && (
                    <div>
                      <Label className='text-xs text-muted-foreground'>
                        <Trans>Last Delivered</Trans>
                      </Label>
                      <div className='text-xs text-muted-foreground mt-0.5'>
                        {new Date(
                          webhook.last_delivered_at * 1000,
                        ).toLocaleString()}
                      </div>
                    </div>
                  )}
                  {webhook.last_delivery_attempt_at && (
                    <div>
                      <Label className='text-xs text-muted-foreground'>
                        <Trans>Last Attempt</Trans>
                      </Label>
                      <div className='text-xs text-muted-foreground mt-0.5'>
                        {new Date(
                          webhook.last_delivery_attempt_at * 1000,
                        ).toLocaleString()}
                      </div>
                    </div>
                  )}
                  {failures > 0 && (
                    <div>
                      <Label className='text-xs text-muted-foreground'>
                        <Trans>Consecutive Failures</Trans>
                      </Label>
                      <div
                        className={`text-xs mt-0.5 ${failures >= 5 ? 'text-red-500 font-medium' : 'text-muted-foreground'}`}
                      >
                        {failures}
                        {failures >= 5 && (
                          <span className='ml-1'>
                            (<Trans>auto-disabled at 5</Trans>)
                          </span>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Divider between webhooks, but not after the last one */}
            {index < webhooks.length - 1 && (
              <div className='mt-2 border-t border-border' />
            )}
          </div>
        );
      })}
    </SettingsSection>
  );
}
