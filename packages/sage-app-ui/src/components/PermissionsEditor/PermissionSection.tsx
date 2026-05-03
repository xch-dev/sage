import { ChevronDown, ChevronRight } from 'lucide-react';
import type { PermissionEntry, PermissionGroupNode } from './types';
import { countNodeEntries } from './permissionTree';
import { PermissionGroupBlock } from './PermissionGroupBlock';

export function PermissionSection({
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
          <button
            type='button'
            className='inline-flex h-8 items-center gap-1 rounded-md px-2 text-sm text-muted-foreground hover:bg-muted'
            onClick={onToggleCollapsed}
          >
            {contentHidden ? (
              <ChevronRight className='h-4 w-4' />
            ) : (
              <ChevronDown className='h-4 w-4' />
            )}
            {itemCount}
          </button>
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
