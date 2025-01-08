import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
// import { PasteInput } from '@/components/PasteInput';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
import { useLanguage } from '@/contexts/LanguageContext';
import { useErrors } from '@/hooks/useErrors';
import useInitialization from '@/hooks/useInitialization';
import { useWallet } from '@/hooks/useWallet';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { clearState, fetchState, useNavigationStore } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { getVersion } from '@tauri-apps/api/app';
import { useContext, useEffect, useState } from 'react';
import { DarkModeContext } from '../App';
import {
  commands,
  KeyInfo,
  Network,
  NetworkConfig,
  WalletConfig,
} from '../bindings';
import { isValidU32 } from '../validation';
import { PasteInput } from '@/components/PasteInput';
import { platform } from '@tauri-apps/plugin-os';
import {
  openAppSettings,
  requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import { useNavigate } from 'react-router-dom';
import { readText } from '@tauri-apps/plugin-clipboard-manager';

export default function Settings() {
  const initialized = useInitialization();
  const wallet = useWallet(initialized);

  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  return (
    <Layout>
      <Header title={t`Settings`} />
      <Container className='max-w-2xl'>
        <Trans>Version {version}</Trans>
        <div className='flex flex-col gap-4 mt-2'>
          <WalletConnectSettings />
          <GlobalSettings />
          <NetworkSettings />
          {wallet && <WalletSettings wallet={wallet} />}
        </div>
      </Container>
    </Layout>
  );
}

function GlobalSettings() {
  const { dark, setDark } = useContext(DarkModeContext);
  const { locale, changeLanguage } = useLanguage();

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <Trans>Global</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent className='space-y-4'>
        <div className='grid gap-6'>
          <div className='flex items-center space-x-2'>
            <Switch
              id='dark-mode'
              checked={dark}
              onCheckedChange={(checked) => setDark(checked)}
            />
            <Label htmlFor='dark-mode'>
              <Trans>Dark Mode</Trans>
            </Label>
          </div>
        </div>
        <div className='grid gap-3'>
          <Label htmlFor='language'>
            <Trans>Language</Trans>
          </Label>
          <Select value={locale} onValueChange={changeLanguage}>
            <SelectTrigger id='language'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='en-US'>English</SelectItem>
              <SelectItem value='de-DE'>Deutsch</SelectItem>
              <SelectItem value='zh-CN'>Chinese</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </CardContent>
    </Card>
  );
}

function WalletConnectSettings() {
  const { pair, sessions, disconnect, connecting } = useWalletConnect();
  const [uri, setUri] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';
  const navigate = useNavigate();
  const { returnValues, setReturnValue } = useNavigationStore();

  useEffect(() => {
    const returnValue = returnValues[location.pathname];
    if (!returnValue) return;

    if (returnValue.status === 'success' && returnValue?.data) {
      setUri(returnValue.data);
      setReturnValue(location.pathname, { status: 'completed' });
    }
  }, [returnValues]);

  const handlePair = async () => {
    try {
      setError(null);
      console.log('connecting ');
      await pair(uri);
      setUri('');
    } catch (err) {
      console.log('called');
      setError(err instanceof Error ? err.message : 'Failed to connect');
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <Trans>WalletConnect</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='flex flex-col gap-4'>
            <div className='flex flex-col gap-2'>
              {sessions.map((session) => (
                <div
                  key={session.topic}
                  className='flex items-center justify-between gap-1'
                >
                  <div className='flex gap-2 items-center'>
                    <img
                      src={session.peer?.metadata?.icons?.[0] ?? ''}
                      alt={session.peer?.metadata?.name ?? t`Unknown App`}
                      className='h-8 w-8'
                    />
                    <span className='text-sm font-medium'>
                      {session.peer?.metadata?.name ?? t`Unknown App`}
                    </span>
                  </div>
                  <Button
                    variant='destructive'
                    size='sm'
                    onClick={() => disconnect(session.topic)}
                  >
                    <Trans>Disconnect</Trans>
                  </Button>
                </div>
              ))}
              {sessions.length === 0 && (
                <span className='text-sm text-gray-500'>
                  <Trans>No active sessions</Trans>
                </span>
              )}
            </div>

            <div className='flex flex-col gap-2'>
              <div className='flex gap-2'>
                <PasteInput
                  value={uri}
                  placeholder={t`Paste WalletConnect URI`}
                  onChange={(e) => setUri(e.target.value)}
                  onEndIconClick={async () => {
                    if (isMobile) {
                      const permissionState = await requestPermissions();
                      if (permissionState === 'denied') {
                        await openAppSettings();
                      } else if (permissionState === 'granted') {
                        navigate('/scan', {
                          state: {
                            returnTo: location.pathname,
                          }, // Use location.pathname
                        });
                      }
                    } else {
                      try {
                        const clipboardText = await readText();
                        if (clipboardText) {
                          setUri(clipboardText);
                        }
                      } catch (error) {
                        console.error('Failed to paste from clipboard:', error);
                      }
                    }
                  }}
                  disabled={connecting}
                />

                <Button onClick={handlePair} disabled={connecting}>
                  {connecting ? (
                    <Trans>Connecting...</Trans>
                  ) : (
                    <Trans>Connect</Trans>
                  )}
                </Button>
              </div>

              {error && <span className='text-sm text-red-500'>{error}</span>}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
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
    <Card>
      <CardHeader>
        <CardTitle>
          <Trans>Network</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='flex items-center space-x-2'>
            <Switch
              id='discover-peers'
              checked={discoverPeers ?? config?.discover_peers ?? true}
              onCheckedChange={(checked) => {
                commands
                  .setDiscoverPeers({ discover_peers: checked })
                  .catch(addError)
                  .finally(() => setDiscoverPeers(checked));
              }}
            />
            <Label htmlFor='discover-peers'>
              <Trans>Discover peers automatically</Trans>
            </Label>
          </div>
          <div className='grid gap-3'>
            <Label htmlFor='target-peers'>
              <Trans>Target Peers</Trans>
            </Label>
            <Input
              id='target-peers'
              type='number'
              className='w-full'
              value={targetPeersText ?? config?.target_peers ?? 500}
              disabled={!(discoverPeers ?? config?.discover_peers)}
              onChange={(event) => setTargetPeers(event.target.value)}
              // TODO error handling
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
          </div>
          <div className='grid gap-3'>
            <Label htmlFor='network'>
              <Trans>Network ID</Trans>
            </Label>
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
              <SelectTrigger id='network' aria-label='Select network'>
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
          </div>
        </div>
      </CardContent>
    </Card>
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
    <Card>
      <CardHeader>
        <CardTitle>
          <Trans>Wallet</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='grid gap-3'>
            <Label htmlFor='walletName'>
              <Trans>Wallet Name</Trans>
            </Label>
            <Input
              id='walletName'
              type='text'
              className='w-full'
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
          </div>
          <div className='flex items-center space-x-2'>
            <Switch
              id='generate-addresses'
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
            <Label htmlFor='generate-addresses'>
              <Trans>Generate addresses automatically</Trans>
            </Label>
          </div>
          <div className='grid gap-3'>
            <Label htmlFor='address-batch-size'>
              <Trans>Address Batch Size</Trans>
            </Label>
            <Input
              id='address-batch-size'
              type='number'
              className='w-full'
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
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
