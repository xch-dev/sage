import { Link } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { CatRecord } from '../bindings';
import { toDecimal } from '@/lib/utils';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { t } from '@lingui/core/macro';
interface TokenListViewProps {
  cats: Array<CatRecord & { balanceInUsd: number; sortValue: number }>;
  xchBalance: string;
  xchDecimals: number;
  getBalanceInUsd: (assetId: string, balance: string) => string;
}

export function TokenListView({ 
  cats, 
  xchBalance, 
  xchDecimals, 
  getBalanceInUsd 
}: TokenListViewProps) {
  return (
    <div role="region" aria-label={t`Token List`}>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead scope="col"></TableHead>
            <TableHead scope="col"><Trans>Name</Trans></TableHead>
            <TableHead scope="col"><Trans>Symbol</Trans></TableHead>
            <TableHead scope="col" className="text-right"><Trans>Balance</Trans></TableHead>
            <TableHead scope="col" className="text-right"><Trans>Value (USD)</Trans></TableHead>
            <TableHead scope="col" className="text-right"><Trans>Price</Trans></TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>
              <img
                alt={t`XCH logo`}
                aria-hidden="true"
                className='h-6 w-6'
                src='https://icons.dexie.space/xch.webp'
              />
            </TableCell>
            <TableCell>
              <Link to="/wallet/token/xch" className="hover:underline">
                Chia
              </Link>
            </TableCell>
            <TableCell>XCH</TableCell>
            <TableCell className="text-right">
              {toDecimal(xchBalance, xchDecimals)}
            </TableCell>
            <TableCell className="text-right">
              <span className="sr-only">USD Value: </span>
              ${getBalanceInUsd('xch', toDecimal(xchBalance, xchDecimals))}
            </TableCell>
            <TableCell className="text-right">
              <span className="sr-only">Price per token: </span>
              ${Number(getBalanceInUsd('xch', '1')).toFixed(2)}
            </TableCell>
          </TableRow>
          {cats.map((cat) => (
            <TableRow key={cat.asset_id}>
              <TableCell>
                {cat.icon_url && (
                  <img
                    alt={t`Token logo`}
                    aria-hidden="true"
                    className='h-6 w-6'
                    src={cat.icon_url}
                  />
                )}
              </TableCell>
              <TableCell>
                <Link 
                  to={`/wallet/token/${cat.asset_id}`}
                  className="hover:underline"
                >
                  {cat.name || <Trans>Unknown CAT</Trans>}
                </Link>
              </TableCell>
              <TableCell>{cat.ticker || '-'}</TableCell>
              <TableCell className="text-right">
                {toDecimal(cat.balance, 3)}
              </TableCell>
              <TableCell className="text-right">
                <span className="sr-only">USD Value: </span>
                ${cat.balanceInUsd.toFixed(2)}
              </TableCell>
              <TableCell className="text-right">
                <span className="sr-only">Price per token: </span>
                {Number(cat.balance) > 0
                  ? `$${(cat.balanceInUsd / Number(toDecimal(cat.balance, 3))).toFixed(2)}`
                  : '-'}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
} 