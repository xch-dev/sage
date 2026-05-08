import { useMemo, useState } from 'react';
import type {
  SageAppCapabilityDefinitionView,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  SystemSageAppView,
  UserBridgeCapability,
  UserSageAppView,
} from '@sage-system-app/sdk';
import type { NetworkPermissionScheme, PermissionEntry } from './types';
import {
  buildCapabilityEntries,
  buildNetworkEntries,
  capabilityDefinitionMap,
  isUserGrantableCapability,
  sortPermissionEntries,
} from './permissionEntries';
import { buildGroupedPermissionTree } from './permissionTree';
import { networkKey, sortNetworkEntries } from './utils';
import { PermissionSection } from './PermissionSection';

interface Props {
  app: UserSageAppView | SystemSageAppView;
  grantedPermissions: SageGrantedPermissionsView;
  capabilityDefinitions: SageAppCapabilityDefinitionView[];
  onGrantedPermissionsChange?: (next: SageGrantedPermissionsInput) => void;
  editable?: boolean;
}

type NetworkWhitelistByNetwork = Partial<
  Record<string, SageNetworkWhitelistEntry[]>
>;

export function PermissionsEditor({
  app,
  grantedPermissions,
  capabilityDefinitions,
  onGrantedPermissionsChange,
  editable = true,
}: Props) {
  const manifest =
    'pendingUpdate' in app && app.pendingUpdate
      ? app.pendingUpdate.manifest
      : app.common.activeSnapshot.manifest;

  const [showAllOptional, setShowAllOptional] = useState(false);

  const definitionsByKey = useMemo(
    () => capabilityDefinitionMap(capabilityDefinitions),
    [capabilityDefinitions],
  );

  const grantedCapabilities = useMemo(
    () => grantedPermissions.capabilities ?? [],
    [grantedPermissions.capabilities],
  );

  const grantedNetworkWhitelist = useMemo<SageNetworkWhitelistEntry[]>(
    () => grantedPermissions.network.whitelist ?? [],
    [grantedPermissions.network.whitelist],
  );

  const grantedNetworkWhitelistByNetwork = useMemo<NetworkWhitelistByNetwork>(
    () => grantedPermissions.network.whitelistByNetwork ?? {},
    [grantedPermissions.network.whitelistByNetwork],
  );

  const requestedRequiredCapabilities = useMemo(
    () => manifest.permissions?.capabilities?.required ?? [],
    [manifest.permissions?.capabilities?.required],
  );

  const requestedOptionalCapabilities = useMemo(
    () => manifest.permissions?.capabilities?.optional ?? [],
    [manifest.permissions?.capabilities?.optional],
  );

  const requestedRequiredNetwork = useMemo<SageNetworkWhitelistEntry[]>(
    () => manifest.permissions?.network?.whitelist?.required ?? [],
    [manifest.permissions?.network?.whitelist?.required],
  );

  const requestedOptionalNetwork = useMemo<SageNetworkWhitelistEntry[]>(
    () => manifest.permissions?.network?.whitelist?.optional ?? [],
    [manifest.permissions?.network?.whitelist?.optional],
  );

  const requestedNetworkByNetwork = useMemo(
    () => manifest.permissions?.network?.whitelistByNetwork ?? {},
    [manifest.permissions?.network?.whitelistByNetwork],
  );

  const userGrantableRequiredCapabilities = useMemo(
    () =>
      requestedRequiredCapabilities.filter((capability) =>
        isUserGrantableCapability(capability, definitionsByKey),
      ),
    [requestedRequiredCapabilities, definitionsByKey],
  );

  const requiredEntries = useMemo(() => {
    const capabilityEntries = buildCapabilityEntries(
      requestedRequiredCapabilities,
      [],
      grantedCapabilities,
      definitionsByKey,
    );

    const sharedNetworkEntries = buildNetworkEntries(
      requestedRequiredNetwork,
      requestedOptionalNetwork,
      grantedNetworkWhitelist,
      'required',
      null,
    );

    const networkSpecificEntries = Object.entries(
      requestedNetworkByNetwork,
    ).flatMap(([networkId, whitelist]) =>
      buildNetworkEntries(
        whitelist?.required ?? [],
        whitelist?.optional ?? [],
        grantedNetworkWhitelistByNetwork[networkId] ?? [],
        'required',
        networkId,
      ),
    );

    return sortPermissionEntries([
      ...capabilityEntries,
      ...sharedNetworkEntries,
      ...networkSpecificEntries,
    ]);
  }, [
    requestedRequiredCapabilities,
    grantedCapabilities,
    requestedRequiredNetwork,
    requestedOptionalNetwork,
    grantedNetworkWhitelist,
    requestedNetworkByNetwork,
    grantedNetworkWhitelistByNetwork,
    definitionsByKey,
  ]);

  const optionalEntries = useMemo(() => {
    const capabilityEntries = buildCapabilityEntries(
      [],
      requestedOptionalCapabilities,
      grantedCapabilities,
      definitionsByKey,
    );

    const sharedNetworkEntries = buildNetworkEntries(
      requestedRequiredNetwork,
      requestedOptionalNetwork,
      grantedNetworkWhitelist,
      'optional',
      null,
    );

    const networkSpecificEntries = Object.entries(
      requestedNetworkByNetwork,
    ).flatMap(([networkId, whitelist]) =>
      buildNetworkEntries(
        whitelist?.required ?? [],
        whitelist?.optional ?? [],
        grantedNetworkWhitelistByNetwork[networkId] ?? [],
        'optional',
        networkId,
      ),
    );

    return sortPermissionEntries([
      ...capabilityEntries,
      ...sharedNetworkEntries,
      ...networkSpecificEntries,
    ]);
  }, [
    requestedOptionalCapabilities,
    grantedCapabilities,
    requestedRequiredNetwork,
    requestedOptionalNetwork,
    grantedNetworkWhitelist,
    requestedNetworkByNetwork,
    grantedNetworkWhitelistByNetwork,
    definitionsByKey,
  ]);

  const grantedOptionalEntries = useMemo(
    () => optionalEntries.filter((entry) => entry.granted),
    [optionalEntries],
  );

  const ungrantedOptionalEntries = useMemo(
    () => optionalEntries.filter((entry) => !entry.granted),
    [optionalEntries],
  );

  const visibleOptionalEntries = useMemo(
    () => (showAllOptional ? optionalEntries : grantedOptionalEntries),
    [showAllOptional, optionalEntries, grantedOptionalEntries],
  );

  const requiredGroups = useMemo(
    () => buildGroupedPermissionTree(requiredEntries),
    [requiredEntries],
  );

  const optionalGroups = useMemo(
    () => buildGroupedPermissionTree(visibleOptionalEntries),
    [visibleOptionalEntries],
  );

  function emitGrantedPermissions(next: SageGrantedPermissionsInput) {
    onGrantedPermissionsChange?.(next);
  }

  function keyToNetworkEntry(key: string): SageNetworkWhitelistEntry | null {
    const [scheme, host] = key.split('://');

    if (!scheme || !host) {
      return null;
    }

    return { scheme, host };
  }

  function handleToggleEntry(
    entry: PermissionEntry,
    nextGranted: boolean,
    scheme?: NetworkPermissionScheme,
  ) {
    if (!editable) {
      return;
    }

    if (entry.kind === 'capability') {
      if (entry.required) {
        return;
      }

      const nextSet = new Set<UserBridgeCapability>(grantedCapabilities);

      if (nextGranted) {
        nextSet.add(entry.capability);
      } else {
        nextSet.delete(entry.capability);
      }

      for (const requiredCapability of userGrantableRequiredCapabilities) {
        nextSet.add(requiredCapability);
      }

      emitGrantedPermissions({
        capabilities: [...nextSet].sort((a, b) => a.localeCompare(b)),
        network: {
          whitelist: grantedNetworkWhitelist,
          whitelistByNetwork: grantedNetworkWhitelistByNetwork,
        },
      });

      return;
    }

    if (!scheme) {
      return;
    }

    const networkId = entry.networkId;

    const currentWhitelist: SageNetworkWhitelistEntry[] =
      networkId === null
        ? grantedNetworkWhitelist
        : (grantedNetworkWhitelistByNetwork[networkId] ?? []);

    const requiredNetwork: SageNetworkWhitelistEntry[] =
      networkId === null
        ? requestedRequiredNetwork
        : (requestedNetworkByNetwork[networkId]?.required ?? []);

    const nextKeys = new Set<string>(
      currentWhitelist.map((item: SageNetworkWhitelistEntry) =>
        networkKey(item),
      ),
    );

    for (const requiredEntry of requiredNetwork) {
      nextKeys.add(networkKey(requiredEntry));
    }

    const httpsKey = `https://${entry.host}`;
    const wssKey = `wss://${entry.host}`;

    if (scheme === 'https') {
      if (nextGranted) {
        nextKeys.add(httpsKey);
      } else if (!nextKeys.has(wssKey)) {
        nextKeys.delete(httpsKey);
      }
    }

    if (scheme === 'wss') {
      if (nextGranted) {
        nextKeys.add(wssKey);
        nextKeys.add(httpsKey);
      } else {
        nextKeys.delete(wssKey);
      }
    }

    for (const key of Array.from(nextKeys)) {
      if (key.startsWith('wss://')) {
        nextKeys.add(`https://${key.slice('wss://'.length)}`);
      }
    }

    const nextWhitelist = Array.from(nextKeys)
      .map(keyToNetworkEntry)
      .filter((item): item is SageNetworkWhitelistEntry => item !== null);

    if (networkId !== null) {
      emitGrantedPermissions({
        capabilities: grantedCapabilities,
        network: {
          whitelist: grantedNetworkWhitelist,
          whitelistByNetwork: {
            ...grantedNetworkWhitelistByNetwork,
            [networkId]: sortNetworkEntries(nextWhitelist),
          },
        },
      });

      return;
    }

    emitGrantedPermissions({
      capabilities: grantedCapabilities,
      network: {
        whitelist: sortNetworkEntries(nextWhitelist),
        whitelistByNetwork: grantedNetworkWhitelistByNetwork,
      },
    });
  }

  if (requiredEntries.length === 0 && optionalEntries.length === 0) {
    return (
      <div className='rounded-xl border border-border px-3 py-4 text-sm text-muted-foreground'>
        This app does not request any permissions.
      </div>
    );
  }

  return (
    <div className='space-y-4'>
      {requiredGroups.length > 0 ? (
        <PermissionSection
          title='Required permissions'
          groups={requiredGroups}
          editable={editable}
          onToggleEntry={handleToggleEntry}
        />
      ) : null}

      {optionalEntries.length > 0 ? (
        <PermissionSection
          title='Optional permissions'
          groups={optionalGroups}
          editable={editable}
          separated={requiredGroups.length > 0}
          onToggleEntry={handleToggleEntry}
          trailingAction={
            !showAllOptional && ungrantedOptionalEntries.length > 0 ? (
              <button
                type='button'
                className='inline-flex items-center rounded-md px-2 py-1 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground'
                onClick={() => setShowAllOptional(true)}
              >
                Show {ungrantedOptionalEntries.length} more optional permission
                {ungrantedOptionalEntries.length === 1 ? '' : 's'}
              </button>
            ) : null
          }
        />
      ) : null}
    </div>
  );
}
