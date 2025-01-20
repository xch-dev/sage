import { Link } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { CatRecord } from '../bindings';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { t } from '@lingui/core/macro';
import { NumberFormat } from '@/components/NumberFormat';
import { formatUsdPrice, fromMojos } from '@/lib/utils';
import { TokenViewProps } from '@/types/TokenViewProps';

type TokenGridViewProps = TokenViewProps;

export function TokenGridView({
  cats,
  xchBalance,
  xchDecimals,
  xchBalanceUsd,
  xchPrice,
}: TokenGridViewProps) {
  return (
    <div
      role='region'
      aria-label={t`Token Grid`}
      className='mt-4 grid gap-2 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 3xl:grid-cols-6'
    >
      <Link to={`/wallet/token/xch`}>
        <Card className='transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900'>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
            <Tooltip>
              <TooltipTrigger asChild>
                <CardTitle className='text-md font-medium'>Chia</CardTitle>
              </TooltipTrigger>
              <TooltipContent>Chia</TooltipContent>
            </Tooltip>
            <img
              alt={t`Token logo`}
              aria-hidden='true'
              className='h-6 w-6'
              src='https://icons.dexie.space/xch.webp'
            />
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-medium truncate'>
              <NumberFormat
                value={fromMojos(xchBalance, xchDecimals)}
                minimumFractionDigits={0}
                maximumFractionDigits={xchDecimals}
              />
            </div>
            <div className='text-sm text-neutral-500'>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div>
                    ~
                    <NumberFormat
                      value={xchBalanceUsd}
                      style='currency'
                      currency='USD'
                      minimumFractionDigits={2}
                      maximumFractionDigits={2}
                    />
                  </div>
                </TooltipTrigger>
                <TooltipContent>
                  <span>1 XCH = ${xchPrice}</span>
                </TooltipContent>
              </Tooltip>
            </div>
          </CardContent>
        </Card>
      </Link>
      {cats.map((cat) => (
        <Link key={cat.asset_id} to={`/wallet/token/${cat.asset_id}`}>
          <Card
            className={`transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900 ${!cat.visible ? 'opacity-50 grayscale' : ''}`}
          >
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
              <Tooltip>
                <TooltipTrigger asChild>
                  <CardTitle className='text-md font-medium truncate'>
                    {cat.name || <Trans>Unknown CAT</Trans>}
                  </CardTitle>
                </TooltipTrigger>
                <TooltipContent>
                  {cat.name || <Trans>Unknown CAT</Trans>}
                </TooltipContent>
              </Tooltip>
              {cat.icon_url && (
                <img
                  alt={t`Token logo`}
                  aria-hidden='true'
                  className='h-6 w-6'
                  src={cat.icon_url}
                />
              )}
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-medium truncate'>
                <NumberFormat
                  value={fromMojos(cat.balance, 3)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={3}
                />{' '}
                {cat.ticker ?? ''}
              </div>
              <div className='text-sm text-neutral-500'>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <div>
                      ~
                      <NumberFormat
                        value={cat.balanceInUsd}
                        style='currency'
                        currency='USD'
                        minimumFractionDigits={2}
                        maximumFractionDigits={2}
                      />
                    </div>
                  </TooltipTrigger>
                  <TooltipContent>
                    <span>
                      1 {cat.ticker ?? 'CAT'} {formatUsdPrice(cat.priceInUsd)}
                    </span>
                  </TooltipContent>
                </Tooltip>
              </div>
            </CardContent>
          </Card>
        </Link>
      ))}
    </div>
  );
}
