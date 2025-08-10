import { OptionRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef } from '@tanstack/react-table';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  AlertCircle,
  Copy,
  EyeIcon,
  EyeOff,
  FilePenLine,
  Flame,
  HandCoins,
  MoreHorizontal,
  SendIcon,
} from 'lucide-react';
import { toast } from 'react-toastify';
import { NumberFormat } from './NumberFormat';

// Add interface for option action handlers
export interface OptionActionHandlers {
  onExercise?: (option: OptionRecord) => void;
  onTransfer?: (option: OptionRecord) => void;
  onBurn?: (option: OptionRecord) => void;
  onToggleVisibility?: (option: OptionRecord) => void;
}

export const columns = (
  actionHandlers?: OptionActionHandlers,
): ColumnDef<OptionRecord>[] => [
  {
    id: 'icon',
    enableSorting: false,
    header: () => <span className='sr-only'>{t`Option Icon`}</span>,
    size: 40,
    cell: () => {
      return (
        <FilePenLine
          className='h-5 w-5 text-muted-foreground ml-1'
          aria-hidden='true'
        />
      );
    },
  },
  {
    accessorKey: 'name',
    header: () => <Trans>Name</Trans>,
    minSize: 150,
    cell: ({ row }) => {
      const option = row.original;
      const name = option.name || t`Unnamed Option`;

      return <div className='font-medium'>{name}</div>;
    },
  },
  {
    accessorKey: 'underlying_asset',
    header: () => <Trans>Underlying Asset</Trans>,
    size: 120,
    meta: {
      className: 'hidden md:table-cell',
    },
    cell: ({ row }) => {
      const asset = row.original.underlying_asset;
      return (
        <div className='flex items-center gap-2'>
          <AssetIcon asset={asset} size='md' />
          <div className='text-sm text-muted-foreground truncate'>
            {asset.kind === 'token' && (
              <NumberFormat
                value={fromMojos(
                  row.original.underlying_amount,
                  asset.precision,
                )}
                minimumFractionDigits={0}
                maximumFractionDigits={asset.precision}
              />
            )}{' '}
            {asset.name ?? asset.ticker ?? t`Unknown`}
          </div>
        </div>
      );
    },
  },
  {
    accessorKey: 'strike_asset',
    header: () => <Trans>Strike Asset</Trans>,
    size: 120,
    meta: {
      className: 'hidden lg:table-cell',
    },
    cell: ({ row }) => {
      const asset = row.original.strike_asset;
      return (
        <div className='flex items-center gap-2'>
          <AssetIcon asset={asset} size='md' />
          <div className='text-sm text-muted-foreground truncate'>
            {asset.kind === 'token' && (
              <NumberFormat
                value={fromMojos(row.original.strike_amount, asset.precision)}
                minimumFractionDigits={0}
                maximumFractionDigits={asset.precision}
              />
            )}{' '}
            {asset.name ?? asset.ticker ?? t`Unknown`}
          </div>
        </div>
      );
    },
  },
  {
    accessorKey: 'expiration_seconds',
    header: () => <Trans>Expiration</Trans>,
    size: 120,
    cell: ({ row }) => {
      const option = row.original;
      const isExpired = option.expiration_seconds * 1000 < Date.now();
      const isExpiringSoon =
        option.expiration_seconds * 1000 < Date.now() + 24 * 60 * 60 * 1000;

      return (
        <div className='flex items-center gap-1.5'>
          {isExpired ? (
            <>
              <AlertCircle className='h-4 w-4 text-red-500' />
              <span className='text-sm text-red-500 font-medium'>
                <Trans>Expired</Trans>
              </span>
            </>
          ) : isExpiringSoon ? (
            <>
              <AlertCircle className='h-4 w-4 text-yellow-500' />
              <span className='text-sm text-yellow-500 font-medium'>
                <Trans>Expiring soon</Trans>
              </span>
            </>
          ) : (
            <span className='text-sm'>
              {formatTimestamp(option.expiration_seconds, 'short', 'short')}
            </span>
          )}
        </div>
      );
    },
  },
  {
    accessorKey: 'created_height',
    header: () => <Trans>Status</Trans>,
    size: 80,
    meta: {
      className: 'hidden md:table-cell',
    },
    cell: ({ row }) => {
      const option = row.original;
      const isPending = option.created_height === null;

      return (
        <div className='text-sm'>
          {isPending ? (
            <span className='text-yellow-500 font-medium'>
              <Trans>Pending</Trans>
            </span>
          ) : (
            <span className='text-green-500 font-medium'>
              <Trans>Active</Trans>
            </span>
          )}
        </div>
      );
    },
  },
  {
    id: 'actions',
    enableSorting: false,
    size: 44,
    cell: ({ row }) => {
      const option = row.original;
      const isPending = option.created_height === null;

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
            {!isPending && (
              <>
                {actionHandlers?.onExercise && (
                  <DropdownMenuItem
                    onClick={() => actionHandlers.onExercise?.(option)}
                  >
                    <HandCoins className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Exercise</Trans>
                  </DropdownMenuItem>
                )}
                {actionHandlers?.onTransfer && (
                  <DropdownMenuItem
                    onClick={() => actionHandlers.onTransfer?.(option)}
                  >
                    <SendIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Transfer</Trans>
                  </DropdownMenuItem>
                )}
                {actionHandlers?.onBurn && (
                  <DropdownMenuItem
                    onClick={() => actionHandlers.onBurn?.(option)}
                  >
                    <Flame className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Burn</Trans>
                  </DropdownMenuItem>
                )}
              </>
            )}

            <DropdownMenuItem
              onClick={() => {
                writeText(option.launcher_id);
                toast.success(t`Option ID copied to clipboard`);
              }}
            >
              <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
              <Trans>Copy ID</Trans>
            </DropdownMenuItem>

            {actionHandlers?.onToggleVisibility && (
              <DropdownMenuItem
                onClick={() => actionHandlers.onToggleVisibility?.(option)}
              >
                {option.visible ? (
                  <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <EyeIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {option.visible ? <Trans>Hide</Trans> : <Trans>Show</Trans>}
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
