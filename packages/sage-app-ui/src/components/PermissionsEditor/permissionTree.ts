import type { PermissionEntry, PermissionGroupNode } from './types';
import { sortPermissionEntries } from './permissionEntries';
import { normalizeKey, segmentLabel } from './utils';

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

export function buildGroupedPermissionTree(
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

  const storageEntries = entries.filter(
    (entry) =>
      entry.kind === 'capability' &&
      normalizeKey(entry.key).startsWith('storage'),
  );

  if (storageEntries.length > 0) {
    const storageNode = makeNode('storage', 'Storage');
    storageNode.entries = sortPermissionEntries(storageEntries);
    storageNode.sensitivityRank = 2;
    roots.push(storageNode);
  }

  const generalCapabilityEntries = entries.filter(
    (entry) =>
      entry.kind === 'capability' &&
      normalizeKey(entry.key) !== 'storage.persistent_webview',
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

export function countNodeEntries(node: PermissionGroupNode): number {
  return (
    node.entries.length +
    node.children.reduce((sum, child) => sum + countNodeEntries(child), 0)
  );
}
