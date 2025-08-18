import { commands, events, TransactionRecord } from '@/bindings';
import { AssetCoin } from '@/components/AssetCoin';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Card } from '@/components/ui/card';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export default function Transaction() {
  const { height } = useParams();

  const [transaction, setTransaction] = useState<TransactionRecord | null>(
    null,
  );

  const updateTransaction = useCallback(() => {
    commands
      .getTransaction({
        height: Number(height),
      })
      .then((data) => {
        if (data.transaction) {
          setTransaction(data.transaction);
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
                {transaction?.spent.map((coin) => (
                  <AssetCoin
                    key={coin.coin_id}
                    asset={coin.asset}
                    amount={coin.amount}
                    coinId={coin.coin_id}
                  />
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
                {transaction?.created.map((coin) => (
                  <AssetCoin
                    key={coin.coin_id}
                    asset={coin.asset}
                    amount={coin.amount}
                    coinId={coin.coin_id}
                  />
                ))}
              </div>
            </Card>
          )}
        </div>
      </Container>
    </>
  );
}
