import { Button } from '@/components/ui/button';
import { t } from '@lingui/core/macro';
import { LayoutGrid, List as ListIcon } from 'lucide-react';

type ViewMode = 'grid' | 'list';

interface ViewToggleProps {
  view: ViewMode;
  onChange: (view: ViewMode) => void;
}

export function ViewToggle({ view, onChange }: ViewToggleProps) {
  return (
    <Button
      variant='outline'
      size='icon'
      onClick={() => onChange(view === 'grid' ? 'list' : 'grid')}
      title={view === 'grid' ? t`Switch to list view` : t`Switch to grid view`}
    >
      {view === 'grid' ? (
        <ListIcon className='h-4 w-4' aria-hidden='true' />
      ) : (
        <LayoutGrid className='h-4 w-4' aria-hidden='true' />
      )}
    </Button>
  );
}

export type { ViewMode };
