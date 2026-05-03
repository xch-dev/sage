import type { ReactNode } from 'react';
import type { PermissionEntry, PermissionGroupNode } from './types';
import { PermissionGroupBlock } from './PermissionGroupBlock';

export function PermissionSection({
  title,
  groups,
  editable,
  separated,
  trailingAction,
  onToggleEntry,
}: {
  title: string;
  groups: PermissionGroupNode[];
  editable: boolean;
  separated?: boolean;
  trailingAction?: ReactNode;
  onToggleEntry: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
  if (groups.length === 0 && !trailingAction) return null;

  return (
    <section className='space-y-3'>
      {separated ? <div className='border-t border-border' /> : null}

      <div className='flex items-center justify-between gap-3'>
        <h3 className='text-base font-semibold tracking-tight'>{title}</h3>
      </div>

      {groups.length > 0 ? (
        <div className='space-y-2'>
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

      {trailingAction ? <div className='pt-1'>{trailingAction}</div> : null}
    </section>
  );
}
