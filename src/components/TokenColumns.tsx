import { ColumnDef } from '@tanstack/react-table';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';
import { NumberFormat } from './NumberFormat';
import { toDecimal, formatUsdPrice } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  MoreHorizontal,
  RefreshCw,
  Eye,
  EyeOff,
  Copy,
  ExternalLink,
  Coins,
} from 'lucide-react';
import { toast } from 'react-toastify';
import { TokenRecord } from '@/types/TokenViewProps';
import { open } from '@tauri-apps/plugin-shell';

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
    header: () => <span className='sr-only'>{t`Token Icon`}</span>,
    cell: ({ row }) => {
      const record = row.original;
      const iconUrl = record.isXch
        ? 'https://icons.dexie.space/xch.webp'
        : record.icon_url;

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
      const name = record.isXch
        ? 'Chia'
        : record.name || <Trans>Unknown CAT</Trans>;
      const path = record.isXch
        ? '/wallet/token/xch'
        : `/wallet/token/${record.asset_id}`;
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
  {
    id: 'actions',
    enableSorting: false,
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
                    open(
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
