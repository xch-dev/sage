import { DataTable } from '@/components/ui/data-table';
import { TokenViewProps } from '@/types/TokenViewProps';
import { columns, TokenRecord } from './TokenColumns';
import { t } from '@lingui/core/macro';
import { cn } from '@/lib/utils';

type TokenListViewProps = TokenViewProps;

export function TokenListView({
  cats,
  xchBalance,
  xchDecimals,
  xchBalanceUsd,
  xchPrice,
}: TokenListViewProps) {
  const tokens: TokenRecord[] = [
    {
      asset_id: 'xch',
      name: 'Chia',
      ticker: 'XCH',
      icon_url: 'https://icons.dexie.space/xch.webp',
      balance: xchBalance,
      balanceInUsd: xchBalanceUsd,
      priceInUsd: xchPrice,
      decimals: xchDecimals,
      isXch: true,
    },
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
        columns={columns}
        data={tokens}
        aria-label={t`Token list`}
        getRowStyles={(row) => ({
          className: cn(
            !row.original.visible && !row.original.isXch && 'opacity-50'
          ),
        })}
      />
    </div>
  );
}
