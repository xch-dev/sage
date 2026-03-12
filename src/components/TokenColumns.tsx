import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { formatUsdPrice, getAssetDisplayName, toDecimal } from '@/lib/utils';
import { PricedTokenRecord } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef } from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  CircleAlertIcon,
  CoinsIcon,
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  MoreHorizontal,
  RefreshCw,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { AssetIcon } from './AssetIcon';
import { NumberFormat } from './NumberFormat';
import { Badge } from './ui/badge';
import { Tooltip, TooltipContent, TooltipTrigger } from './ui/tooltip';

export interface TokenActionHandlers {
  onRefreshInfo?: (assetId: string) => void;
  onToggleVisibility?: (asset: PricedTokenRecord) => void;
}

export const columns = (
  actionHandlers?: TokenActionHandlers,
): ColumnDef<PricedTokenRecord>[] => [
  {
    id: 'icon',
    enableSorting: false,
    header: () => <span className='sr-only'>{t`Token Icon`}</span>,
    size: 40,
    cell: ({ row }) => {
      return (
        <AssetIcon
          asset={{ ...row.original, kind: 'token' }}
          size='sm'
          className='ml-1'
        />
      );
    },
  },
  {
    accessorKey: 'name',
    header: () => <Trans>Name</Trans>,
    minSize: 120,
    cell: ({ row }) => {
      const name = getAssetDisplayName(
        row.original.name,
        row.original.ticker,
        'token',
      );
      const path = `/wallet/token/${row.original.asset_id ?? 'xch'}`;
      const ariaLabel = t`View ${name} token details`;

      return (
        <div className='inline-flex items-center gap-1'>
          <Link
            to={path}
            className={`hover:underline ${row.original.revocation_address || row.original.fee_policy ? 'text-yellow-600 dark:text-yellow-200' : ''}`}
            aria-label={ariaLabel}
          >
            {name}
          </Link>

          {row.original.revocation_address && (
            <Tooltip delayDuration={200}>
              <TooltipTrigger>
                <CircleAlertIcon className='h-4 w-4 text-yellow-600 dark:text-yellow-200' />
              </TooltipTrigger>
              <TooltipContent>
                <p>
                  <Trans>Asset can be revoked by the issuer</Trans>
                </p>
              </TooltipContent>
            </Tooltip>
          )}

          {row.original.fee_policy && (
            <Tooltip delayDuration={200}>
              <TooltipTrigger asChild>
                <Badge variant='secondary' className='text-[10px] px-1 py-0 h-4'>
                  <Trans>Fee CAT</Trans>
                </Badge>
              </TooltipTrigger>
              <TooltipContent>
                <p>
                  <Trans>
                    Fee policy: {row.original.fee_policy.fee_basis_points} bps
                  </Trans>
                </p>
              </TooltipContent>
            </Tooltip>
          )}
        </div>
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
          value={toDecimal(record.balance, record.precision)}
          maximumFractionDigits={record.precision}
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
        <span className='sr-only'>{t`USD Value: `}</span>
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
        <span className='sr-only'>{t`Price per token: `}</span>
        {formatUsdPrice(row.original.priceInUsd)}
      </div>
    ),
  },
  {
    id: 'actions',
    enableSorting: false,
    size: 44,
    meta: {
      cellClassName: 'px-2',
    },
    cell: ({ row }) => {
      const record = row.original;
      const balance = toDecimal(record.balance, record.precision);

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='ghost'
              className='h-6 w-6 p-0 flex items-center justify-center'
              aria-label={t`Open actions menu`}
            >
              <span className='sr-only'>{t`Open menu`}</span>
              <MoreHorizontal className='h-4 w-4' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end'>
            {record.asset_id !== null && (
              <>
                {actionHandlers?.onRefreshInfo && (
                  <DropdownMenuItem
                    onClick={() => {
                      if (!record.asset_id) return;
                      actionHandlers.onRefreshInfo?.(record.asset_id);
                    }}
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
                {record.asset_id && (
                  <DropdownMenuItem
                    onClick={() => {
                      openUrl(
                        `https://dexie.space/offers/XCH/${record.asset_id}`,
                      ).catch((error) => {
                        toast.error(t`Failed to open dexie.space: ${error}`);
                      });
                    }}
                  >
                    <ExternalLink className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>View on dexie</Trans>
                  </DropdownMenuItem>
                )}
                {record.asset_id && (
                  <DropdownMenuItem
                    onClick={() => {
                      if (!record.asset_id) return;
                      navigator.clipboard.writeText(record.asset_id);
                      toast.success(t`Asset ID copied to clipboard`);
                    }}
                  >
                    <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Copy Asset ID</Trans>
                  </DropdownMenuItem>
                )}
              </>
            )}
            <DropdownMenuItem
              onClick={() => {
                navigator.clipboard.writeText(balance.toString());
                toast.success(t`Balance copied to clipboard`);
              }}
            >
              <CoinsIcon className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy Balance</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
