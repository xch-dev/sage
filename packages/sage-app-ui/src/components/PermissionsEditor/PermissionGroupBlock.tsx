import { Globe, HardDrive, KeyRound, Radio, Shield } from 'lucide-react';
import type { PermissionEntry, PermissionGroupNode } from './types';
import { normalizeKey } from './utils';
import { PermissionRow } from './PermissionRow';

function groupIcon(node: PermissionGroupNode) {
  const normalized = normalizeKey(node.id);

  if (normalized === 'network') return <Globe className='h-4 w-4' />;

  if (normalized === 'storage.persistent_webview') {
    return <HardDrive className='h-4 w-4' />;
  }

  if (normalized.includes('secret') || normalized.includes('wallet')) {
    return <KeyRound className='h-4 w-4' />;
  }

  if (normalized.includes('send') || normalized.includes('submit')) {
    return <Radio className='h-4 w-4' />;
  }

  return <Shield className='h-4 w-4' />;
}

export function PermissionGroupBlock({
  node,
  editable,
  onToggleEntry,
}: {
  node: PermissionGroupNode;
  editable: boolean;
  onToggleEntry: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
  return (
    <div className='space-y-3 rounded-2xl border border-border p-4'>
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
