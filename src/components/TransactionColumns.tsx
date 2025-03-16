'use client';

import { ColumnDef } from '@tanstack/react-table';
import { DataTableColumnHeader } from '@/components/ui/data-table-column-header';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';
import { MoreHorizontal, Copy, Wallet } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { toast } from 'react-toastify';
import { AmountCell } from './AmountCell';
import { formatAddress, formatTimestamp } from '@/lib/utils';

export interface FlattenedTransaction {
  transactionHeight: number;
  type: string;
  ticker?: string | null;
  icon_url?: string | null;
  amount: string;
  address: string | null;
  coin_id: string;
  displayName: string;
  timestamp: number | null;
}

export const columns: ColumnDef<FlattenedTransaction>[] = [
  {
    accessorKey: 'transactionHeight',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Block #`} />
    ),
    enableSorting: false,
    size: 80,
    meta: {
      className: 'w-[55px] min-w-[55px] md:w-[80px] md:min-w-[80px]',
    },
    cell: ({ row, table }) => {
      // Get all rows data
      const rows = table.options.data as FlattenedTransaction[];

      // Check if this is the first row for this transaction height
      const isFirstInGroup =
        rows.findIndex(
          (tx) => tx.transactionHeight === row.original.transactionHeight,
        ) === rows.indexOf(row.original);

      // Only show block number for first row in group
      return isFirstInGroup ? (
        <Link
          to={`/transactions/${row.getValue('transactionHeight')}`}
          className='hover:underline'
          onClick={(e) => e.stopPropagation()}
        >
          {row.getValue('transactionHeight')}
        </Link>
      ) : null;
    },
  },
  {
    accessorKey: 'timestamp',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Date`} />
    ),
    enableSorting: false,
    size: 160,
    meta: {
      className: 'hidden md:table-cell w-[160px] min-w-[160px]',
    },
    cell: ({ row, table }) => {
      // Get all rows data
      const rows = table.options.data as FlattenedTransaction[];

      // Check if this is the first row for this transaction height
      const isFirstInGroup =
        rows.findIndex(
          (tx) => tx.transactionHeight === row.original.transactionHeight,
        ) === rows.indexOf(row.original);

      // Only show timestamp for first row in group
      return isFirstInGroup ? (
        <div className='hidden md:block'>
          {formatTimestamp(row.getValue('timestamp'), 'short', 'short')}
        </div>
      ) : null;
    },
  },
  {
    id: 'icon',
    enableSorting: false,
    size: 48,
    meta: {
      className: 'w-[40px] min-w-[40px] md:w-[48px] md:min-w-[48px]',
    },
    header: () => <span className='sr-only'>{t`Asset Icon`}</span>,
    cell: ({ row }) => {
      const type = row.getValue('type') as string;
      const ticker = row.original.ticker;
      const assetName = type === 'xch' ? 'XCH' : (ticker ?? 'CAT');

      return (
        <div className='w-6 h-6' role='img' aria-label={`${assetName} icon`}>
          {row.original.icon_url ? (
            <img
              src={row.original.icon_url}
              aria-hidden='true'
              loading='lazy'
            />
          ) : null}
        </div>
      );
    },
  },
  {
    accessorKey: 'type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Ticker`} />
    ),
    enableSorting: false,
    size: 120,
    meta: {
      className: 'w-[70px] min-w-[70px] md:w-[120px] md:min-w-[120px]',
    },
    cell: ({ row }) => {
      return <div className='truncate'>{row.original.displayName}</div>;
    },
  },
  {
    accessorKey: 'amount',
    header: ({ column }) => (
      <div className='text-right'>
        <DataTableColumnHeader column={column} title={t`Amount`} />
      </div>
    ),
    enableSorting: false,
    size: 120,
    meta: {
      className: 'w-[85px] min-w-[85px] md:w-[120px] md:min-w-[120px]',
    },
    cell: ({ row }) => (
      <AmountCell amount={row.getValue('amount')} type={row.getValue('type')} />
    ),
  },
  {
    accessorKey: 'address',
    header: ({ column }) => (
      <div className='hidden md:block'>
        <DataTableColumnHeader column={column} title={t`Address`} />
      </div>
    ),
    enableSorting: false,
    size: 200,
    meta: {
      className: 'hidden md:table-cell w-full md:w-[200px] md:min-w-[200px]',
    },
    cell: ({ row }) => (
      <div className='hidden md:block font-mono'>
        {formatAddress(row.getValue<string | null>('address') || '', 7, 4)}
      </div>
    ),
  },
  {
    id: 'actions',
    enableSorting: false,
    size: 50,
    meta: {
      className: 'w-[40px] min-w-[40px] md:w-[50px] md:min-w-[50px]',
    },
    cell: ({ row }) => {
      const txCoin = row.original;

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='ghost'
              className='h-6 w-6 p-0'
              aria-label={t`Open actions menu`}
            >
              <span className='sr-only'>{t`Open menu`}</span>
              <MoreHorizontal className='h-4 w-4' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end'>
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(txCoin.amount ?? '');
                toast.success(t`Amount copied to clipboard`);
              }}
            >
              <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy amount</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(txCoin.address ?? '');
                toast.success(t`Address copied to clipboard`);
              }}
            >
              <Wallet className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy address</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
