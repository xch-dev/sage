import { ColumnDef } from '@tanstack/react-table';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';
import { NumberFormat } from './NumberFormat';
import { toDecimal, formatUsdPrice } from '@/lib/utils';

export interface TokenRecord {
  asset_id: string;
  name: string | null;
  ticker: string | null;
  icon_url: string | null;
  balance: number | string;
  balanceInUsd: number;
  priceInUsd: number;
  decimals: number;
  isXch?: boolean;
  visible?: boolean;
}

export const columns: ColumnDef<TokenRecord>[] = [
  {
    id: 'icon',
    header: () => <span className='sr-only'>{t`Token Icon`}</span>,
    cell: ({ row }) => {
      const record = row.original;
      const iconUrl = record.isXch ? 'https://icons.dexie.space/xch.webp' : record.icon_url;
      
      return iconUrl ? (
        <div className='w-[40px] min-w-[40px]'>
          <img
            alt={t`Token logo`}
            aria-hidden='true'
            className='h-6 w-6'
            src={iconUrl}
          />
        </div>
      ) : null;
    },
  },
  {
    accessorKey: 'name',
    header: () => <Trans>Name</Trans>,
    cell: ({ row }) => {
      const record = row.original;
      const name = record.isXch ? 'Chia' : record.name || <Trans>Unknown CAT</Trans>;
      const path = record.isXch ? '/wallet/token/xch' : `/wallet/token/${record.asset_id}`;
      const ariaLabel = record.isXch
        ? t`View Chia token details`
        : record.name
        ? t`View ${record.name} token details`
        : t`View Unknown CAT token details`;

      return (
        <div className='max-w-[120px] truncate'>
          <Link to={path} className='hover:underline' aria-label={ariaLabel}>
            {name}
          </Link>
        </div>
      );
    },
  },
  {
    accessorKey: 'ticker',
    header: () => (
      <div className='hidden sm:block'>
        <Trans>Symbol</Trans>
      </div>
    ),
    cell: ({ row }) => {
      const record = row.original;
      const ticker = record.isXch ? 'XCH' : record.ticker || '-';
      return <div className='hidden sm:block'>{ticker}</div>;
    },
  },
  {
    accessorKey: 'balance',
    header: () => (
      <div className='text-right'>
        <Trans>Balance</Trans>
      </div>
    ),
    cell: ({ row }) => {
      const record = row.original;
      return (
        <div className='text-right'>
          <NumberFormat
            value={toDecimal(record.balance, record.decimals)}
            maximumFractionDigits={record.decimals}
          />
        </div>
      );
    },
  },
  {
    accessorKey: 'balanceInUsd',
    header: () => (
      <div className='text-right hidden md:block'>
        <Trans>Balance (USD)</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className='text-right hidden md:block'>
        <span className='sr-only'>USD Value: </span>
        <NumberFormat
          value={row.original.balanceInUsd}
          style='currency'
          currency='USD'
          maximumFractionDigits={2}
        />
      </div>
    ),
  },
  {
    accessorKey: 'priceInUsd',
    header: () => (
      <div className='text-right hidden md:block'>
        <Trans>Price (USD)</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className='text-right hidden md:block'>
        <span className='sr-only'>Price per token: </span>
        {formatUsdPrice(row.original.priceInUsd)}
      </div>
    ),
  },
]; 