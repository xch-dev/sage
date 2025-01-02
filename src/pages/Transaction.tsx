import {
  commands,
  events,
  type TransactionCoin,
  TransactionRecord,
} from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Card } from '@/components/ui/card';
import { nftUri } from '@/lib/nftUri';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { open } from '@tauri-apps/plugin-shell';
import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export default function Transaction() {
  const { height } = useParams();

  const [transaction, setTransaction] = useState<TransactionRecord | null>(
    null,
  );

  const updateTransaction = useCallback(() => {
    commands.getTransaction({ height: Number(height) }).then((data) => {
      setTransaction(data.transaction);
    });
  }, [height]);

  useEffect(() => {
    updateTransaction();

    const unlisten = events.syncEvent.listen((data) => {
      switch (data.payload.type) {
        case 'coin_state':
        case 'cat_info':
        case 'did_info':
        case 'nft_data':
        case 'puzzle_batch_synced':
          updateTransaction();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateTransaction]);

  return (
    <>
      <Header title={`Transaction #${height}`} />
      <Container>
        <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
          {(transaction?.spent.length ?? 0) > 0 && (
            <Card className='p-4 space-y-2'>
              <div className='text-xl font-medium'>Sent</div>
              <div className='space-y-2'>
                {transaction?.spent.map((coin, i) => (
                  <TransactionCoin key={i} coin={coin} />
                ))}
              </div>
            </Card>
          )}
          {(transaction?.created.length ?? 0) > 0 && (
            <Card className='p-4 space-y-2'>
              <div className='text-xl font-medium'>Received</div>
              <div className='space-y-2'>
                {transaction?.created.map((coin, i) => (
                  <TransactionCoin key={i} coin={coin} />
                ))}
              </div>
            </Card>
          )}
        </div>
      </Container>
    </>
  );
}

interface TransactionCoinProps {
  coin: TransactionCoin;
}

function TransactionCoin({ coin }: TransactionCoinProps) {
  const coinId = coin.coin_id;

  return (
    <div
      className='p-2 rounded-md bg-neutral-100 dark:bg-neutral-800 cursor-pointer'
      onClick={() => open(`https://spacescan.io/coin/0x${coin.coin_id}`)}
    >
      <TransactionCoinKind coin={coin} />
      <div className='flex items-center gap-1'>
        <div className='text-sm text-muted-foreground truncate'>
          <Trans>Coin with id {coinId}</Trans>
        </div>
      </div>
    </div>
  );
}

interface TransactionCoinKindProps {
  coin: TransactionCoin;
}

function TransactionCoinKind({ coin }: TransactionCoinKindProps) {
  const walletState = useWalletState();

  switch (coin.type) {
    case 'xch': {
      return (
        <div className='flex items-center gap-2'>
          <img src='https://icons.dexie.space/xch.webp' className='w-8 h-8' />

          <div className='text-md text-neutral-700 dark:text-neutral-300 break-all'>
            {toDecimal(coin.amount, walletState.sync.unit.decimals)}{' '}
            <span className='break-normal'>{walletState.sync.unit.ticker}</span>
          </div>
        </div>
      );
    }
    case 'cat': {
      return (
        <div className='flex items-center gap-2'>
          <img src={coin.icon_url!} className='w-8 h-8' />

          <div className='flex flex-col'>
            <div className='text-md text-neutral-700 dark:text-neutral-300 break-all'>
              {toDecimal(coin.amount, 3)}{' '}
              <span className='break-normal'>
                {coin.ticker ?? coin.name ?? 'CAT'}
              </span>
            </div>
          </div>
        </div>
      );
    }
    case 'nft': {
      return (
        <div className='flex items-center gap-2'>
          <img
            src={nftUri(coin.image_mime_type, coin.image_data)}
            className='w-8 h-8'
          />

          <div className='text-md text-neutral-700 dark:text-neutral-300'>
            {coin.name ?? t`Unknown`}
          </div>
        </div>
      );
    }
  }

  return <div>{coin.coin_id}</div>;
}
