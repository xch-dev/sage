import { toDecimal, fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { BadgeMinus, BadgePlus, BoxIcon, ForwardIcon } from 'lucide-react';
import { TransactionSummary, Unit } from '../bindings';
import { Badge } from './ui/badge';
import { formatNumber } from '../i18n';

interface SpentCoin {
  sort: number;
  badge: string;
  label: string;
  coinId: string;
}

interface CreatedCoin {
  sort: number;
  badge: string;
  label: string;
  address: string;
  amount?: string;
}

// Export AdvancedSummary for use in WalletConnectContext
export interface AdvancedTransactionSummaryProps {
  summary: TransactionSummary;
}

export function AdvancedTransactionSummary({
  summary,
}: AdvancedTransactionSummaryProps) {
  const walletState = useWalletState();

  const { spent, created } = calculateTransaction(
    walletState.sync.unit,
    summary,
  );

  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex flex-col gap-2 w-full font-medium text-left text-neutral-900 dark:text-neutral-200 bg-neutral-100 dark:bg-neutral-900 p-2 rounded-md'>
        <div className='flex items-center gap-2 text-lg'>
          <BadgeMinus className='w-6 h-6' />
          <span>
            <Trans>Spent Coins</Trans>
          </span>
        </div>
        <div className='flex flex-col gap-2'>
          {spent
            .sort((a, b) => a.sort - b.sort)
            .map((spent, i) => (
              <div
                key={i}
                className='flex flex-col gap-1 border-2 p-1.5 rounded-md'
              >
                <div className='flex items-center gap-2'>
                  <Badge className='max-w-[100px]'>
                    <span className='truncate'>{spent.badge}</span>
                  </Badge>
                  <span>{spent.label}</span>
                </div>
                <div className='flex items-center gap-1'>
                  <BoxIcon className='w-4 h-4' />
                  <div className='truncate text-neutral-600 dark:text-neutral-400'>
                    {spent.coinId}
                  </div>
                </div>
              </div>
            ))}
        </div>
      </div>
      <div className='flex flex-col gap-2 w-full font-medium text-left text-neutral-900 dark:text-neutral-200 bg-neutral-100 dark:bg-neutral-900 p-2 rounded-md'>
        <div className='flex items-center gap-2 text-lg'>
          <BadgePlus className='w-6 h-6' />
          <span>
            <Trans>Transaction Output</Trans>
          </span>
        </div>
        <div className='flex flex-col gap-2'>
          {!BigNumber(summary.fee || 0).isZero() && (
            <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
              <div className='flex items-center gap-2'>
                <Badge>
                  <Trans>Fee</Trans>
                </Badge>
                <span>
                  {toDecimal(summary.fee, walletState.sync.unit.decimals)}{' '}
                  {walletState.sync.unit.ticker}
                </span>
              </div>
            </div>
          )}
          {created
            .sort((a, b) => a.sort - b.sort)
            .map((created, i) => (
              <div
                key={i}
                className='flex flex-col gap-1 border-2 p-1.5 rounded-md'
              >
                <div className='flex items-center gap-2'>
                  <Badge className='max-w-[100px]'>
                    <span className='truncate'>{created.badge}</span>
                  </Badge>
                  <span>{created.label}</span>
                </div>
                <div className='flex items-center gap-1'>
                  <ForwardIcon className='w-4 h-4' />
                  <div className='truncate text-neutral-600 dark:text-neutral-400'>
                    {created.address}
                  </div>
                </div>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}

export interface CalculatedTransaction {
  spent: SpentCoin[];
  created: CreatedCoin[];
}

export function calculateTransaction(
  xch: Unit,
  summary: TransactionSummary,
): CalculatedTransaction {
  const spent: SpentCoin[] = [];
  const created: CreatedCoin[] = [];

  for (const input of summary.inputs || []) {
    if (input.type === 'xch') {
      spent.push({
        badge: 'Chia',
        label: `${formatNumber({
          value: fromMojos(input.amount, xch.decimals),
          minimumFractionDigits: 0,
          maximumFractionDigits: xch.decimals,
        })} ${xch.ticker}`,
        coinId: input.coin_id,
        sort: 1,
      });

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        created.push({
          badge: 'Chia',
          label: `${formatNumber({
            value: fromMojos(output.amount, xch.decimals),
            minimumFractionDigits: 0,
            maximumFractionDigits: xch.decimals,
          })} ${xch.ticker}`,
          address: output.burning
            ? t`Permanently Burned`
            : output.receiving
              ? t`You`
              : output.address,
          sort: 1,
        });
      }
    }

    if (input.type === 'cat') {
      const ticker = input.ticker || 'CAT';

      spent.push({
        badge: `CAT ${input.name || input.asset_id}`,
        label: `${formatNumber({
          value: fromMojos(input.amount, 3),
          minimumFractionDigits: 0,
          maximumFractionDigits: 3,
        })} ${ticker}`,
        coinId: input.coin_id,
        sort: 2,
      });

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        created.push({
          badge: `CAT ${input.name || input.asset_id}`,
          label: `${formatNumber({
            value: fromMojos(output.amount, 3),
            minimumFractionDigits: 0,
            maximumFractionDigits: 3,
          })} ${ticker}`,
          address: output.burning
            ? t`Permanently Burned`
            : output.receiving
              ? t`You`
              : output.address,
          sort: 2,
        });
      }
    }

    if (input.type === 'did') {
      if (
        !summary.inputs
          .map((i) => i.outputs)
          .flat()
          .find((o) => o.coin_id === input.coin_id)
      ) {
        spent.push({
          badge: t`Profile`,
          label: input.name || t`Unnamed`,
          coinId: input.coin_id,
          sort: 3,
        });
      }

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).eq(1)) {
          created.push({
            badge: t`Profile`,
            label: input.name || t`Unnamed`,
            address: output.burning
              ? t`Permanently Burned`
              : output.receiving
                ? t`You`
                : output.address,
            sort: 3,
          });
        }
      }
    }

    if (input.type === 'nft') {
      if (
        !summary.inputs
          .map((i) => i.outputs)
          .flat()
          .find((o) => o.coin_id === input.coin_id)
      ) {
        spent.push({
          badge: 'NFT',
          label: input.name || t`Unknown`,
          coinId: input.coin_id,
          sort: 4,
        });
      }

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).isEqualTo(1)) {
          created.push({
            badge: 'NFT',
            label: input.name || t`Unknown`,
            address: output.burning
              ? t`Permanently Burned`
              : output.receiving
                ? t`You`
                : output.address,
            sort: 4,
          });
        }
      }
    }
  }
  return { spent, created };
}
