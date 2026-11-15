import { DataTable } from '@/components/ui/data-table';
import { useNetwork } from '@/hooks/useNetwork';
import { usePrices } from '@/hooks/usePrices';
import { cn } from '@/lib/utils';
import { TokenViewProps } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { columns, TokenActionHandlers } from './TokenColumns';

type TokenListViewProps = TokenViewProps & {
  actionHandlers?: TokenActionHandlers;
};

export function TokenListView({ tokens, actionHandlers }: TokenListViewProps) {
  const { isTestnet } = useNetwork();
  const { currency, pricesEnabled } = usePrices();

  return (
    <div role='region' aria-label={t`Token List`}>
      <DataTable
        columns={columns(
          actionHandlers,
          isTestnet,
          pricesEnabled ? currency : null,
        )}
        data={tokens}
        aria-label={t`Token list`}
        rowLabel={t`asset`}
        rowLabelPlural={t`assets`}
        getRowStyles={(row) => ({
          className: cn(
            !row.original.visible &&
              row.original.asset_id !== null &&
              'opacity-50',
          ),
        })}
      />
    </div>
  );
}
