import { Link } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { CatRecord } from '../bindings';
import { toDecimal } from '@/lib/utils';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { t } from '@lingui/core/macro';

interface TokenGridViewProps {
  cats: Array<CatRecord & { balanceInUsd: number; sortValue: number }>;
  xchBalance: string;
  xchDecimals: number;
  getBalanceInUsd: (assetId: string, balance: string) => string;
}

export function TokenGridView({
  cats,
  xchBalance,
  xchDecimals,
  getBalanceInUsd,
}: TokenGridViewProps) {
  return (
    <div className='mt-4 grid gap-2 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 3xl:grid-cols-6'>
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
              alt={t`XCH logo`}
              className='h-6 w-6'
              src='https://icons.dexie.space/xch.webp'
            />
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-medium truncate'>
              {toDecimal(xchBalance, xchDecimals)}
            </div>
            <div className='text-sm text-neutral-500'>
              ~${getBalanceInUsd('xch', toDecimal(xchBalance, xchDecimals))}
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
                    {cat.name || t`Unknown CAT`}
                  </CardTitle>
                </TooltipTrigger>
                <TooltipContent>
                  {cat.name || t`Unknown CAT`}
                </TooltipContent>
              </Tooltip>
              {cat.icon_url && (
                <img
                  alt={`${cat.asset_id} logo`}
                  className='h-6 w-6'
                  src={cat.icon_url}
                />
              )}
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-medium truncate'>
                {toDecimal(cat.balance, 3)} {cat.ticker ?? ''}
              </div>
              <div className='text-sm text-neutral-500'>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <div>~${cat.balanceInUsd}</div>
                  </TooltipTrigger>
                  <TooltipContent>
                    <span>
                      1 {cat.ticker ?? 'CAT'}{' '}
                      {Number(
                        cat.balanceInUsd /
                          Number(toDecimal(cat.balance, 3)),
                      ) < 0.01
                        ? ' < 0.01¢'
                        : Number(
                              cat.balanceInUsd /
                                Number(toDecimal(cat.balance, 3)),
                            ) < 0.01
                          ? ` = ${((cat.balanceInUsd / Number(toDecimal(cat.balance, 3))) * 100).toFixed(2)}¢`
                          : ` = $${(cat.balanceInUsd / Number(toDecimal(cat.balance, 3))).toFixed(2)}`}
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