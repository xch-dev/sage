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
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead></TableHead>
          <TableHead><Trans>Name</Trans></TableHead>
          <TableHead><Trans>Symbol</Trans></TableHead>
          <TableHead className="text-right"><Trans>Balance</Trans></TableHead>
          <TableHead className="text-right"><Trans>Value (USD)</Trans></TableHead>
          <TableHead className="text-right"><Trans>Price</Trans></TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell>
            <img
              alt={`XCH logo`}
              className='h-6 w-6'
              src='https://icons.dexie.space/xch.webp'
            />
          </TableCell>
          <TableCell>Chia</TableCell>
          <TableCell>XCH</TableCell>
          <TableCell className="text-right">{toDecimal(xchBalance, xchDecimals)}</TableCell>
          <TableCell className="text-right">
            ${getBalanceInUsd('xch', toDecimal(xchBalance, xchDecimals))}
          </TableCell>
          <TableCell className="text-right">
            ${Number(getBalanceInUsd('xch', '1')).toFixed(2)}
          </TableCell>
        </TableRow>
        {cats.map((cat) => (
          <TableRow key={cat.asset_id}>
            <TableCell>
              {cat.icon_url && (
                <img
                  alt={`${cat.asset_id} logo`}
                  className='h-6 w-6'
                  src={cat.icon_url}
                />
              )}
            </TableCell>
            <TableCell className="font-medium">
              <Link to={`/wallet/token/${cat.asset_id}`}>
                {cat.name || <Trans>Unknown CAT</Trans>}
              </Link>
            </TableCell>
            <TableCell>{cat.ticker || '-'}</TableCell>
            <TableCell className="text-right">
              {toDecimal(cat.balance, 3)}
            </TableCell>
            <TableCell className="text-right">${cat.balanceInUsd.toFixed(2)}</TableCell>
            <TableCell className="text-right">
              {Number(cat.balance) > 0
                ? `$${(cat.balanceInUsd / Number(toDecimal(cat.balance, 3))).toFixed(2)}`
                : '-'}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
} 