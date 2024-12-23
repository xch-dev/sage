import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { ChevronLeft, ChevronRight, Info } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { t } from '@lingui/core/macro';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';

export function Transactions() {
  const walletState = useWalletState();

  const { addError } = useErrors();

  // TODO: Show pending transactions
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [page, setPage] = useState(0);
  const [total, setTotal] = useState(0);

  const pageSize = 8;

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setPending(data.transactions))
      .catch(addError);

    commands
      .getTransactions({ offset: page * pageSize, limit: pageSize })
      .then((data) => {
        setTransactions(data.transactions);
        setTotal(Math.max(1, Math.ceil(data.total / pageSize)));
      })
      .catch(addError);
  }, [addError, page]);

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
            <AlertTitle>Note</AlertTitle>
            <AlertDescription>
              You have not made any transactions yet.
            </AlertDescription>
          </Alert>
        )}

        <div className='flex items-center gap-4'>
          <Button
            variant='outline'
            size='icon'
            onClick={() => setPage(Math.max(0, page - 1))}
            disabled={page === 0}
          >
            <ChevronLeft className='w-4 h-4' />
          </Button>
          <span className='text-sm'>
            {page + 1}/{total}
          </span>
          <Button
            variant='outline'
            size='icon'
            onClick={() => setPage(Math.min(total - 1, page + 1))}
            disabled={page === total - 1}
          >
            <ChevronRight className='w-4 h-4' />
          </Button>
        </div>

        {transactions.length > 0 && (
          <div className='flex flex-col gap-2 mt-2'>
            {transactions.map((transaction, i) => {
              return <Transaction key={i} transaction={transaction} />;
            })}
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

  console.log(transaction);

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
    <div className='flex items-center gap-2 p-4 rounded-sm bg-neutral-100 dark:bg-neutral-900'>
      <div className='flex justify-between'>
        <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
          <div className='flex flex-col gap-2'>
            <div>Block #{transaction.height}</div>
            <div className='text-sm text-muted-foreground truncate'>
              {transaction.spent.length} coins spent,{' '}
              {transaction.created.length} created
            </div>
          </div>
          <AssetPreview label='Sent' assets={assets} />
          <AssetPreview label='Received' assets={assets} created />
        </div>
      </div>
    </div>
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
    ([_, cat]) => cat.amount.isLessThan(0) || created,
  );

  const filteredNfts = Object.entries(assets.nfts).filter(
    ([_, nft]) => nft.exists === !!created,
  );

  return (
    <div className='flex flex-col gap-1 w-[125px] lg:w-[200px] xl:w-[300px]'>
      <div>{label}</div>

      {!showXch && filteredCats.length === 0 && filteredNfts.length === 0 && (
        <div className='text-sm text-muted-foreground truncate'>None</div>
      )}

      {showXch && (
        <div className='flex items-center gap-2'>
          <img src='https://icons.dexie.space/xch.webp' className='w-8 h-8' />

          <div className='text-sm text-muted-foreground truncate'>
            {toDecimal(
              assets.xch.abs().toString(),
              walletState.sync.unit.decimals,
            )}{' '}
            {walletState.sync.unit.ticker}
          </div>
        </div>
      )}
      {filteredCats.map(([_, cat]) => (
        <div className='flex items-center gap-2'>
          <img src={cat.icon_url!} className='w-8 h-8' />

          <div className='text-sm text-muted-foreground truncate'>
            {toDecimal(cat.amount.abs().toString(), 3)}{' '}
            {cat.name ?? cat.ticker ?? 'Unknown'}
          </div>
        </div>
      ))}
      {filteredNfts.map(([_, nft]) => (
        <div className='flex items-center gap-2'>
          <img
            src={nftUri(nft.image_mime_type, nft.image_data)}
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground truncate'>
            {nft.name ?? 'Unknown'}
          </div>
        </div>
      ))}
    </div>
  );
}
