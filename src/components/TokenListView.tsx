import { NumberFormat } from '@/components/NumberFormat';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatUsdPrice, toDecimal } from '@/lib/utils';
import { TokenViewProps } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';

type TokenListViewProps = TokenViewProps;

export function TokenListView({
  cats,
  xchBalance,
  xchDecimals,
  xchBalanceUsd,
  xchPrice,
}: TokenListViewProps) {
  return (
    <div role='region' aria-label={t`Token List`}>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead scope='col' className='w-[40px] min-w-[40px]'>
              <span className='sr-only'>
                <Trans>Token Icon</Trans>
              </span>
            </TableHead>
            <TableHead scope='col' className='w-[120px]'>
              <Trans>Name</Trans>
            </TableHead>
            <TableHead scope='col' className='hidden sm:table-cell'>
              <Trans>Symbol</Trans>
            </TableHead>
            <TableHead scope='col' className='text-right w-[100px]'>
              <Trans>Balance</Trans>
            </TableHead>
            <TableHead scope='col' className='text-right hidden md:table-cell'>
              <Trans>Balance (USD)</Trans>
            </TableHead>
            <TableHead scope='col' className='text-right hidden md:table-cell'>
              <Trans>Price (USD)</Trans>
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell className='w-[40px] min-w-[40px]'>
              <img
                alt={t`XCH logo`}
                aria-hidden='true'
                className='h-6 w-6'
                src='https://icons.dexie.space/xch.webp'
              />
            </TableCell>
            <TableCell className='max-w-[120px] truncate'>
              <Link
                to='/wallet/token/xch'
                className='hover:underline'
                aria-label={t`View Chia token details`}
              >
                Chia
              </Link>
            </TableCell>
            <TableCell className='hidden sm:table-cell'>XCH</TableCell>
            <TableCell className='text-right '>
              <NumberFormat
                value={toDecimal(xchBalance, xchDecimals)}
                maximumFractionDigits={xchDecimals}
              />
            </TableCell>
            <TableCell className='text-right hidden md:table-cell'>
              <span className='sr-only'>USD Value: </span>
              <NumberFormat
                value={xchBalanceUsd}
                style='currency'
                currency='USD'
                maximumFractionDigits={2}
              />
            </TableCell>
            <TableCell className='text-right hidden md:table-cell'>
              <span className='sr-only'>Price per token: </span>
              <NumberFormat
                value={xchPrice}
                style='currency'
                currency='USD'
                maximumFractionDigits={2}
              />
            </TableCell>
          </TableRow>
          {cats.map((cat) => (
            <TableRow key={cat.asset_id}>
              <TableCell className='w-[40px] min-w-[40px]'>
                {cat.icon_url && (
                  <img
                    alt={t`Token logo`}
                    aria-hidden='true'
                    className='h-6 w-6'
                    src={cat.icon_url}
                  />
                )}
              </TableCell>
              <TableCell className='max-w-[120px] truncate'>
                <Link
                  to={`/wallet/token/${cat.asset_id}`}
                  className='hover:underline'
                  aria-label={(() => {
                    const name = cat.name;
                    return name
                      ? t`View ${name} token details`
                      : t`View Unknown CAT token details`;
                  })()}
                >
                  {cat.name || <Trans>Unknown CAT</Trans>}
                </Link>
              </TableCell>
              <TableCell className='hidden sm:table-cell'>
                {cat.ticker || '-'}
              </TableCell>
              <TableCell className='text-right '>
                <NumberFormat
                  value={toDecimal(cat.balance, 3)}
                  maximumFractionDigits={3}
                />
              </TableCell>
              <TableCell className='text-right hidden md:table-cell'>
                <span className='sr-only'>USD Value: </span>
                <NumberFormat
                  value={cat.balanceInUsd}
                  style='currency'
                  currency='USD'
                  maximumFractionDigits={2}
                />
              </TableCell>
              <TableCell className='text-right hidden md:table-cell'>
                <span className='sr-only'>Price per token: </span>
                {formatUsdPrice(cat.priceInUsd)}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
