import { NumberFormat } from '@/components/NumberFormat';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { formatUsdPrice, fromMojos, getAssetDisplayName } from '@/lib/utils';
import { PricedTokenRecord, TokenViewProps } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { MoreHorizontal } from 'lucide-react';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { AssetIcon } from './AssetIcon';
import { TokenActionHandlers } from './TokenColumns';

type TokenGridViewProps = TokenViewProps & {
  actionHandlers?: TokenActionHandlers;
};

function TokenCardMenu({
  record,
  actionHandlers,
}: {
  record: PricedTokenRecord;
  actionHandlers?: TokenActionHandlers;
}) {
  const balance = fromMojos(record.balance, record.precision);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant='ghost'
          className='h-6 w-6 p-0'
          aria-label={t`Open actions menu`}
          onClick={(e) => e.preventDefault()} // Prevent card navigation when clicking menu
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
                onClick={(e) => {
                  e.preventDefault();
                  if (!record.asset_id) return;
                  actionHandlers.onRefreshInfo?.(record.asset_id);
                }}
              >
                <Trans>Refresh Info</Trans>
              </DropdownMenuItem>
            )}
            {actionHandlers?.onToggleVisibility && (
              <DropdownMenuItem
                onClick={(e) => {
                  e.preventDefault();
                  actionHandlers.onToggleVisibility?.(record);
                }}
              >
                {record.visible ? <Trans>Hide</Trans> : <Trans>Show</Trans>}{' '}
                <Trans>Asset</Trans>
              </DropdownMenuItem>
            )}
            <DropdownMenuItem
              onClick={(e) => {
                e.preventDefault();
                if (!record.asset_id) return;
                navigator.clipboard.writeText(record.asset_id);
                toast.success(t`Asset ID copied to clipboard`);
              }}
            >
              <Trans>Copy Asset ID</Trans>
            </DropdownMenuItem>
          </>
        )}
        <DropdownMenuItem
          onClick={(e) => {
            e.preventDefault();
            navigator.clipboard.writeText(balance.toString());
            toast.success(t`Balance copied to clipboard`);
          }}
        >
          <Trans>Copy Balance</Trans>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function TokenGridView({ tokens, actionHandlers }: TokenGridViewProps) {
  return (
    <div
      role='region'
      aria-label={t`Token Grid`}
      className='relative w-full overflow-auto mt-4'
    >
      <div className='grid gap-2 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 3xl:grid-cols-6'>
        {tokens.map((token) => {
          return (
            <Link
              key={token.asset_id ?? 'xch'}
              to={`/wallet/token/${token.asset_id ?? 'xch'}`}
            >
              <Card
                className={`transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900 ${!token.visible ? 'opacity-50 grayscale' : ''}`}
              >
                <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <CardTitle className='text-md font-medium truncate'>
                        {getAssetDisplayName(token.name, token.ticker, 'token')}
                      </CardTitle>
                    </TooltipTrigger>
                    <TooltipContent>
                      {getAssetDisplayName(token.name, token.ticker, 'token')}
                    </TooltipContent>
                  </Tooltip>

                  <AssetIcon asset={{ ...token, kind: 'token' }} size='sm' />
                </CardHeader>
                <CardContent className='flex flex-col gap-1'>
                  <div className='text-2xl font-medium truncate'>
                    <NumberFormat
                      value={fromMojos(token.balance, token.precision)}
                      minimumFractionDigits={0}
                      maximumFractionDigits={3}
                    />{' '}
                    {token.ticker ?? ''}
                  </div>
                  <div className='flex justify-between items-center text-sm text-neutral-500'>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div>
                          ~
                          <NumberFormat
                            value={token.balanceInUsd}
                            style='currency'
                            currency='USD'
                            minimumFractionDigits={2}
                            maximumFractionDigits={2}
                          />
                        </div>
                      </TooltipTrigger>
                      <TooltipContent>
                        <span>
                          1 {token.ticker ?? 'CAT'}{' '}
                          {formatUsdPrice(token.priceInUsd)}
                        </span>
                      </TooltipContent>
                    </Tooltip>
                    <TokenCardMenu
                      record={token}
                      actionHandlers={actionHandlers}
                    />
                  </div>
                </CardContent>
              </Card>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
