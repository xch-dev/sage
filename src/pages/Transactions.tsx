import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { open } from '@tauri-apps/plugin-shell';
import BigNumber from 'bignumber.js';
import { Info } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';
import { NumberFormat } from '@/components/NumberFormat';

import { Pagination } from '@/components/Pagination';
import { useLocalStorage } from 'usehooks-ts';
import { ViewToggle, ViewMode } from '@/components/ViewToggle';
import { TransactionTableView } from '../components/TransactionTableView';

export function Transactions() {
  const { addError } = useErrors();

  const [params, setParams] = useSearchParams();
  const page = parseInt(params.get('page') ?? '1');
  const setPage = (page: number) => setParams({ page: page.toString() });

  const [pageSize, setPageSize] = useLocalStorage('transactionsPageSize', 8);
  const [view, setView] = useLocalStorage<ViewMode>('transactionsView', 'list');

  // TODO: Show pending transactions
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [total, setTotal] = useState(0);

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setPending(data.transactions))
      .catch(addError);
    console.log('getTransactionsEx', (page - 1) * pageSize, pageSize);
    commands
      .getTransactions({ offset: (page - 1) * pageSize, limit: pageSize })
      .then((data) => {
        setTransactions(data.transactions);
        setTotal(data.total);
      })
      .catch(addError);
  }, [addError, page, pageSize]);

  useEffect(() => {
    updateTransactions();

    const unlisten = events.syncEvent.listen((data) => {
      switch (data.payload.type) {
        case 'coin_state':
        case 'cat_info':
        case 'did_info':
        case 'nft_data':
        case 'puzzle_batch_synced':
          updateTransactions();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateTransactions]);

  return (
    <>
      <Header title={t`Transactions`}>
        <ReceiveAddress />
      </Header>
      <Container>
        {transactions.length === 0 && (
          <Alert className='mb-4'>
            <Info className='h-4 w-4' />
            <AlertTitle>
              <Trans>Note</Trans>
            </AlertTitle>
            <AlertDescription>
              <Trans>You have not made any transactions yet.</Trans>
            </AlertDescription>
          </Alert>
        )}

        <div className='flex items-center justify-between mb-4'>
          <Pagination
            total={total}
            page={page}
            onPageChange={setPage}
            pageSize={pageSize}
            onPageSizeChange={setPageSize}
          />
          <ViewToggle view={view} onChange={setView} />
        </div>

        {view === 'list' ? (
          <div
            className='space-y-4'
            role='list'
            aria-label={t`Transaction history`}
          >
            {transactions.map((transaction) => (
              <Transaction key={transaction.height} transaction={transaction} />
            ))}
          </div>
        ) : (
          <TransactionTableView 
            transactions={transactions} 
            onSortingChange={(ascending) => {
              commands.getTransactionsEx({ 
                offset: (page - 1) * pageSize, 
                limit: pageSize,
                ascending 
              })
              .then((data) => {
                setTransactions(data.transactions);
                setTotal(data.total);
              })
              .catch(addError);
            }}
          />
        )}

        {total > pageSize && (
          <div className='mt-4'>
            <Pagination
              total={total}
              page={page}
              onPageChange={setPage}
              pageSize={pageSize}
              onPageSizeChange={setPageSize}
            />
          </div>
        )}
      </Container>
    </>
  );
}

interface TransactionProps {
  transaction: TransactionRecord;
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
              aria-label={
                t`View block ` + transactionHeight + t` on Spacescan.io`
              }
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
                {transactionSpentCount} inputs, {transactionCreatedCount}{' '}
                outputs
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
