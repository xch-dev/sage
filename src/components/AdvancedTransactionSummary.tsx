import { fromMojos, toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { BadgeMinus, BadgePlus, BoxIcon, ForwardIcon } from 'lucide-react';
import { TransactionSummary, Unit } from '../bindings';
import { formatNumber } from '../i18n';
import { AddressLabels } from './AddressLabels';
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
  /** Identifies the asset: XCH, a CAT's asset ID, or a singleton's ID. */
  assetKey: string;
  /** Raw amount and display unit, for fungible assets (XCH and CATs) only. */
  fungible?: FungibleAmount;
}

interface FungibleAmount {
  mojos: BigNumber;
  precision: number;
  ticker: string;
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
                  <AddressLabels address={created.address} compact />
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

  const inputs = summary.inputs || [];
  const inputCoinIds = new Set(inputs.map((input) => input.coin_id));
  const outputCoinIds = new Set(
    inputs.flatMap((input) => input.outputs.map((output) => output.coin_id)),
  );

  for (const input of inputs) {
    const isEphemeral = outputCoinIds.has(input.coin_id);

    if (input.asset && !input.asset.asset_id) {
      if (!isEphemeral) {
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
      }

      for (const output of input.outputs) {
        if (inputCoinIds.has(output.coin_id)) {
          continue;
        }

        const fungible = {
          mojos: BigNumber(output.amount),
          precision: xch.precision,
          ticker: xch.ticker,
        };

        created.push({
          badge: 'Chia',
          label: formatFungibleAmount(fungible),
          assetKey: 'xch',
          fungible,
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

      if (!isEphemeral) {
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
      }

      for (const output of input.outputs) {
        if (inputCoinIds.has(output.coin_id)) {
          continue;
        }

        const fungible = {
          mojos: BigNumber(output.amount),
          precision: 3,
          ticker,
        };

        created.push({
          badge: `CAT ${input.asset.name ?? input.asset.ticker ?? input.asset.asset_id}`,
          label: formatFungibleAmount(fungible),
          assetKey: `cat:${input.asset.asset_id}`,
          fungible,
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
      if (!isEphemeral) {
        spent.push({
          badge: t`Profile`,
          label: input.asset.name ?? t`Unnamed`,
          coinId: input.coin_id,
          sort: 3,
        });
      }

      for (const output of input.outputs) {
        if (inputCoinIds.has(output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).eq(1)) {
          created.push({
            badge: t`Profile`,
            label: input.asset.name ?? t`Unnamed`,
            assetKey: `did:${input.asset.asset_id ?? input.coin_id}`,
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
      if (!isEphemeral) {
        spent.push({
          badge: 'NFT',
          label: input.asset.name ?? t`Unknown`,
          coinId: input.coin_id,
          sort: 4,
        });
      }

      for (const output of input.outputs) {
        if (inputCoinIds.has(output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).isEqualTo(1)) {
          created.push({
            badge: 'NFT',
            label: input.asset.name ?? t`Unknown`,
            assetKey: `nft:${input.asset.asset_id ?? input.coin_id}`,
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
      if (!isEphemeral) {
        spent.push({
          badge: 'Option',
          label: input.asset.name ?? t`Untitled`,
          coinId: input.coin_id,
          sort: 5,
        });
      }

      for (const output of input.outputs) {
        if (inputCoinIds.has(output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).isEqualTo(1)) {
          created.push({
            badge: 'Option',
            label: input.asset.name ?? t`Untitled`,
            assetKey: `option:${input.asset.asset_id ?? input.coin_id}`,
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

function formatFungibleAmount({ mojos, precision, ticker }: FungibleAmount) {
  return `${formatNumber({
    value: fromMojos(mojos, precision),
    minimumFractionDigits: 0,
    maximumFractionDigits: precision,
  })} ${ticker}`;
}

export interface SentAssetGroup {
  key: string;
  badge: string;
  /** The formatted total for fungible assets, or the asset's label otherwise. */
  total: string;
  /** Each recipient once, in the order they first appear. */
  recipients: string[];
}

/**
 * Groups created coins by asset for the confirmation summary. Fungible totals
 * are summed from the raw amounts of every output, including repeated outputs
 * to the same address, and only the final total is formatted.
 */
export function groupSentAssets(created: CreatedCoin[]): SentAssetGroup[] {
  const groups = new Map<
    string,
    { coin: CreatedCoin; mojos: BigNumber; recipients: Set<string> }
  >();

  for (const coin of [...created].sort((a, b) => a.sort - b.sort)) {
    const group = groups.get(coin.assetKey);

    if (group) {
      group.recipients.add(coin.address);
      if (coin.fungible) group.mojos = group.mojos.plus(coin.fungible.mojos);
    } else {
      groups.set(coin.assetKey, {
        coin,
        mojos: coin.fungible?.mojos ?? BigNumber(0),
        recipients: new Set([coin.address]),
      });
    }
  }

  return [...groups].map(([key, { coin, mojos, recipients }]) => ({
    key,
    badge: coin.badge,
    total: coin.fungible
      ? formatFungibleAmount({ ...coin.fungible, mojos })
      : coin.label,
    recipients: [...recipients],
  }));
}
