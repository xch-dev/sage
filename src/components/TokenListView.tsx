import { DataTable } from '@/components/ui/data-table';
import { cn } from '@/lib/utils';
import { UITokenRecord, TokenViewProps } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { columns, TokenActionHandlers } from './TokenColumns';

type TokenListViewProps = TokenViewProps & {
  actionHandlers?: TokenActionHandlers;
};

export function TokenListView({
  cats,
  xchRecord,
  actionHandlers,
}: TokenListViewProps) {
  const tokens: UITokenRecord[] = [
    xchRecord,
    ...cats.map((cat) => ({
      asset_id: cat.asset_id,
      name: cat.name,
      ticker: cat.ticker,
      icon_url: cat.icon_url,
      balance: cat.balance,
      balanceInUsd: cat.balanceInUsd,
      priceInUsd: cat.priceInUsd,
      decimals: 3,
      isXch: false,
      visible: cat.visible,
    })),
  ];

  return (
    <div role='region' aria-label={t`Token List`}>
      <DataTable
        columns={columns(actionHandlers)}
        data={tokens}
        aria-label={t`Token list`}
        rowLabel={t`asset`}
        rowLabelPlural={t`assets`}
        getRowStyles={(row) => ({
          className: cn(
            !row.original.visible && !row.original.isXch && 'opacity-50',
          ),
        })}
      />
    </div>
  );
}
