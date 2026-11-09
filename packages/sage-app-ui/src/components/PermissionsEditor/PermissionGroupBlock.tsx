import { Globe, HardDrive, KeyRound, Radio, Shield } from 'lucide-react';
import type { PermissionEntry, PermissionGroupNode } from './types';
import { normalizeKey } from './utils';
import { PermissionRow } from './PermissionRow';

function groupIcon(node: PermissionGroupNode) {
  const normalized = normalizeKey(node.id);

  if (normalized === 'network') return <Globe className='h-3.5 w-3.5' />;

  if (normalized === 'storage.persistent_webview') {
    return <HardDrive className='h-3.5 w-3.5' />;
  }

  if (normalized.includes('secret') || normalized.includes('wallet')) {
    return <KeyRound className='h-3.5 w-3.5' />;
  }

  if (normalized.includes('send') || normalized.includes('submit')) {
    return <Radio className='h-3.5 w-3.5' />;
  }

  return <Shield className='h-3.5 w-3.5' />;
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
  const items = [
    ...node.entries.map((entry) => ({
      kind: 'entry' as const,
      id: entry.id,
      entry,
    })),
    ...node.children.map((child) => ({
      kind: 'child' as const,
      id: child.id,
      child,
    })),
  ];

  return (
    <div className='space-y-1.5'>
      <div className='flex items-center gap-2 py-1'>
        <div className='text-muted-foreground'>{groupIcon(node)}</div>
        <div className='text-sm font-medium'>{node.label}</div>
      </div>

      <div className='ml-2 space-y-1.5 pl-4'>
        {items.map((item, index) => {
          const isLast = index === items.length - 1;

          return (
            <div key={item.id} className='relative'>
              <div
                className={
                  isLast
                    ? 'absolute -left-4 top-0 h-1/2 border-l border-border/70'
                    : 'absolute -left-4 bottom-0 top-0 border-l border-border/70'
                }
              />
              <div className='absolute -left-4 top-1/2 h-px w-3 border-t border-border/70' />

              {item.kind === 'entry' ? (
                <PermissionRow
                  entry={item.entry}
                  editable={editable}
                  onToggle={onToggleEntry}
                />
              ) : (
                <PermissionGroupBlock
                  node={item.child}
                  editable={editable}
                  onToggleEntry={onToggleEntry}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
