import { useMemo, useState } from 'react';
import type {
  SageAppCapabilityDefinitionView,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SystemSageAppView,
  UserBridgeCapability,
  UserSageAppView,
} from '@sage-system-app/sdk';
import type { PermissionEntry } from './types';
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

  const [showOptional, setShowOptional] = useState(false);

  const definitionsByKey = useMemo(
    () => capabilityDefinitionMap(capabilityDefinitions),
    [capabilityDefinitions],
  );

  const grantedCapabilities = useMemo(
    () => grantedPermissions.capabilities ?? [],
    [grantedPermissions.capabilities],
  );

  const grantedNetworkWhitelist = useMemo(
    () => grantedPermissions.network.whitelist ?? [],
    [grantedPermissions.network.whitelist],
  );

  const requestedRequiredCapabilities = useMemo(
    () => manifest.permissions?.capabilities?.required ?? [],
    [manifest.permissions?.capabilities?.required],
  );

  const requestedOptionalCapabilities = useMemo(
    () => manifest.permissions?.capabilities?.optional ?? [],
    [manifest.permissions?.capabilities?.optional],
  );

  const requestedRequiredNetwork = useMemo(
    () => manifest.permissions?.network?.whitelist?.required ?? [],
    [manifest.permissions?.network?.whitelist?.required],
  );

  const requestedOptionalNetwork = useMemo(
    () => manifest.permissions?.network?.whitelist?.optional ?? [],
    [manifest.permissions?.network?.whitelist?.optional],
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

    const networkEntries = buildNetworkEntries(
      requestedRequiredNetwork,
      [],
      grantedNetworkWhitelist,
    );

    return sortPermissionEntries([...capabilityEntries, ...networkEntries]);
  }, [
    requestedRequiredCapabilities,
    grantedCapabilities,
    requestedRequiredNetwork,
    grantedNetworkWhitelist,
    definitionsByKey,
  ]);

  const optionalEntries = useMemo(() => {
    const capabilityEntries = buildCapabilityEntries(
      [],
      requestedOptionalCapabilities,
      grantedCapabilities,
      definitionsByKey,
    );

    const networkEntries = buildNetworkEntries(
      [],
      requestedOptionalNetwork,
      grantedNetworkWhitelist,
    );

    return sortPermissionEntries([...capabilityEntries, ...networkEntries]);
  }, [
    requestedOptionalCapabilities,
    grantedCapabilities,
    requestedOptionalNetwork,
    grantedNetworkWhitelist,
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

  const requiredGroups = useMemo(
    () => buildGroupedPermissionTree(requiredEntries),
    [requiredEntries],
  );

  const grantedOptionalGroups = useMemo(
    () => buildGroupedPermissionTree(grantedOptionalEntries),
    [grantedOptionalEntries],
  );

  const ungrantedOptionalGroups = useMemo(
    () => buildGroupedPermissionTree(ungrantedOptionalEntries),
    [ungrantedOptionalEntries],
  );

  function emitGrantedPermissions(next: SageGrantedPermissionsInput) {
    onGrantedPermissionsChange?.(next);
  }

  function handleToggleEntry(entry: PermissionEntry, nextGranted: boolean) {
    if (!editable || entry.required) {
      return;
    }

    if (entry.kind === 'capability') {
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
        },
      });

      return;
    }

    const requiredKeys = new Set<string>(
      requestedRequiredNetwork.map((item) => networkKey(item)),
    );

    const nextOptional = requestedOptionalNetwork.filter((item) => {
      const key = networkKey(item);

      if (requiredKeys.has(key)) {
        return false;
      }

      if (key !== entry.key) {
        return grantedNetworkWhitelist.some(
          (grantedEntry) => networkKey(grantedEntry) === key,
        );
      }

      return nextGranted;
    });

    emitGrantedPermissions({
      capabilities: grantedCapabilities,
      network: {
        whitelist: sortNetworkEntries([
          ...requestedRequiredNetwork,
          ...nextOptional,
        ]),
      },
    });
  }

  if (
    requiredEntries.length === 0 &&
    grantedOptionalEntries.length === 0 &&
    ungrantedOptionalEntries.length === 0
  ) {
    return (
      <div className='rounded-xl border border-border px-3 py-4 text-sm text-muted-foreground'>
        This app does not request any permissions.
      </div>
    );
  }

  return (
    <div className='space-y-5'>
      {requiredGroups.length > 0 ? (
        <PermissionSection
          title='Required permissions'
          subtitle='These are necessary for the app to function.'
          groups={requiredGroups}
          editable={editable}
          onToggleEntry={handleToggleEntry}
        />
      ) : null}

      {grantedOptionalGroups.length > 0 ? (
        <PermissionSection
          title='Granted optional permissions'
          subtitle='These optional permissions are currently enabled.'
          groups={grantedOptionalGroups}
          editable={editable}
          onToggleEntry={handleToggleEntry}
        />
      ) : null}

      {ungrantedOptionalGroups.length > 0 ? (
        <PermissionSection
          title='Optional permissions'
          subtitle='You can grant these now or keep them disabled.'
          groups={ungrantedOptionalGroups}
          editable={editable}
          collapsed={!showOptional}
          onToggleCollapsed={() => setShowOptional((prev) => !prev)}
          onToggleEntry={handleToggleEntry}
        />
      ) : null}
    </div>
  );
}
