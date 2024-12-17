import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
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
import { useErrors } from '@/hooks/useErrors';
import useInitialization from '@/hooks/useInitialization';
import { useWallet } from '@/hooks/useWallet';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { clearState, fetchState } from '@/state';
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

export default function Settings() {
  const initialized = useInitialization();
  const wallet = useWallet(initialized);

  return (
    <Layout>
      <Header title='Settings' />
      <Container className='max-w-2xl'>
        <div className='flex flex-col gap-4'>
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

  return (
    <Card>
      <CardHeader>
        <CardTitle>Global</CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='flex items-center space-x-2'>
            <Switch
              id='dark-mode'
              checked={dark}
              onCheckedChange={(checked) => setDark(checked)}
            />
            <Label htmlFor='dark-mode'>Dark Mode</Label>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function WalletConnectSettings() {
  const { pair, sessions, disconnect } = useWalletConnect();
  const [uri, setUri] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  const handlePair = async () => {
    try {
      setError(null);
      await pair(uri);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect');
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>WalletConnect</CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='flex flex-col gap-4'>
            <div className='flex flex-col gap-2'>
              {sessions.map((session: any) => (
                <div
                  key={session.topic}
                  className='flex items-center justify-between gap-1'
                >
                  <div className='flex gap-2 items-center'>
                    <img
                      src={session.peer.metadata.icons[0]}
                      alt={session.peer.metadata.name}
                      className='h-8 w-8'
                    />
                    <span className='text-sm font-medium'>
                      {session.peer.metadata.name}
                    </span>
                  </div>
                  <Button
                    variant='destructive'
                    size='sm'
                    onClick={() => disconnect(session.topic)}
                  >
                    Disconnect
                  </Button>
                </div>
              ))}
              {sessions.length === 0 && (
                <span className='text-sm text-gray-500'>
                  No active sessions
                </span>
              )}
            </div>
            <div className='flex flex-col gap-2'>
              <div className='flex gap-2'>
                <Input
                  id='wc-uri'
                  type='text'
                  placeholder='Paste WalletConnect URI'
                  onChange={(e) => setUri(e.target.value)}
                />
                <Button onClick={handlePair}>Connect</Button>
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
        <CardTitle>Network</CardTitle>
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
            <Label htmlFor='discover-peers'>Discover peers automatically</Label>
          </div>
          <div className='grid gap-3'>
            <Label htmlFor='target-peers'>Target Peers</Label>
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
            <Label htmlFor='network'>Network ID</Label>
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
                <SelectValue placeholder='Select network' />
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
        <CardTitle>Wallet</CardTitle>
      </CardHeader>
      <CardContent>
        <div className='grid gap-6'>
          <div className='grid gap-3'>
            <Label htmlFor='walletName'>Wallet Name</Label>
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
              Generate addresses automatically
            </Label>
          </div>
          <div className='grid gap-3'>
            <Label htmlFor='address-batch-size'>Address Batch Size</Label>
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
