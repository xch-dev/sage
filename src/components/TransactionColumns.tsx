'use client';

import { ColumnDef } from '@tanstack/react-table';
import { DataTableColumnHeader } from '@/components/ui/data-table-column-header';
import { t } from '@lingui/core/macro';
import { NumberFormat } from './NumberFormat';
import { fromMojos } from '@/lib/utils';
import { Link } from 'react-router-dom';
import { MoreHorizontal } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { toast } from 'react-toastify';

export interface FlattenedTransaction {
  transactionHeight: number;
  type: string;
  ticker?: string | null;
  icon_url?: string | null;
  amount: string;
  address: string | null;
  coin_id: string;
}

export const columns: ColumnDef<FlattenedTransaction>[] = [
  {
    accessorKey: 'transactionHeight',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Block #`} />
    ),
    enableSorting: true,
    sortingFn: 'basic',
    sortDescFirst: true,
    sortUndefined: 1,
    meta: {
      serverSort: true,
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
          className='hover:underline text-sm md:text-base'
          onClick={(e) => e.stopPropagation()}
        >
          {row.getValue('transactionHeight')}
        </Link>
      ) : null;
    },
  },
  {
    id: 'icon',
    enableSorting: false,
    header: () => null,
    cell: ({ row }) => {
      const type = row.getValue('type') as string;
      const ticker = row.original.ticker;

      return (
        <div className='w-4 h-4 md:w-6 md:h-6'>
          {type === 'xch' ? (
            <img
              alt='XCH'
              src='https://icons.dexie.space/xch.webp'
              aria-hidden='true'
            />
          ) : type === 'cat' && row.original.icon_url ? (
            <img
              alt={ticker ?? 'CAT'}
              src={row.original.icon_url}
              aria-hidden='true'
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
    cell: ({ row }) => {
      const type = row.getValue('type') as string;
      return (
        <div className='text-sm md:text-base'>
          {type === 'xch'
            ? 'XCH'
            : type === 'cat'
              ? (row.original.ticker ?? 'CAT')
              : type.toUpperCase()}
        </div>
      );
    },
  },
  {
    accessorKey: 'amount',
    header: ({ column }) => (
      <div className='w-[120px] md:w-[150px] text-right'>
        <DataTableColumnHeader column={column} title={t`Amount`} />
      </div>
    ),
    enableSorting: false,
    cell: ({ row }) => {
      const amount = row.getValue('amount') as string;
      const isPositive = amount.startsWith('+');
      return (
        <div className='w-[120px] md:w-[150px] text-right text-sm md:text-base'>
          <span className={isPositive ? 'text-green-600' : 'text-red-600'}>
            <NumberFormat
              value={fromMojos(amount, 12)}
              minimumFractionDigits={0}
              maximumFractionDigits={12}
            />
          </span>
        </div>
      );
    },
  },
  {
    accessorKey: 'address',
    header: ({ column }) => (
      <div className='hidden md:block'>
        <DataTableColumnHeader column={column} title={t`Address`} />
      </div>
    ),
    enableSorting: false,
    cell: ({ row }) => (
      <div className='hidden md:block font-mono text-sm'>
        {row.getValue<string | null>('address')?.slice(0, 15)}...
      </div>
    ),
  },
  {
    id: 'actions',
    enableSorting: false,
    cell: ({ row }) => {
      const txCoin = row.original;

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='ghost'
              className='h-6 w-6 p-0'
              aria-label='Open actions menu'
            >
              <span className='sr-only'>Open menu</span>
              <MoreHorizontal
                className='h-3 w-3 md:h-4 md:w-4'
                aria-hidden='true'
              />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end'>
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(txCoin.amount ?? '');
                toast.success(t`Amount copied to clipboard`);
              }}
            >
              Copy amount
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(txCoin.address ?? '');
                toast.success(t`Address copied to clipboard`);
              }}
            >
              Copy address
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
