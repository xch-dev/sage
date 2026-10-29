import { ResyncDialog } from '@/components/dialogs/ResyncDialog';
import { EmojiPicker } from '@/components/EmojiPicker';
import { PasteInput } from '@/components/PasteInput';
import { Button } from '@/components/ui/button';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import {
  clearState,
  fetchState,
  updateSyncStatus,
  useWalletState,
} from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import prettyBytes from 'pretty-bytes';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import {
  commands,
  GetDatabaseStatsResponse,
  KeyInfo,
  Network,
  PerformDatabaseMaintenanceResponse,
  Wallet,
} from '../../bindings';
import { SettingItem, SettingsSection } from './SettingsSection';

export function WalletSettings({ fingerprint }: { fingerprint: number }) {
  const { addError } = useErrors();
  // The logged-in wallet, which the sidebar shows the name and emoji of.
  const { wallet: activeWallet, setWallet: setActiveWallet } = useWallet();

  const [wallet, setWallet] = useState<Wallet | null>(null);
  // Watch and vault wallets have no change address, since they don't derive
  // addresses.
  const { isWatchOnly: isWatch, isVault } = useWallet();
  const isWatchOnly = isWatch || isVault;
  const [localName, setLocalName] = useState<string>('');
  const [localChangeAddress, setLocalChangeAddress] = useState('');
  const [networks, setNetworks] = useState<Network[]>([]);
  const [defaultNetwork, setDefaultNetwork] = useState<string | null>(null);

  const saveChangeAddress = (address: string) => {
    const trimmedAddress = address.trim();
    const currentAddress = wallet?.change_address || '';
    if (trimmedAddress !== currentAddress) {
      setChangeAddress(trimmedAddress || null);
    }
  };

  const {
    handleScanOrPaste: handleScanOrPasteChangeAddress,
    handleScanImage: handleScanImageChangeAddress,
  } = useScannerOrClipboard((scanResValue) => {
    setLocalChangeAddress(scanResValue);
    saveChangeAddress(scanResValue);
  });

  useEffect(() => {
    commands
      .getNetworks({})
      .then((data) => setNetworks(data.networks))
      .catch(addError);

    commands
      .networkConfig()
      .then((config) => setDefaultNetwork(config.default_network))
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
  }, [addError, fingerprint]);

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

  return (
    <SettingsSection title={t`Details`}>
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
                .renameWallet({
                  fingerprint,
                  name: localName,
                })
                .then(() => {
                  if (wallet) {
                    setWallet({ ...wallet, name: localName });
                  }
                  if (activeWallet?.fingerprint === fingerprint) {
                    setActiveWallet({ ...activeWallet, name: localName });
                  }
                })
                .catch(addError);
            }}
          />
        }
      />

      <SettingItem
        label={t`Emoji`}
        description={t`Shown next to the wallet's name`}
        control={
          <EmojiPicker
            value={wallet?.emoji}
            placeholder={t`Choose emoji`}
            disabled={!wallet}
            onChange={(emoji) => {
              commands
                .setWalletEmoji({ fingerprint, emoji })
                .then(() => {
                  if (wallet) {
                    setWallet({ ...wallet, emoji });
                  }
                  if (activeWallet?.fingerprint === fingerprint) {
                    setActiveWallet({ ...activeWallet, emoji });
                  }
                })
                .catch(addError);
            }}
          />
        }
      />

      <SettingItem
        label={t`Override Network`}
        description={
          wallet?.network || !defaultNetwork
            ? t`Override the default network for this wallet`
            : t`Using the default network (${defaultNetwork})`
        }
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

      {!isWatchOnly && (
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
              onScanImage={handleScanImageChangeAddress}
            />
          </div>
        </SettingItem>
      )}
    </SettingsSection>
  );
}

export function WalletSyncSettings({ fingerprint }: { fingerprint: number }) {
  const { addError } = useErrors();

  const walletState = useWalletState();

  const [key, setKey] = useState<KeyInfo | null>(null);
  // Holds the wallet's Delta Sync override.
  const [wallet, setWallet] = useState<Wallet | null>(null);
  // Watch and vault wallets have no derivations, since they don't derive
  // addresses.
  const { isWatchOnly: isWatch, isVault } = useWallet();
  const isWatchOnly = isWatch || isVault;
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
  const [defaultDeltaSync, setDefaultDeltaSync] = useState<boolean | null>(
    null,
  );

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
    commands.walletConfig(fingerprint).then(setWallet).catch(addError);
    commands
      .defaultWalletConfig()
      .then((defaults) => setDefaultDeltaSync(defaults.delta_sync))
      .catch(addError);
    fetchDatabaseStats();
  }, [addError, fingerprint, fetchDatabaseStats]);

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

  const setDeltaSyncOverride = (deltaSync: boolean | null) => {
    commands
      .setDeltaSyncOverride({ fingerprint, delta_sync: deltaSync })
      .then(() => {
        if (wallet) setWallet({ ...wallet, delta_sync: deltaSync });
      })
      .catch(addError);
  };

  const deltaSyncValue =
    wallet?.delta_sync === true
      ? 'on'
      : wallet?.delta_sync === false
        ? 'off'
        : 'default';

  return (
    <>
      <SettingsSection title={t`Syncing`}>
        <SettingItem
          label={t`Delta Sync`}
          description={t`A more lightweight sync that only checks unseen blocks. Takes effect the next time you open the wallet.`}
          control={
            <Select
              value={deltaSyncValue}
              disabled={!wallet}
              onValueChange={(value) =>
                setDeltaSyncOverride(
                  value === 'default' ? null : value === 'on',
                )
              }
            >
              <SelectTrigger className='w-[160px]' aria-label={t`Delta Sync`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='default'>
                  {defaultDeltaSync === null
                    ? t`Default`
                    : defaultDeltaSync
                      ? t`Default (On)`
                      : t`Default (Off)`}
                </SelectItem>
                <SelectItem value='on'>{t`On`}</SelectItem>
                <SelectItem value='off'>{t`Off`}</SelectItem>
              </SelectContent>
            </Select>
          }
        />
      </SettingsSection>

      {!isWatchOnly && (
        <SettingsSection title={t`Addresses`}>
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
        </SettingsSection>
      )}

      <SettingsSection title={t`Database`}>
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

      <SettingsSection title={t`Danger Zone`} destructive>
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
    </>
  );
}
