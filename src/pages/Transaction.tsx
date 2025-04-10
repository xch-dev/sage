import {
  commands,
  events,
  type TransactionCoin,
  TransactionRecord,
} from '@/bindings';
import Container from '@/components/Container';
import { CopyButton } from '@/components/CopyButton';
import Header from '@/components/Header';
import { NumberFormat } from '@/components/NumberFormat';
import { Card } from '@/components/ui/card';
import { nftUri } from '@/lib/nftUri';
import { formatAddress, fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { toast } from 'react-toastify';

export default function Transaction() {
  const { height } = useParams();

  const [transaction, setTransaction] = useState<TransactionRecord | null>(
    null,
  );

  const updateTransaction = useCallback(() => {
    commands
      .getTransactions({
        offset: 0,
        limit: 1,
        ascending: true,
        find_value: height ?? '',
      })
      .then((data) => {
        if (data.transactions.length > 0) {
          setTransaction(data.transactions[0]);
        } else {
          setTransaction(null);
        }
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
      <Header title={t`Transaction #${height}`} />
      <Container>
        <Card className='p-6 mb-4'>
          <div className='text-xl font-medium'>
            <Trans>Transaction Time</Trans>
          </div>
          <div className='text-md text-neutral-700 dark:text-neutral-300 mt-2'>
            {formatTimestamp(transaction?.timestamp ?? null)}
          </div>
        </Card>
        <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
          {(transaction?.spent.length ?? 0) > 0 && (
            <Card className='p-6'>
              <div className='text-xl font-medium mb-4'>
                <Trans>Sent</Trans>
              </div>
              <div className='space-y-4'>
                {transaction?.spent.map((coin, i) => (
                  <TransactionCoin key={i} coin={coin} />
                ))}
              </div>
            </Card>
          )}
          {(transaction?.created.length ?? 0) > 0 && (
            <Card className='p-6'>
              <div className='text-xl font-medium mb-4'>
                <Trans>Received</Trans>
              </div>
              <div className='space-y-4'>
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
    <div className='rounded-xl border border-neutral-200 bg-white text-neutral-950 shadow dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-50 p-4'>
      <div
        className='cursor-pointer'
        onClick={() => openUrl(`https://spacescan.io/coin/0x${coin.coin_id}`)}
        aria-label={t`View coin ${coinId} on Spacescan.io`}
        role='button'
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            openUrl(`https://spacescan.io/coin/0x${coin.coin_id}`);
          }
        }}
      >
        <TransactionCoinKind coin={coin} />
        <div className='flex items-center gap-1 mt-2'>
          <div className='text-sm text-muted-foreground truncate'>
            <Trans>Coin with id {coinId}</Trans>
          </div>
        </div>
      </div>
      {coin.type !== 'xch' && <TransactionCoinId coin={coin} />}
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
          <img
            alt={t`XCH`}
            src='https://icons.dexie.space/xch.webp'
            className='w-8 h-8'
            aria-hidden={true}
          />

          <div className='text-md text-neutral-700 dark:text-neutral-300 break-all'>
            <NumberFormat
              value={fromMojos(coin.amount, walletState.sync.unit.decimals)}
              minimumFractionDigits={0}
              maximumFractionDigits={walletState.sync.unit.decimals}
            />{' '}
            <span className='break-normal'>{walletState.sync.unit.ticker}</span>
          </div>
        </div>
      );
    }
    case 'cat': {
      return (
        <div className='flex items-center gap-2'>
          <img
            alt={coin.name ?? t`Unknown`}
            src={coin.icon_url!}
            className='w-8 h-8'
            aria-hidden={true}
          />

          <div className='flex flex-col'>
            <div className='text-md text-neutral-700 dark:text-neutral-300 break-all'>
              <NumberFormat
                value={fromMojos(coin.amount, 3)}
                minimumFractionDigits={0}
                maximumFractionDigits={3}
              />{' '}
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
            alt={coin.name ?? t`Unknown`}
            src={nftUri(coin.icon ? 'image/png' : null, coin.icon)}
            className='w-8 h-8'
            aria-label={coin.name ?? t`Unknown`}
          />

          <div className='text-md text-neutral-700 dark:text-neutral-300'>
            {coin.name ?? t`Unknown`}
          </div>
        </div>
      );
    }
  }

  return <div className='break-all'>{coin.coin_id}</div>;
}

interface TransactionCoinIdProps {
  coin: TransactionCoin;
}

function TransactionCoinId({ coin }: TransactionCoinIdProps) {
  let id = '';
  let label = '';
  let toastMessage = '';

  switch (coin.type) {
    case 'cat':
      id = coin.asset_id;
      label = t`Asset ID`;
      toastMessage = t`Asset ID copied to clipboard`;
      break;
    case 'nft':
      id = coin.launcher_id;
      label = t`Launcher ID`;
      toastMessage = t`Launcher ID copied to clipboard`;
      break;
    case 'did':
      id = coin.launcher_id;
      label = t`Launcher ID`;
      toastMessage = t`Launcher ID copied to clipboard`;
      break;
    default:
      return null;
  }

  const handleCopyClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  const handleCopy = () => {
    toast.success(toastMessage);
  };

  return (
    <div className='flex items-center gap-2 mt-2 text-sm text-muted-foreground'>
      <span>{label}:</span>
      <span className='font-mono'>{formatAddress(id, 6, 6)}</span>
      <div onClick={handleCopyClick}>
        <CopyButton value={id} className='h-6 w-6 p-0' onCopy={handleCopy} />
      </div>
    </div>
  );
}
