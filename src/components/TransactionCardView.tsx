import { Link } from 'react-router-dom';
import { open } from '@tauri-apps/plugin-shell';
import BigNumber from 'bignumber.js';
import { Info } from 'lucide-react';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { TransactionRecord } from '../bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { useWalletState } from '@/state';
import { fromMojos } from '@/lib/utils';
import { nftUri } from '@/lib/nftUri';

interface TransactionCardViewProps {
  transactions: TransactionRecord[];
}

export function TransactionCardView({ transactions }: TransactionCardViewProps) {
  return (
    <div className='space-y-4' role='list' aria-label={t`Transaction history`}>
      {transactions.map((transaction) => (
        <Transaction key={transaction.height} transaction={transaction} />
      ))}
    </div>
  );
}

// Move existing Transaction component and related interfaces here
interface TransactionProps {
  transaction: TransactionRecord;
}

interface TransactionCat {
  amount: BigNumber;
  name: string | null;
  ticker: string | null;
  icon_url: string | null;
}

interface TransactionNft {
  name: string | null;
  image_mime_type: string | null;
  image_data: string | null;
  exists: boolean;
}

interface TransactionAssets {
  xch: BigNumber;
  cats: Record<string, TransactionCat>;
  nfts: Record<string, TransactionNft>;
}

interface AssetPreviewProps {
  label: string;
  assets: TransactionAssets;
  created?: boolean;
}

function Transaction({ transaction }: TransactionProps) {
  let xch = BigNumber(0);
  const cats: Record<string, TransactionCat> = {};
  const nfts: Record<string, TransactionNft> = {};

  const transactionHeight = transaction.height;
  const transactionSpentCount = transaction.spent.length;
  const transactionCreatedCount = transaction.created.length;

  for (const [coins, add] of [
    [transaction.created, true],
    [transaction.spent, false],
  ] as const) {
    for (const coin of coins) {
      switch (coin.type) {
        case 'xch': {
          if (add) {
            xch = xch.plus(coin.amount);
          } else {
            xch = xch.minus(coin.amount);
          }
          break;
        }
        case 'cat': {
          const existing = cats[coin.asset_id] || {
            amount: BigNumber(0),
            name: coin.name,
            ticker: coin.ticker,
            icon_url: coin.icon_url,
          };
          if (add) {
            existing.amount = existing.amount.plus(coin.amount);
          } else {
            existing.amount = existing.amount.minus(coin.amount);
          }
          cats[coin.asset_id] = existing;
          break;
        }
        case 'nft': {
          nfts[coin.launcher_id] = {
            name: coin.name,
            image_mime_type: coin.image_mime_type,
            image_data: coin.image_data,
            exists: add,
          };
          break;
        }
      }
    }
  }

  const assets = { xch, cats, nfts };

  return (
    <Link
      to={`/transactions/${transactionHeight}`}
      className='flex items-center gap-2 p-4 rounded-sm bg-neutral-100 dark:bg-neutral-900'
      role='listitem'
      aria-label={t`Transaction at block ` + transactionHeight}
    >
      <div className='flex justify-between'>
        <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
          <div className='flex flex-col gap-1'>
            <div
              className='text-blue-700 dark:text-blue-300 cursor-pointer'
              onClick={(event) => {
                event.preventDefault();
                open(`https://spacescan.io/block/${transactionHeight}`);
              }}
              role='button'
              aria-label={t`View block ` + transactionHeight + t` on Spacescan.io`}
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  open(`https://spacescan.io/block/${transactionHeight}`);
                }
              }}
            >
              <Trans>Block #{transactionHeight}</Trans>
            </div>
            <div className='text-sm text-muted-foreground md:w-[120px]'>
              <Trans>
                {transactionSpentCount} inputs, {transactionCreatedCount} outputs
              </Trans>
            </div>
          </div>
          <AssetPreview label={t`Sent`} assets={assets} />
          <AssetPreview label={t`Received`} assets={assets} created />
        </div>
      </div>
    </Link>
  );
}

function AssetPreview({ label, assets, created }: AssetPreviewProps) {
  const walletState = useWalletState();

  const showXch =
    (assets.xch.isGreaterThan(0) && created) ||
    (assets.xch.isLessThan(0) && !created);

  const filteredCats = Object.entries(assets.cats).filter(
    ([_, cat]) => cat.amount.isLessThan(0) === !created,
  );

  const filteredNfts = Object.entries(assets.nfts).filter(
    ([_, nft]) => nft.exists === !!created,
  );

  return (
    <div className='flex flex-col gap-1 md:w-[150px] lg:w-[200px] xl:w-[300px]'>
      <div>{label}</div>

      {!showXch && filteredCats.length === 0 && filteredNfts.length === 0 && (
        <div className='text-sm text-muted-foreground truncate'>
          <Trans>None</Trans>
        </div>
      )}

      {showXch && (
        <div className='flex items-center gap-2'>
          <img
            alt={t`XCH`}
            src='https://icons.dexie.space/xch.webp'
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground break-all'>
            <NumberFormat
              value={fromMojos(
                assets.xch.abs(),
                walletState.sync.unit.decimals,
              )}
              minimumFractionDigits={0}
              maximumFractionDigits={walletState.sync.unit.decimals}
            />{' '}
            <span className='break-normal'>{walletState.sync.unit.ticker}</span>
          </div>
        </div>
      )}
      {filteredCats.map(([_, cat]) => (
        <div className='flex items-center gap-2'>
          <img
            alt={cat.name ?? t`Unknown`}
            src={cat.icon_url!}
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground break-all'>
            <NumberFormat
              value={fromMojos(cat.amount.abs(), 3)}
              minimumFractionDigits={0}
              maximumFractionDigits={3}
            />{' '}
            <span className='break-normal'>
              {cat.ticker ?? cat.name ?? 'CAT'}
            </span>
          </div>
        </div>
      ))}
      {filteredNfts.map(([_, nft]) => (
        <div className='flex items-center gap-2'>
          <img
            src={nftUri(nft.image_mime_type, nft.image_data)}
            className='w-8 h-8'
            alt={nft.name ?? t`Unknown`}
          />

          <div className='text-sm text-muted-foreground'>
            {nft.name ?? t`Unknown`}
          </div>
        </div>
      ))}
    </div>
  );
} 