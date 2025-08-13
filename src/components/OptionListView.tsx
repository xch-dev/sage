import { OptionRecord } from '@/bindings';
import { DataTable } from '@/components/ui/data-table';
import { OptionActionHandlers } from '@/hooks/useOptionActions';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { columns } from './OptionColumns';

interface OptionListViewProps {
  options: OptionRecord[];
  updateOptions: () => void;
  showHidden: boolean;
  actionHandlers?: OptionActionHandlers;
}

export function OptionListView({
  options,
  actionHandlers,
}: OptionListViewProps) {
  return (
    <div role='region' aria-label={t`Option List`}>
      <DataTable
        columns={columns(actionHandlers)}
        data={options}
        aria-label={t`Option list`}
        rowLabel={t`option`}
        rowLabelPlural={t`options`}
        getRowStyles={(row) => ({
          className: cn(
            !row.original.visible && 'opacity-50',
            row.original.created_height === null && 'pulsate-opacity',
          ),
        })}
      />
    </div>
  );
}
