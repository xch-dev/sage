import { ChevronDown, ChevronRight } from 'lucide-react';
import type { PermissionEntry, PermissionGroupNode } from './types';
import { countNodeEntries } from './permissionTree';
import { PermissionGroupBlock } from './PermissionGroupBlock';

export function PermissionSection({
  title,
  groups,
  editable,
  separated,
  collapsed,
  onToggleCollapsed,
  onToggleEntry,
}: {
  title: string;
  groups: PermissionGroupNode[];
  editable: boolean;
  separated?: boolean;
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
    <section className='space-y-2.5'>
      {separated ? <div className='border-t border-border/70' /> : null}

      <div className='flex items-center justify-between gap-3 px-2'>
        <h3 className='text-base font-semibold tracking-tight'>{title}</h3>

        {onToggleCollapsed ? (
          <button
            type='button'
            className='inline-flex h-7 items-center gap-1 rounded-md px-2 text-xs text-muted-foreground hover:bg-muted'
            onClick={onToggleCollapsed}
          >
            {contentHidden ? (
              <ChevronRight className='h-3.5 w-3.5' />
            ) : (
              <ChevronDown className='h-3.5 w-3.5' />
            )}
            {itemCount}
          </button>
        ) : null}
      </div>

      {!contentHidden ? (
        <div className='space-y-2 px-4'>
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
    </section>
  );
}
