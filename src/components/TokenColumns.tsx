import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { formatUsdPrice, toDecimal } from '@/lib/utils';
import { TokenRecord } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef } from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  Coins,
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  MoreHorizontal,
  RefreshCw,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { NumberFormat } from './NumberFormat';

// Add new interface for token action handlers
export interface TokenActionHandlers {
  onRefreshInfo?: (assetId: string) => void;
  onToggleVisibility?: (asset: TokenRecord) => void;
}

export const columns = (
  actionHandlers?: TokenActionHandlers,
): ColumnDef<TokenRecord>[] => [
  {
    id: 'icon',
    enableSorting: false,
    header: () => <span className='sr-only'>{t`Token Icon`}</span>,
    size: 40,
    cell: ({ row }) => {
      const record = row.original;
      const iconUrl = record.isXch
        ? 'https://icons.dexie.space/xch.webp'
        : record.icon_url;

      return iconUrl ? (
        <img
          alt={t`Token logo`}
          aria-hidden='true'
          className='h-6 w-6 ml-1'
          src={iconUrl}
        />
      ) : null;
    },
  },
  {
    accessorKey: 'name',
    header: () => <Trans>Name</Trans>,
    minSize: 120,
    cell: ({ row }) => {
      const record = row.original;
      const name = record.isXch
        ? 'Chia'
        : record.name || <Trans>Unknown CAT</Trans>;
      const path = record.isXch
        ? '/wallet/token/xch'
        : `/wallet/token/${record.asset_id}`;
      const ariaLabel = record.isXch
        ? t`View Chia token details`
        : t`View ${name} token details`;

      return (
        <Link to={path} className='hover:underline' aria-label={ariaLabel}>
          {name}
        </Link>
      );
    },
  },
  {
    accessorKey: 'ticker',
    header: () => <Trans>Symbol</Trans>,
    size: 80,
    meta: {
      className: 'hidden md:table-cell',
    },
    cell: ({ row }) => {
      return row.original.ticker;
    },
  },
  {
    accessorKey: 'balance',
    header: () => <Trans>Balance</Trans>,
    size: 100,
    cell: ({ row }) => {
      const record = row.original;
      return (
        <NumberFormat
          value={toDecimal(record.balance, record.decimals)}
          maximumFractionDigits={record.decimals}
        />
      );
    },
  },
  {
    accessorKey: 'balanceInUsd',
    header: () => <Trans>Value</Trans>,
    size: 90,
    meta: {
      className: 'hidden md:table-cell',
    },
    cell: ({ row }) => (
      <div>
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
    header: () => <Trans>Price</Trans>,
    size: 60,
    meta: {
      className: 'hidden md:table-cell',
    },
    cell: ({ row }) => (
      <div>
        <span className='sr-only'>Price per token: </span>
        {formatUsdPrice(row.original.priceInUsd)}
      </div>
    ),
  },
  {
    id: 'actions',
    enableSorting: false,
    size: 44,
    cell: ({ row }) => {
      const record = row.original;
      const balance = toDecimal(record.balance, record.decimals);

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
            {!record.isXch && (
              <>
                {actionHandlers?.onRefreshInfo && (
                  <DropdownMenuItem
                    onClick={() =>
                      actionHandlers.onRefreshInfo?.(record.asset_id)
                    }
                  >
                    <RefreshCw className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Refresh Info</Trans>
                  </DropdownMenuItem>
                )}
                {actionHandlers?.onToggleVisibility && (
                  <DropdownMenuItem
                    onClick={() => actionHandlers.onToggleVisibility?.(record)}
                  >
                    {record.visible ? (
                      <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                    ) : (
                      <Eye className='mr-2 h-4 w-4' aria-hidden='true' />
                    )}
                    {record.visible ? <Trans>Hide</Trans> : <Trans>Show</Trans>}{' '}
                    <Trans>Asset</Trans>
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem
                  onClick={() => {
                    openUrl(
                      `https://dexie.space/offers/XCH/${record.asset_id}`,
                    ).catch((error) => {
                      console.error('Failed to open dexie.space:', error);
                      toast.error(t`Failed to open dexie.space`);
                    });
                  }}
                >
                  <ExternalLink className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>View on dexie</Trans>
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => {
                    navigator.clipboard.writeText(record.asset_id);
                    toast.success(t`Asset ID copied to clipboard`);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Copy Asset ID</Trans>
                </DropdownMenuItem>
              </>
            )}
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(balance.toString());
                toast.success(t`Balance copied to clipboard`);
              }}
            >
              <Coins className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy Balance</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
