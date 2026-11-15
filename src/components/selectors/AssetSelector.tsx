import { TokenRecord, commands } from '@/bindings';
import { NftSelector } from '@/components/selectors/NftSelector';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { useRequestedNft } from '@/hooks/useRequestedNft';
import { optionsEnabled } from '@/lib/features';
import { fromMojos, toDecimal } from '@/lib/utils';
import { Assets } from '@/state';
import BigNumber from 'bignumber.js';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AlertTriangle, PlusIcon } from 'lucide-react';
import { ReactNode, useCallback, useEffect, useState } from 'react';
import { NumberFormat } from '../NumberFormat';
import {
  OfferAssetRow,
  OfferAssetRowFooter,
  RequestedNftPreview,
} from './OfferAssetRow';
import { OptionSelector } from './OptionSelector';

interface AssetSelectorProps {
  offering?: boolean;
  prefix: string;
  assets: Assets;
  setAssets: (value: Assets) => void;
  /** Shown in place of the asset rows while nothing has been added. */
  emptyMessage?: ReactNode;
  fee?: string;
}

export function AssetSelector({
  offering,
  prefix,
  assets,
  setAssets,
  emptyMessage,
  fee,
}: AssetSelectorProps) {
  const [ownedTokens, setOwnedTokens] = useState<TokenRecord[]>([]);
  const [tokenIds, setTokenIds] = useState<number[]>([]);
  const [nftIds, setNftIds] = useState<number[]>([]);
  const [optionIds, setOptionIds] = useState<number[]>([]);
  const [assetsInOffers, setAssetsInOffers] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!offering) return;
    Promise.all([commands.getCats({}), commands.getToken({ asset_id: null })])
      .then(([data, xch]) =>
        setOwnedTokens([...(xch.token ? [xch.token] : []), ...data.cats]),
      )
      .catch(console.error);
  }, [offering]);

  // Check which assets are part of open offers
  useEffect(() => {
    if (!offering) return;

    const checkAssetsInOffers = async () => {
      const assetsToCheck = [
        ...assets.nfts.filter((nft) => nft),
        ...assets.options.filter((option) => option),
      ];

      const newAssetsInOffers = new Set<string>();

      for (const assetId of assetsToCheck) {
        if (assetId) {
          try {
            const response = await commands.getOffersForAsset({
              asset_id: assetId,
            });
            if (response.offers.some((offer) => offer.status === 'active')) {
              newAssetsInOffers.add(assetId);
            }
          } catch (error) {
            console.error(
              `Failed to check offers for asset ${assetId}:`,
              error,
            );
          }
        }
      }

      setAssetsInOffers(newAssetsInOffers);
    };

    checkAssetsInOffers();
  }, [offering, assets.nfts, assets.options]);

  const generateId = useCallback(() => Date.now() + Math.random(), []);

  const addToken = () => {
    const newId = generateId();
    setTokenIds([...tokenIds, newId]);
    setAssets({
      ...assets,
      tokens: [...assets.tokens, { asset_id: '', amount: '' }],
    });
  };

  const addNft = () => {
    const newId = generateId();
    setNftIds([...nftIds, newId]);
    setAssets({
      ...assets,
      nfts: [...assets.nfts, ''],
    });
  };

  const addOption = () => {
    const newId = generateId();
    setOptionIds([...optionIds, newId]);
    setAssets({
      ...assets,
      options: [...assets.options, ''],
    });
  };

  const updateToken = (
    index: number,
    field: 'asset_id' | 'amount',
    value: string | null,
  ) => {
    const newTokens = [...assets.tokens];
    newTokens[index] = { ...newTokens[index], [field]: value };
    setAssets({ ...assets, tokens: newTokens });
  };

  const removeToken = (index: number) => {
    const newTokens = assets.tokens.filter((_, i) => i !== index);
    const newTokenIds = tokenIds.filter((_, i) => i !== index);
    setTokenIds(newTokenIds);
    setAssets({ ...assets, tokens: newTokens });
  };

  const updateNft = (index: number, value: string | null) => {
    const newNfts = [...assets.nfts];
    newNfts[index] = value || '';
    setAssets({ ...assets, nfts: newNfts });
  };

  const removeNft = (index: number) => {
    const newNfts = assets.nfts.filter((_, i) => i !== index);
    const newNftIds = nftIds.filter((_, i) => i !== index);
    setNftIds(newNftIds);
    setAssets({ ...assets, nfts: newNfts });
  };

  const updateOption = (index: number, value: string) => {
    const newOptions = [...assets.options];
    newOptions[index] = value;
    setAssets({ ...assets, options: newOptions });
  };

  const removeOption = (index: number) => {
    const newOptions = assets.options.filter((_, i) => i !== index);
    const newOptionIds = optionIds.filter((_, i) => i !== index);
    setOptionIds(newOptionIds);
    setAssets({ ...assets, options: newOptions });
  };

  // Initialize IDs if they don't exist
  useEffect(() => {
    if (tokenIds.length !== assets.tokens.length) {
      const newTokenIds = Array.from({ length: assets.tokens.length }, () =>
        generateId(),
      );
      setTokenIds(newTokenIds);
    }
  }, [assets.tokens.length, tokenIds.length, generateId]);

  useEffect(() => {
    if (nftIds.length !== assets.nfts.length) {
      const newNftIds = Array.from({ length: assets.nfts.length }, () =>
        generateId(),
      );
      setNftIds(newNftIds);
    }
  }, [assets.nfts.length, nftIds.length, generateId]);

  useEffect(() => {
    if (optionIds.length !== assets.options.length) {
      const newOptionIds = Array.from({ length: assets.options.length }, () =>
        generateId(),
      );
      setOptionIds(newOptionIds);
    }
  }, [assets.options.length, optionIds.length, generateId]);

  return (
    <div className='flex flex-col gap-4'>
      {assets.tokens.length === 0 &&
        assets.nfts.length === 0 &&
        assets.options.length === 0 &&
        emptyMessage && (
          <div className='flex min-h-14 items-center justify-center rounded-lg border border-dashed border-border px-4 text-center text-sm text-muted-foreground'>
            {emptyMessage}
          </div>
        )}

      {assets.tokens.map(({ asset_id: assetId, amount }, i) => {
        const ownedToken = offering
          ? ownedTokens.find((t) => t.asset_id === assetId)
          : undefined;
        const maxValue = ownedToken
          ? BigNumber.max(
              0,
              BigNumber(
                toDecimal(ownedToken.selectable_balance, ownedToken.precision),
              ).minus(assetId === null ? fee || 0 : 0),
            )
          : undefined;
        const overBalance =
          maxValue !== undefined &&
          amount !== '' &&
          BigNumber(amount.toString()).gt(maxValue);

        return (
          <OfferAssetRow
            key={tokenIds[i] || `token-${i}`}
            onRemove={() => removeToken(i)}
            removeLabel={t`Remove token`}
            tone={overBalance ? 'error' : undefined}
            footer={
              ownedToken && (
                <OfferAssetRowFooter tone={overBalance ? 'error' : undefined}>
                  {overBalance ? (
                    assetId === null ? (
                      <Trans>
                        More than your spendable balance after the network fee
                      </Trans>
                    ) : (
                      <Trans>More than your spendable balance</Trans>
                    )
                  ) : (
                    <span className='truncate'>
                      <SpendableBalance token={ownedToken} />
                    </span>
                  )}
                </OfferAssetRowFooter>
              )
            }
          >
            <div className='min-w-0 flex-1'>
              <TokenAmountInput
                id={`${prefix}-cat-${i}-amount`}
                aria-label={t`Amount`}
                className='h-12 rounded-none border-0 bg-transparent shadow-none focus-visible:ring-0'
                placeholder={t`Amount`}
                value={amount}
                onValueChange={(values) =>
                  updateToken(i, 'amount', values.value)
                }
                precision={assetId === null ? 12 : 3}
                hideMaxButton
              />
            </div>
            {offering && (
              <Button
                type='button'
                variant='ghost'
                className='h-12 shrink-0 rounded-none px-3 text-xs'
                disabled={!maxValue || maxValue.isZero()}
                onClick={() =>
                  maxValue && updateToken(i, 'amount', maxValue.toFixed())
                }
                aria-label={t`Use maximum spendable balance`}
                title={t`Use maximum spendable balance`}
              >
                <Trans>Max</Trans>
              </Button>
            )}
            <div className='min-w-0 flex-1'>
              <TokenSelector
                value={assetId}
                onChange={(newAssetId) => {
                  const newTokens = [...assets.tokens];
                  newTokens[i] = {
                    ...newTokens[i],
                    asset_id: newAssetId,
                    amount: '',
                  };
                  setAssets({ ...assets, tokens: newTokens });
                }}
                disabled={assets.tokens
                  .filter((t, idx) => t.asset_id !== '' && idx !== i)
                  .map((t) => t.asset_id)}
                className={rowSelectClassName}
                hideZeroBalance={offering === true}
                showAllCats={offering !== true}
                includeXch={true}
              />
            </div>
          </OfferAssetRow>
        );
      })}

      {assets.nfts.map((nft, i) =>
        offering ? (
          <OfferAssetRow
            key={nftIds[i] || `nft-${i}`}
            onRemove={() => removeNft(i)}
            removeLabel={t`Remove NFT`}
            tone={nft && assetsInOffers.has(nft) ? 'warning' : undefined}
            footer={
              nft &&
              assetsInOffers.has(nft) && (
                <OfferAssetRowFooter tone='warning'>
                  <AlertTriangle className='h-3 w-3' aria-hidden='true' />
                  <Trans>This NFT is part of an open offer</Trans>
                </OfferAssetRowFooter>
              )
            }
          >
            <div className='min-w-0 flex-1'>
              <NftSelector
                value={nft || null}
                onChange={(nftId) => updateNft(i, nftId || '')}
                disabled={assets.nfts.filter(
                  (id, idx) => id !== '' && idx !== i,
                )}
                className={rowSelectClassName}
              />
            </div>
          </OfferAssetRow>
        ) : (
          <RequestedNftRow
            key={nftIds[i] || `nft-${i}`}
            value={nft}
            onChange={(value) => updateNft(i, value)}
            onRemove={() => removeNft(i)}
          />
        ),
      )}

      {assets.options.map((option, i) => (
        <OfferAssetRow
          key={optionIds[i] || `option-${i}`}
          onRemove={() => removeOption(i)}
          removeLabel={t`Remove option`}
          tone={
            offering && option && assetsInOffers.has(option)
              ? 'warning'
              : undefined
          }
          footer={
            offering &&
            option &&
            assetsInOffers.has(option) && (
              <OfferAssetRowFooter tone='warning'>
                <AlertTriangle className='h-3 w-3' aria-hidden='true' />
                <Trans>This option is part of an open offer</Trans>
              </OfferAssetRowFooter>
            )
          }
        >
          <div className='min-w-0 flex-1'>
            {offering ? (
              <OptionSelector
                value={option || null}
                onChange={(optionId) => updateOption(i, optionId || '')}
                disabled={assets.options.filter(
                  (id, idx) => id !== '' && idx !== i,
                )}
                className={rowSelectClassName}
              />
            ) : (
              <Input
                aria-label={t`Option ID`}
                className={rowInputClassName}
                placeholder={t`Option ID`}
                value={option}
                onChange={(e) => updateOption(i, e.target.value)}
              />
            )}
          </div>
        </OfferAssetRow>
      ))}

      <div className='flex flex-wrap gap-2'>
        <Button variant='outline' size='sm' onClick={addToken}>
          <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
          <Trans>Token</Trans>
        </Button>

        <Button variant='outline' size='sm' onClick={addNft}>
          <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
          <Trans>NFT</Trans>
        </Button>

        {optionsEnabled && (
          <Button variant='outline' size='sm' onClick={addOption}>
            <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
            <Trans>Option</Trans>
          </Button>
        )}
      </div>
    </div>
  );
}

// The selectors and inputs sit inside an OfferAssetRow, which draws the border.
const rowSelectClassName =
  'h-12 rounded-none border-0 bg-transparent shadow-none';
const rowInputClassName =
  'h-12 rounded-none border-0 bg-transparent font-mono text-sm shadow-none focus-visible:ring-0';

function SpendableBalance({ token }: { token: TokenRecord }) {
  const ticker = token.ticker ?? '';

  return (
    <Trans>
      <NumberFormat
        value={fromMojos(token.selectable_balance, token.precision)}
        minimumFractionDigits={0}
        maximumFractionDigits={token.precision}
      />{' '}
      {ticker} spendable
    </Trans>
  );
}

interface RequestedNftRowProps {
  value: string;
  onChange: (value: string) => void;
  onRemove: () => void;
}

function RequestedNftRow({ value, onChange, onRemove }: RequestedNftRowProps) {
  const lookup = useRequestedNft(value);

  return (
    <OfferAssetRow
      onRemove={onRemove}
      removeLabel={t`Remove NFT`}
      tone={
        lookup.status === 'invalid'
          ? 'error'
          : lookup.status === 'missing'
            ? 'warning'
            : undefined
      }
      footer={<RequestedNftPreview lookup={lookup} />}
    >
      <div className='min-w-0 flex-1'>
        <Input
          aria-label={t`NFT ID`}
          className={rowInputClassName}
          placeholder={t`NFT ID (nft1…)`}
          autoComplete='off'
          spellCheck={false}
          value={value}
          // Pasted IDs often bring a space or newline, which the offer
          // wouldn't accept.
          onChange={(e) => onChange(e.target.value.trim())}
        />
      </div>
    </OfferAssetRow>
  );
}
