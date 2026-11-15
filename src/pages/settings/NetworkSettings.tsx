import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { clearState, fetchState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  commands,
  Network,
  NetworkConfig,
  WalletDefaults,
} from '../../bindings';
import { isValidU32 } from '../../validation';
import { SettingItem, SettingsSection } from './SettingsSection';

export function NetworkSettings() {
  const { addError } = useErrors();
  const { wallet } = useWallet();

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
    <>
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
                aria-label={t`Default Network`}
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
      </SettingsSection>

      <SettingsSection title={t`Peers`}>
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

        {wallet && (
          <SettingItem
            label={t`Connected Peers`}
            description={t`View and manage the peers you're connected to`}
            control={
              <Button variant='outline' size='sm' asChild>
                <Link to='/peers'>
                  <Trans>View Peers</Trans>
                </Link>
              </Button>
            }
          />
        )}
      </SettingsSection>

      <SyncSettings
        mempoolDiscovery={config?.mempool_discovery ?? true}
        setMempoolDiscovery={(mempool_discovery) => {
          if (config) setConfig({ ...config, mempool_discovery });
        }}
      />
    </>
  );
}

interface SyncSettingsProps {
  mempoolDiscovery: boolean;
  setMempoolDiscovery: (mempoolDiscovery: boolean) => void;
}

function SyncSettings({
  mempoolDiscovery,
  setMempoolDiscovery,
}: SyncSettingsProps) {
  const { addError } = useErrors();

  const [defaults, setDefaults] = useState<WalletDefaults | null>(null);

  useEffect(() => {
    commands.defaultWalletConfig().then(setDefaults).catch(addError);
  }, [addError]);

  return (
    <SettingsSection title={t`Syncing`}>
      <SettingItem
        label={t`Delta Sync`}
        description={t`A more lightweight sync that only checks unseen blocks`}
        control={
          <Switch
            checked={defaults?.delta_sync ?? true}
            onCheckedChange={(checked) => {
              if (!defaults) return;
              setDefaults({ ...defaults, delta_sync: checked });
              commands.setDeltaSync({ delta_sync: checked }).catch(addError);
            }}
          />
        }
      />
      <SettingItem
        label={t`Mempool Discovery`}
        description={t`Look up pending transactions with Coinset, so they show before they're confirmed, even after a resync. Coinset can see which pending transactions are yours.`}
        control={
          <Switch
            checked={mempoolDiscovery}
            onCheckedChange={(checked) => {
              commands
                .setMempoolDiscovery({ mempool_discovery: checked })
                .catch(addError)
                .finally(() => setMempoolDiscovery(checked));
            }}
          />
        }
      />
    </SettingsSection>
  );
}
