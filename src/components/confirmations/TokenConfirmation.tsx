import { CoinRecord } from '@/bindings';
import { CopyButton } from '@/components/CopyButton';
import { MemoDisplay } from '@/components/MemoDisplay.tsx';
import { formatNumber } from '@/i18n.ts';
import { offersEnabled } from '@/lib/features';
import { fromMojos } from '@/lib/utils';
import { formatMemo, Memo } from '@/types/CoinMemo.ts';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import {
  CoinsIcon,
  MergeIcon,
  SplitIcon,
  TriangleAlertIcon,
} from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

type TokenOperationType =
  | 'split'
  | 'combine'
  | 'issue'
  | 'send'
  | 'clawback'
  | 'finalize_clawback';

interface TokenConfirmationProps {
  type: TokenOperationType;
  coins?: CoinRecord[];
  outputCount?: number;
  ticker?: string;
  precision?: number;
  name?: string;
  amount?: string;
  revocable?: boolean;
  currentMemo?: Memo;
}

export function TokenConfirmation({
  type,
  coins,
  outputCount,
  ticker,
  precision,
  name,
  amount,
  revocable,
  currentMemo,
}: TokenConfirmationProps) {
  const config = {
    split: {
      icon: SplitIcon,
      title: <Trans>Split Coins</Trans>,
      variant: 'info' as const,
      message: offersEnabled ? (
        <Trans>
          You are splitting coins into multiple coins of equal value. This can
          help with parallel transactions and offer creation.
        </Trans>
      ) : (
        <Trans>
          You are splitting coins into multiple coins of equal value. This can
          help with parallel transactions.
        </Trans>
      ),
    },
    combine: {
      icon: MergeIcon,
      title: <Trans>Combine Coins</Trans>,
      variant: 'info' as const,
      message: (
        <Trans>
          You are combining multiple coins into a single coin. This can help
          reduce the number of coins in your wallet.
        </Trans>
      ),
    },
    issue: {
      icon: CoinsIcon,
      title: <Trans>Token Issuance</Trans>,
      variant: 'info' as const,
      message: (
        <>
          <Trans>
            You are issuing a new token. This will create a CAT (Chia Asset
            Token) that can be sent to other users and traded on exchanges.
          </Trans>{' '}
          {revocable && (
            <Trans>
              This wallet&apos;s change address will be used as the revocation
              address.
            </Trans>
          )}
        </>
      ),
    },
    clawback: {
      icon: CoinsIcon,
      title: <Trans>Claw Back</Trans>,
      variant: 'info' as const,
      message: (
        <Trans>
          You are about to claw back coins. This will return them to your
          wallet.
        </Trans>
      ),
    },
    send: {
      icon: CoinsIcon,
      title: <Trans>Send Token</Trans>,
      variant: 'info' as const,
      message: null,
    },
    finalize_clawback: {
      icon: CoinsIcon,
      title: <Trans>Finalize Clawback</Trans>,
      variant: 'info' as const,
      message: (
        <Trans>
          You are about to finalize the clawback transaction. This will send the
          funds to the original recipient (even if the recipient wallet does not
          support clawbacks).
        </Trans>
      ),
    },
  };

  const { icon: Icon, title, variant, message } = config[type];

  const coinCount = coins?.length ?? 0;

  const totalAmount =
    coins?.reduce((acc, coin) => acc + BigInt(coin.amount), BigInt(0)) ??
    BigInt(0);

  const formattedTotalAmount =
    precision !== undefined
      ? String(fromMojos(totalAmount.toString(), precision))
      : '';

  return (
    <div className='space-y-3 text-xs'>
      {message && (
        <ConfirmationAlert icon={Icon} title={title} variant={variant}>
          {message}
        </ConfirmationAlert>
      )}

      {type === 'issue' && revocable && (
        <ConfirmationAlert
          icon={TriangleAlertIcon}
          title={<Trans>Revocable CAT Warning</Trans>}
          variant='warning'
        >
          <Trans>
            Only issue a revocable CAT if you understand the risks. Sage cannot
            revoke it; revocation currently requires external tools.
          </Trans>
        </ConfirmationAlert>
      )}

      {type === 'send' && currentMemo && (
        <ConfirmationCard>
          <div className='flex items-center justify-between'>
            <div className='break-words whitespace-pre-wrap flex-1'>
              <MemoDisplay memo={currentMemo} />
            </div>
            <CopyButton
              value={formatMemo(currentMemo)}
              className='h-4 w-4 shrink-0 ml-2'
              onCopy={() => toast.success(t`Data copied to clipboard`)}
            />
          </div>
        </ConfirmationCard>
      )}

      {type === 'issue' && (
        <ConfirmationCard
          icon={<CoinsIcon className='h-8 w-8 text-blue-500' />}
          title={name}
        >
          <div className='grid grid-cols-3 gap-2'>
            <div>
              <div className='text-muted-foreground text-xs mb-1'>
                <Trans>Ticker</Trans>
              </div>
              <div className='font-medium'>{ticker}</div>
            </div>

            <div>
              <div className='text-muted-foreground text-xs mb-1'>
                <Trans>Amount</Trans>
              </div>
              <div className='font-medium'>
                {formatNumber({
                  value: amount ?? '0',
                  style: 'decimal',
                  minimumFractionDigits: 0,
                  maximumFractionDigits: 0,
                })}{' '}
                {ticker}
              </div>
            </div>

            <div>
              <div className='text-muted-foreground text-xs mb-1'>
                <Trans>Revocable</Trans>
              </div>
              <div className='font-medium'>
                {revocable ? <Trans>Yes</Trans> : <Trans>No</Trans>}
              </div>
            </div>
          </div>
        </ConfirmationCard>
      )}

      {(type === 'split' ||
        type === 'combine' ||
        type === 'clawback' ||
        type === 'finalize_clawback') &&
        coins && (
          <>
            <ConfirmationCard
              title={
                type === 'split' ? (
                  <Plural
                    value={coinCount}
                    one='Split # coin'
                    other='Split # coins'
                  />
                ) : type === 'combine' ? (
                  <Plural
                    value={coinCount}
                    one='Combine # coin'
                    other='Combine # coins'
                  />
                ) : type === 'clawback' ? (
                  <Plural
                    value={coinCount}
                    one='Claw back # coin'
                    other='Claw back # coins'
                  />
                ) : (
                  <Plural
                    value={coinCount}
                    one='Finalize clawback # coin'
                    other='Finalize clawback # coins'
                  />
                )
              }
            >
              <div className='space-y-2 max-h-40 overflow-y-auto'>
                {coins.map((coin) => (
                  <div
                    key={coin.coin_id}
                    className='flex justify-between items-center border-b border-border pb-1 last:border-0 last:pb-0'
                  >
                    <div className='truncate flex-1 mr-2'>
                      {coin.coin_id.substring(0, 8)}...
                      {coin.coin_id.substring(coin.coin_id.length - 8)}
                    </div>
                    <div className='font-medium'>
                      {String(
                        fromMojos(coin.amount.toString(), precision ?? 0),
                      )}{' '}
                      {ticker}
                    </div>
                  </div>
                ))}
              </div>
            </ConfirmationCard>

            <ConfirmationCard>
              <div className='grid grid-cols-2 gap-4'>
                <div>
                  <div className='text-muted-foreground mb-1'>
                    <Trans>Total Amount</Trans>
                  </div>
                  <div className='font-medium'>
                    {formattedTotalAmount} {ticker}
                  </div>
                </div>
                <div>
                  <div className='text-muted-foreground mb-1'>
                    {type === 'split' ? (
                      <Trans>Output Count</Trans>
                    ) : (
                      <Trans>Result</Trans>
                    )}
                  </div>
                  <div className='font-medium'>
                    {type === 'split' ? (
                      <Trans>{outputCount} coins</Trans>
                    ) : type === 'combine' ? (
                      <Trans>1 combined coin</Trans>
                    ) : type === 'clawback' ? (
                      <Plural
                        value={coinCount}
                        one='# clawed back coin'
                        other='# clawed back coins'
                      />
                    ) : type === 'finalize_clawback' ? (
                      <Plural
                        value={coinCount}
                        one='# finalized clawback'
                        other='# finalized clawbacks'
                      />
                    ) : null}
                  </div>
                </div>
              </div>
            </ConfirmationCard>
          </>
        )}

      {type === 'issue' && (
        <div className='text-muted-foreground'>
          {offersEnabled ? (
            <Trans>
              Once issued, this token will appear in your wallet. You can then
              send it to other addresses or create offers to trade it.
            </Trans>
          ) : (
            <Trans>
              Once issued, this token will appear in your wallet. You can then
              send it to other addresses.
            </Trans>
          )}
        </div>
      )}
    </div>
  );
}
