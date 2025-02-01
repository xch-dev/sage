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
    cell: ({ row }) => (
      <Link
        to={`/transactions/${row.getValue('transactionHeight')}`}
        className='hover:underline'
        onClick={(e) => e.stopPropagation()}
      >
        #{row.getValue('transactionHeight')}
      </Link>
    ),
  },
  {
    accessorKey: 'type',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Ticker`} />
    ),
    enableSorting: true,
    cell: ({ row }) => {
      const type = row.getValue('type') as string;
      return type === 'xch'
        ? 'XCH'
        : type === 'cat'
          ? (row.original.ticker ?? 'CAT')
          : type.toUpperCase();
    },
  },
  {
    accessorKey: 'amount',
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title={t`Amount`} />
    ),
    enableSorting: true,
    cell: ({ row }) => {
      const amount = row.getValue('amount') as string;
      const isPositive = amount.startsWith('+');
      return (
        <div className='text-right w-[80px]'>
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
      <DataTableColumnHeader column={column} title={t`Address`} />
    ),
    enableSorting: true,
    cell: ({ row }) => (
      <div className='font-mono text-sm'>
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
              className='h-8 w-8 p-0'
              aria-label='Open actions menu'
            >
              <span className='sr-only'>Open menu</span>
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
