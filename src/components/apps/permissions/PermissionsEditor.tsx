import { useEffect, useMemo, useState } from 'react';
import type {
  SageAppCapabilityDefinitionView,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  SystemSageAppView,
  UserBridgeCapability,
  UserSageAppView,
} from '@/bindings';
import { commands } from '@/bindings';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Globe,
  HardDrive,
  KeyRound,
  Radio,
  Shield,
} from 'lucide-react';

interface Props {
  app: UserSageAppView | SystemSageAppView;
  grantedPermissions: SageGrantedPermissionsView;
  onGrantedPermissionsChange?: (next: SageGrantedPermissionsInput) => void;
  editable?: boolean;
}

type PermissionEntry =
  | {
      id: string;
      kind: 'capability';
      key: string;
      capability: UserBridgeCapability;
      label: string;
      description: string | null;
      required: boolean;
      granted: boolean;
      sensitivityRank: number;
    }
  | {
      id: string;
      kind: 'network';
      key: string;
      label: string;
      description: string | null;
      required: boolean;
      granted: boolean;
      sensitivityRank: number;
    };

interface PermissionGroupNode {
  id: string;
  label: string;
  children: PermissionGroupNode[];
  entries: PermissionEntry[];
  sensitivityRank: number;
}

function networkKey(entry: SageNetworkWhitelistEntry): string {
  return `${entry.scheme}://${entry.host}`;
}

function sortNetworkEntries(
  entries: SageNetworkWhitelistEntry[],
): SageNetworkWhitelistEntry[] {
  return [...entries].sort((a, b) =>
    networkKey(a).localeCompare(networkKey(b)),
  );
}

function titleCasePart(value: string): string {
  if (!value) return value;
  return value.charAt(0).toUpperCase() + value.slice(1);
}

function segmentLabel(segment: string): string {
  return segment.split('_').filter(Boolean).map(titleCasePart).join(' ');
}

function formatCapabilityLeafLabel(key: string): string {
  const parts = key.split('.');
  return segmentLabel(parts[parts.length - 1] ?? key);
}

function normalizeKey(key: string): string {
  return key.trim().toLowerCase();
}

function capabilitySensitivityRank(key: string): number {
  if (key.includes('secret')) return 0;
  if (key === 'persistent_storage') return 2;
  if (key.includes('send') || key.includes('network')) return 3;
  return 4;
}

function capabilityDefinitionMap(
  definitions: SageAppCapabilityDefinitionView[],
): Map<UserBridgeCapability, SageAppCapabilityDefinitionView> {
  return new Map(
    definitions.map((definition) => [
      definition.key as UserBridgeCapability,
      definition,
    ]),
  );
}

function isUserGrantableCapability(
  capability: UserBridgeCapability,
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
): boolean {
  return definitionsByKey.get(capability)?.flags.userGrantable === true;
}

function buildCapabilityEntries(
  requestedRequired: UserBridgeCapability[],
  requestedOptional: UserBridgeCapability[],
  grantedCapabilities: UserBridgeCapability[],
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
): PermissionEntry[] {
  const grantedSet = new Set<UserBridgeCapability>(grantedCapabilities);

  const requiredEntries: PermissionEntry[] = requestedRequired
    .filter((capability) =>
      isUserGrantableCapability(capability, definitionsByKey),
    )
    .map((capability) => {
      const definition = definitionsByKey.get(capability);
      const key = capability;

      return {
        id: `capability:${key}`,
        kind: 'capability',
        key,
        capability,
        label: definition?.label ?? formatCapabilityLeafLabel(key),
        description: definition?.description ?? null,
        required: true,
        granted: true,
        sensitivityRank: capabilitySensitivityRank(key),
      };
    });

  const optionalEntries: PermissionEntry[] = requestedOptional
    .filter((capability) =>
      isUserGrantableCapability(capability, definitionsByKey),
    )
    .map((capability) => {
      const definition = definitionsByKey.get(capability);
      const key = capability;

      return {
        id: `capability:${key}`,
        kind: 'capability',
        key,
        capability,
        label: definition?.label ?? formatCapabilityLeafLabel(key),
        description: definition?.description ?? null,
        required: false,
        granted: grantedSet.has(capability),
        sensitivityRank: capabilitySensitivityRank(key),
      };
    });

  return [...requiredEntries, ...optionalEntries];
}

function buildNetworkEntries(
  requestedRequired: SageNetworkWhitelistEntry[],
  requestedOptional: SageNetworkWhitelistEntry[],
  grantedNetworkWhitelist: SageNetworkWhitelistEntry[],
): PermissionEntry[] {
  const grantedSet = new Set(
    grantedNetworkWhitelist.map((entry) => networkKey(entry)),
  );

  const requiredEntries: PermissionEntry[] = requestedRequired.map((entry) => {
    const key = networkKey(entry);

    return {
      id: `network:${key}`,
      kind: 'network',
      key,
      label: key,
      description: null,
      required: true,
      granted: true,
      sensitivityRank: 1,
    };
  });

  const optionalEntries: PermissionEntry[] = requestedOptional.map((entry) => {
    const key = networkKey(entry);

    return {
      id: `network:${key}`,
      kind: 'network',
      key,
      label: key,
      description: null,
      required: false,
      granted: grantedSet.has(key),
      sensitivityRank: 1,
    };
  });

  return [...requiredEntries, ...optionalEntries];
}

function sortPermissionEntries(entries: PermissionEntry[]): PermissionEntry[] {
  return [...entries].sort((a, b) => {
    if (a.sensitivityRank !== b.sensitivityRank) {
      return a.sensitivityRank - b.sensitivityRank;
    }

    if (a.kind !== b.kind) {
      return a.kind.localeCompare(b.kind);
    }

    return a.key.localeCompare(b.key);
  });
}

function groupIcon(node: PermissionGroupNode) {
  const normalized = normalizeKey(node.id);

  if (normalized === 'network') return <Globe className='h-4 w-4' />;
  if (normalized === 'persistent_storage')
    return <HardDrive className='h-4 w-4' />;
  if (normalized.includes('secret') || normalized.includes('wallet')) {
    return <KeyRound className='h-4 w-4' />;
  }
  if (normalized.includes('send') || normalized.includes('submit')) {
    return <Radio className='h-4 w-4' />;
  }

  return <Shield className='h-4 w-4' />;
}

function makeNode(id: string, label: string): PermissionGroupNode {
  return {
    id,
    label,
    children: [],
    entries: [],
    sensitivityRank: 999,
  };
}

function updateNodeSensitivity(
  node: PermissionGroupNode,
  rank: number,
): PermissionGroupNode {
  node.sensitivityRank = Math.min(node.sensitivityRank, rank);
  return node;
}

function buildGroupedPermissionTree(
  entries: PermissionEntry[],
): PermissionGroupNode[] {
  const roots: PermissionGroupNode[] = [];

  const networkEntries = entries.filter((entry) => entry.kind === 'network');
  if (networkEntries.length > 0) {
    const networkNode = makeNode('network', 'Network access');
    networkNode.entries = sortPermissionEntries(networkEntries);
    networkNode.sensitivityRank = 1;
    roots.push(networkNode);
  }

  const persistentEntries = entries.filter(
    (entry) =>
      entry.kind === 'capability' &&
      normalizeKey(entry.key) === 'persistent_storage',
  );
  if (persistentEntries.length > 0) {
    const persistentNode = makeNode('persistent_storage', 'Persistent storage');
    persistentNode.entries = sortPermissionEntries(persistentEntries);
    persistentNode.sensitivityRank = 2;
    roots.push(persistentNode);
  }

  const generalCapabilityEntries = entries.filter(
    (entry) =>
      entry.kind === 'capability' &&
      normalizeKey(entry.key) !== 'persistent_storage',
  );

  const capabilityRoot = makeNode('capabilities_root', 'Capabilities');

  for (const entry of generalCapabilityEntries) {
    const parts = entry.key.split('.');
    const leafParentParts = parts.slice(0, -1);

    if (leafParentParts.length === 0) {
      capabilityRoot.entries.push(entry);
      updateNodeSensitivity(capabilityRoot, entry.sensitivityRank);
      continue;
    }

    let current = capabilityRoot;
    updateNodeSensitivity(current, entry.sensitivityRank);

    for (let index = 0; index < leafParentParts.length; index += 1) {
      const segment = leafParentParts[index];
      const fullPath = leafParentParts.slice(0, index + 1).join('.');
      let child = current.children.find((node) => node.id === fullPath);

      if (!child) {
        child = makeNode(fullPath, segmentLabel(segment));
        current.children.push(child);
      }

      updateNodeSensitivity(child, entry.sensitivityRank);
      current = child;
    }

    current.entries.push(entry);
    updateNodeSensitivity(current, entry.sensitivityRank);
  }

  function sortNode(node: PermissionGroupNode) {
    node.entries = sortPermissionEntries(node.entries);

    node.children.sort((a, b) => {
      if (a.sensitivityRank !== b.sensitivityRank) {
        return a.sensitivityRank - b.sensitivityRank;
      }

      return a.label.localeCompare(b.label);
    });

    for (const child of node.children) {
      sortNode(child);
    }
  }

  sortNode(capabilityRoot);

  if (capabilityRoot.entries.length > 0 || capabilityRoot.children.length > 0) {
    roots.push(...capabilityRoot.children);

    if (capabilityRoot.entries.length > 0) {
      const miscNode = makeNode('misc', 'Other capabilities');
      miscNode.entries = capabilityRoot.entries;
      miscNode.sensitivityRank = capabilityRoot.sensitivityRank;
      roots.push(miscNode);
    }
  }

  roots.sort((a, b) => {
    if (a.sensitivityRank !== b.sensitivityRank) {
      return a.sensitivityRank - b.sensitivityRank;
    }

    return a.label.localeCompare(b.label);
  });

  return roots;
}

function CapabilityHelp({ description }: { description: string | null }) {
  if (!description) return null;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type='button'
          className='shrink-0 rounded-sm p-0.5 text-muted-foreground transition-colors hover:text-foreground'
          onClick={(event) => event.preventDefault()}
          aria-label='Permission details'
        >
          <CircleHelp className='h-4 w-4' />
        </button>
      </TooltipTrigger>
      <TooltipContent className='max-w-xs text-left'>
        {description}
      </TooltipContent>
    </Tooltip>
  );
}

function PermissionRow({
  entry,
  editable,
  onToggle,
}: {
  entry: PermissionEntry;
  editable: boolean;
  onToggle: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
  return (
    <label className='flex items-start gap-3 rounded-xl border px-3 py-3 text-sm'>
      <Checkbox
        checked={entry.granted}
        disabled={!editable || entry.required}
        onCheckedChange={(checked) => {
          onToggle(entry, Boolean(checked));
        }}
        className='mt-0.5'
      />

      <div className='min-w-0 flex-1'>
        <div className='flex items-center gap-2'>
          <div
            className={
              entry.kind === 'network'
                ? 'min-w-0 flex-1 truncate font-mono text-sm'
                : 'min-w-0 flex-1 truncate font-medium'
            }
          >
            {entry.label}
          </div>

          {entry.kind === 'capability' ? (
            <CapabilityHelp description={entry.description} />
          ) : null}

          {entry.required ? (
            <span className='rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground'>
              Required
            </span>
          ) : null}
        </div>
      </div>
    </label>
  );
}

function PermissionGroupBlock({
  node,
  editable,
  onToggleEntry,
}: {
  node: PermissionGroupNode;
  editable: boolean;
  onToggleEntry: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
  return (
    <div className='space-y-3 rounded-2xl border p-4'>
      <div className='flex items-center gap-2'>
        <div className='text-muted-foreground'>{groupIcon(node)}</div>
        <div className='text-sm font-medium'>{node.label}</div>
      </div>

      {node.entries.length > 0 ? (
        <div className='space-y-2'>
          {node.entries.map((entry) => (
            <PermissionRow
              key={entry.id}
              entry={entry}
              editable={editable}
              onToggle={onToggleEntry}
            />
          ))}
        </div>
      ) : null}

      {node.children.length > 0 ? (
        <div className='space-y-3'>
          {node.children.map((child) => (
            <PermissionGroupBlock
              key={child.id}
              node={child}
              editable={editable}
              onToggleEntry={onToggleEntry}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function countNodeEntries(node: PermissionGroupNode): number {
  return (
    node.entries.length +
    node.children.reduce((sum, child) => sum + countNodeEntries(child), 0)
  );
}

function PermissionSection({
  title,
  subtitle,
  groups,
  editable,
  collapsed,
  onToggleCollapsed,
  onToggleEntry,
}: {
  title: string;
  subtitle?: string;
  groups: PermissionGroupNode[];
  editable: boolean;
  collapsed?: boolean;
  onToggleCollapsed?: () => void;
  onToggleEntry: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
  if (groups.length === 0) return null;

  const itemCount = groups.reduce(
    (count, group) => count + countNodeEntries(group),
    0,
  );
  const contentHidden = Boolean(collapsed);

  return (
    <div className='space-y-3'>
      <div className='flex items-center justify-between gap-3'>
        <div>
          <h3 className='text-sm font-medium'>{title}</h3>
          {subtitle ? (
            <div className='mt-1 text-xs text-muted-foreground'>{subtitle}</div>
          ) : null}
        </div>

        {onToggleCollapsed ? (
          <Button
            type='button'
            variant='ghost'
            size='sm'
            className='h-8 gap-1 px-2 text-muted-foreground'
            onClick={onToggleCollapsed}
          >
            {contentHidden ? (
              <ChevronRight className='h-4 w-4' />
            ) : (
              <ChevronDown className='h-4 w-4' />
            )}
            {itemCount}
          </Button>
        ) : null}
      </div>

      {!contentHidden ? (
        <div className='space-y-3'>
          {groups.map((group) => (
            <PermissionGroupBlock
              key={group.id}
              node={group}
              editable={editable}
              onToggleEntry={onToggleEntry}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function PermissionsEditor({
  app,
  grantedPermissions,
  onGrantedPermissionsChange,
  editable = true,
}: Props) {
  const manifest =
    'pendingUpdate' in app && app.pendingUpdate
      ? app.pendingUpdate.manifest
      : app.common.activeSnapshot.manifest;

  const [showOptional, setShowOptional] = useState(false);
  const [capabilityDefinitions, setCapabilityDefinitions] = useState<
    SageAppCapabilityDefinitionView[]
  >([]);

  useEffect(() => {
    let cancelled = false;

    void commands
      .getUserCapabilityDefinitions()
      .then((definitions) => {
        if (!cancelled) {
          setCapabilityDefinitions(definitions);
        }
      })
      .catch((err) => {
        console.error('Failed to load capability definitions:', err);
      });

    return () => {
      cancelled = true;
    };
  }, []);

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
          whitelist: grantedNetworkWhitelist
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
        ])
      },
    });
  }

  if (
    requiredEntries.length === 0 &&
    grantedOptionalEntries.length === 0 &&
    ungrantedOptionalEntries.length === 0
  ) {
    return (
      <div className='rounded-xl border px-3 py-4 text-sm text-muted-foreground'>
        This app does not request any permissions.
      </div>
    );
  }

  return (
    <TooltipProvider delayDuration={0} skipDelayDuration={0}>
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
    </TooltipProvider>
  );
}
