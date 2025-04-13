'use client';

import { Button } from '@/components/ui/button';
import { DataTableColumnHeader } from '@/components/ui/data-table-column-header';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { formatAddress, formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef } from '@tanstack/react-table';
import { Copy, MoreHorizontal, Wallet } from 'lucide-react';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { AmountCell } from './AmountCell';

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
      <DataTableColumnHeader column={column} title={t`Transaction`} />
    ),
    enableSorting: false,
    size: 140,
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
          title={
            row.original?.timestamp ? row.getValue('transactionHeight') : ''
          }
        >
          {formatTimestamp(row.original?.timestamp, 'short', 'short') ||
            row.getValue('transactionHeight')}
        </Link>
      ) : null;
    },
  },
  {
    accessorKey: 'type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Asset`} />
    ),
    enableSorting: false,
    size: 140,
    cell: ({ row }) => {
      const displayName = row.original.displayName;

      return (
        <div className='flex items-center gap-3'>
          <div
            className='w-6 h-6 flex-shrink-0'
            role='img'
            aria-label={`${displayName} icon`}
          >
            {row.original.icon_url ? (
              <img
                src={row.original.icon_url}
                aria-hidden='true'
                loading='lazy'
              />
            ) : null}
          </div>

          <div className='truncate'>{row.original.displayName}</div>
        </div>
      );
    },
  },
  {
    accessorKey: 'amount',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Amount`} />
    ),
    enableSorting: false,
    size: 120,
    cell: ({ row }) => (
      <AmountCell amount={row.getValue('amount')} type={row.getValue('type')} />
    ),
  },
  {
    accessorKey: 'address',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Address`} />
    ),
    enableSorting: false,
    size: 130,
    meta: {
      className: 'hidden md:table-cell',
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
              <Trans>Copy Amount</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(txCoin.address ?? '');
                toast.success(t`Address copied to clipboard`);
              }}
            >
              <Wallet className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy Address</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
