import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
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
import { useWallet } from '@/hooks/useWallet';
import { clearState, fetchState } from '@/state';
import { useContext, useEffect, useState } from 'react';
import { DarkModeContext } from '../App';
import { commands, NetworkConfig, WalletConfig, WalletInfo } from '../bindings';
import { isValidU32 } from '../validation';

export default function Settings() {
  const { wallet } = useWallet();

  return (
    <Layout>
      <Header title='Settings' />
      <Container className='max-w-2xl'>
        <div className='flex flex-col gap-4'>
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

function NetworkSettings() {
  const [discoverPeers, setDiscoverPeers] = useState<boolean | null>(null);
  const [targetPeersText, setTargetPeers] = useState<string | null>(null);
  const [networkId, setNetworkId] = useState<string | null>(null);

  const targetPeers =
    targetPeersText === null ? null : parseInt(targetPeersText);

  const invalidTargetPeers =
    targetPeers === null || !isValidU32(targetPeers, 1);

  const [config, setConfig] = useState<NetworkConfig | null>(null);

  useEffect(() => {
    commands.networkConfig().then((res) => {
      if (res.status === 'error') {
        return;
      }
      setConfig(res.data);
    });
  }, []);

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
                commands.setDiscoverPeers(checked);
                setDiscoverPeers(checked);
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
                  commands.setTargetPeers(targetPeers);
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
                  commands.setNetworkId(networkId).then(() => {
                    fetchState();
                  });
                  setNetworkId(networkId);
                }
              }}
            >
              <SelectTrigger id='network' aria-label='Select network'>
                <SelectValue placeholder='Select network' />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='mainnet'>Mainnet</SelectItem>
                <SelectItem value='testnet11'>Testnet 11</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function WalletSettings(props: { wallet: WalletInfo }) {
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
    commands.walletConfig(props.wallet.fingerprint).then((res) => {
      if (res.status === 'error') {
        return;
      }
      setConfig(res.data);
    });
  }, [props.wallet.fingerprint]);

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
                    commands.renameWallet(props.wallet.fingerprint, name);
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
                commands.setDeriveAutomatically(
                  props.wallet.fingerprint,
                  checked,
                );
                setDeriveAutomatically(checked);
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
                  commands.setDerivationBatchSize(
                    props.wallet.fingerprint,
                    derivationBatchSize,
                  );
                }
              }}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
