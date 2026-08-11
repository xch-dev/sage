import { fromMojos, toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { BadgeMinus, BadgePlus, BoxIcon, ForwardIcon } from 'lucide-react';
import { TransactionSummary, Unit } from '../bindings';
import { formatNumber } from '../i18n';
import { Badge } from './ui/badge';

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
      <div className='flex flex-col gap-2 w-full font-medium text-left text-card-foreground bg-card border border-border shadow-card p-2 rounded-lg'>
        <div className='flex items-center gap-2 text-lg'>
          <BadgeMinus className='w-6 h-6' aria-hidden='true' />
          <span>
            <Trans>Spent Coins</Trans>
          </span>
        </div>
        <div className='flex flex-col gap-2'>
          {spent
            .sort((a, b) => a.sort - b.sort)
            .map((spent) => (
              <div
                key={spent.coinId}
                className='flex flex-col gap-1 border border-border p-1.5 rounded-md bg-card/50'
              >
                <div className='flex items-center gap-2'>
                  <Badge className='max-w-[100px]'>
                    <span className='truncate'>{spent.badge}</span>
                  </Badge>
                  <span>{spent.label}</span>
                </div>
                <div className='flex items-center gap-1'>
                  <BoxIcon
                    className='w-4 h-4 text-muted-foreground'
                    aria-hidden='true'
                  />
                  <div className='truncate text-muted-foreground text-sm'>
                    {spent.coinId}
                  </div>
                </div>
              </div>
            ))}
        </div>
      </div>
      <div className='flex flex-col gap-2 w-full font-medium text-left text-card-foreground bg-card border border-border shadow-card p-2 rounded-lg'>
        <div className='flex items-center gap-2 text-lg'>
          <BadgePlus className='w-6 h-6' aria-hidden='true' />
          <span>
            <Trans>Transaction Output</Trans>
          </span>
        </div>
        <div className='flex flex-col gap-2'>
          {!BigNumber(summary.fee || 0).isZero() && (
            <div className='flex flex-col gap-1 border border-border p-1.5 rounded-md bg-card/50'>
              <div className='flex items-center gap-2'>
                <Badge>
                  <Trans>Fee</Trans>
                </Badge>
                <span>
                  {toDecimal(summary.fee, walletState.sync.unit.precision)}{' '}
                  {walletState.sync.unit.ticker}
                </span>
              </div>
            </div>
          )}
          {created
            .sort((a, b) => a.sort - b.sort)
            .map((created) => (
              <div
                key={created.label}
                className='flex flex-col gap-1 border border-border p-1.5 rounded-md bg-card/50'
              >
                <div className='flex items-center gap-2'>
                  <Badge className='max-w-[100px]'>
                    <span className='truncate'>{created.badge}</span>
                  </Badge>
                  <span>{created.label}</span>
                </div>
                <div className='flex items-center gap-1'>
                  <ForwardIcon
                    className='w-4 h-4 text-muted-foreground'
                    aria-hidden='true'
                  />
                  <div className='truncate text-muted-foreground text-sm'>
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
    if (input.asset && !input.asset.asset_id) {
      spent.push({
        badge: 'Chia',
        label: `${formatNumber({
          value: fromMojos(input.amount, xch.precision),
          minimumFractionDigits: 0,
          maximumFractionDigits: xch.precision,
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
            value: fromMojos(output.amount, xch.precision),
            minimumFractionDigits: 0,
            maximumFractionDigits: xch.precision,
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

    if (input.asset?.kind === 'token' && input.asset.asset_id) {
      const ticker = input.asset.ticker || 'CAT';

      spent.push({
        badge: `CAT ${input.asset.name ?? input.asset.ticker ?? input.asset.asset_id}`,
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
          badge: `CAT ${input.asset.name ?? input.asset.ticker ?? input.asset.asset_id}`,
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

    if (input.asset?.kind === 'did') {
      if (
        !summary.inputs
          .map((i) => i.outputs)
          .flat()
          .find((o) => o.coin_id === input.coin_id)
      ) {
        spent.push({
          badge: t`Profile`,
          label: input.asset.name ?? t`Unnamed`,
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
            label: input.asset.name ?? t`Unnamed`,
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

    if (input.asset?.kind === 'nft') {
      if (
        !summary.inputs
          .map((i) => i.outputs)
          .flat()
          .find((o) => o.coin_id === input.coin_id)
      ) {
        spent.push({
          badge: 'NFT',
          label: input.asset.name ?? t`Unknown`,
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
            label: input.asset.name ?? t`Unknown`,
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

    if (input.asset?.kind === 'option') {
      if (
        !summary.inputs
          .map((i) => i.outputs)
          .flat()
          .find((o) => o.coin_id === input.coin_id)
      ) {
        spent.push({
          badge: 'Option',
          label: input.asset.name ?? t`Untitled`,
          coinId: input.coin_id,
          sort: 5,
        });
      }

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).isEqualTo(1)) {
          created.push({
            badge: 'Option',
            label: input.asset.name ?? t`Untitled`,
            address: output.burning
              ? t`Permanently Burned`
              : output.receiving
                ? t`You`
                : output.address,
            sort: 5,
          });
        }
      }
    }
  }
  return { spent, created };
}
