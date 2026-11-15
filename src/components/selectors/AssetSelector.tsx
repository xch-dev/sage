import { TokenRecord, commands } from '@/bindings';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { useOwnedNft } from '@/hooks/useOwnedNft';
import { useOwnedOption } from '@/hooks/useOwnedOption';
import { useRequestedNft } from '@/hooks/useRequestedNft';
import { optionsEnabled } from '@/lib/features';
import {
  cn,
  formatAddress,
  fromMojos,
  isValidAddress,
  toDecimal,
} from '@/lib/utils';
import { Assets } from '@/state';
import BigNumber from 'bignumber.js';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  AlertTriangle,
  ArrowUpToLine,
  ChevronDown,
  PlusIcon,
} from 'lucide-react';
import { ReactNode, useCallback, useEffect, useState } from 'react';
import { NumberFormat } from '../NumberFormat';
import { NftPicker, NftSummary } from './NftPicker';
import {
  OfferAssetRow,
  OfferAssetRowFooter,
  RequestedNftPreview,
} from './OfferAssetRow';
import { OptionPicker, OptionSummary } from './OptionPicker';
import { TokenIcon, TokenPicker } from './TokenPicker';

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
  const [tokens, setTokens] = useState<TokenRecord[]>([]);
  const [tokenIds, setTokenIds] = useState<number[]>([]);
  const [nftIds, setNftIds] = useState<number[]>([]);
  const [optionIds, setOptionIds] = useState<number[]>([]);
  const [assetsInOffers, setAssetsInOffers] = useState<Set<string>>(new Set());
  // The row that was just added, whose input should take focus.
  const [focusId, setFocusId] = useState<number | null>(null);

  // What you offer comes from your own tokens; you can request any known CAT.
  const loadTokens = useCallback(() => {
    Promise.all([
      offering ? commands.getCats({}) : commands.getAllCats({}),
      commands.getToken({ asset_id: null }),
    ])
      .then(([data, xch]) => {
        const cats = [...data.cats].sort((a, b) => {
          if (!a.name && !b.name) return 0;
          if (!a.name) return 1;
          if (!b.name) return -1;
          return a.name.localeCompare(b.name);
        });
        setTokens([...(xch.token ? [xch.token] : []), ...cats]);
      })
      .catch(console.error);
  }, [offering]);

  useEffect(loadTokens, [loadTokens]);

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

  const addToken = (assetId: string | null) => {
    const newId = generateId();
    setTokenIds([...tokenIds, newId]);
    setFocusId(newId);
    setAssets({
      ...assets,
      tokens: [...assets.tokens, { asset_id: assetId, amount: '' }],
    });
  };

  const toggleOfferedNft = (nftId: string) => {
    const index = assets.nfts.indexOf(nftId);
    if (index !== -1) {
      removeNft(index);
      return;
    }
    setNftIds([...nftIds, generateId()]);
    setAssets({ ...assets, nfts: [...assets.nfts, nftId] });
  };

  const addNft = () => {
    const newId = generateId();
    setNftIds([...nftIds, newId]);
    setFocusId(newId);
    setAssets({
      ...assets,
      nfts: [...assets.nfts, ''],
    });
  };

  const toggleOfferedOption = (optionId: string) => {
    const index = assets.options.indexOf(optionId);
    if (index !== -1) {
      removeOption(index);
      return;
    }
    setOptionIds([...optionIds, generateId()]);
    setAssets({ ...assets, options: [...assets.options, optionId] });
  };

  const addOption = () => {
    const newId = generateId();
    setOptionIds([...optionIds, newId]);
    setFocusId(newId);
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
        const token = tokens.find((t) => t.asset_id === assetId);
        const ownedToken = offering ? token : undefined;
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
            <TokenPicker
              tokens={tokens}
              offering={offering}
              value={assetId}
              taken={assets.tokens.map((t) => t.asset_id)}
              onOpen={loadTokens}
              onSelect={(newAssetId) => {
                if (newAssetId === assetId) return;
                const newTokens = [...assets.tokens];
                newTokens[i] = { asset_id: newAssetId, amount: '' };
                setAssets({ ...assets, tokens: newTokens });
              }}
            >
              <Button
                type='button'
                variant='ghost'
                className='h-12 max-w-[45%] shrink-0 gap-2 rounded-none px-3 font-normal'
                aria-label={t`Change token`}
              >
                <TokenIcon token={token} size='sm' />
                <span className='truncate'>{tokenLabel(token, assetId)}</span>
                <ChevronDown
                  className='h-4 w-4 shrink-0 opacity-50'
                  aria-hidden='true'
                />
              </Button>
            </TokenPicker>
            <div className='relative min-w-0 flex-1'>
              <TokenAmountInput
                id={`${prefix}-cat-${i}-amount`}
                aria-label={t`Amount`}
                className={cn(
                  'h-12 rounded-none border-0 bg-transparent shadow-none focus-visible:ring-0',
                  offering && 'pr-11',
                )}
                placeholder={t`Amount`}
                value={amount}
                onValueChange={(values) =>
                  updateToken(i, 'amount', values.value)
                }
                precision={assetId === null ? 12 : 3}
                hideMaxButton
                autoFocus={tokenIds[i] === focusId}
              />
              {offering && (
                <Button
                  type='button'
                  variant='ghost'
                  size='icon'
                  className='absolute right-1.5 top-1/2 h-8 w-8 -translate-y-1/2 text-muted-foreground hover:text-foreground'
                  disabled={!maxValue || maxValue.isZero()}
                  onClick={() =>
                    maxValue && updateToken(i, 'amount', maxValue.toFixed())
                  }
                  aria-label={t`Use maximum spendable balance`}
                  title={t`Use maximum spendable balance`}
                >
                  <ArrowUpToLine className='h-4 w-4' aria-hidden='true' />
                </Button>
              )}
            </div>
          </OfferAssetRow>
        );
      })}

      {assets.nfts.map((nft, i) =>
        offering ? (
          // Offered NFTs are only added from the picker, so are never blank.
          nft && (
            <OfferedNftRow
              key={nftIds[i] || `nft-${i}`}
              nftId={nft}
              inOpenOffer={assetsInOffers.has(nft)}
              onRemove={() => removeNft(i)}
            />
          )
        ) : (
          <RequestedNftRow
            key={nftIds[i] || `nft-${i}`}
            value={nft}
            onChange={(value) => updateNft(i, value)}
            onRemove={() => removeNft(i)}
            autoFocus={nftIds[i] === focusId}
          />
        ),
      )}

      {assets.options.map((option, i) =>
        offering ? (
          // Offered options are only added from the picker, so are never blank.
          option && (
            <OfferedOptionRow
              key={optionIds[i] || `option-${i}`}
              optionId={option}
              inOpenOffer={assetsInOffers.has(option)}
              onRemove={() => removeOption(i)}
            />
          )
        ) : (
          <RequestedOptionRow
            key={optionIds[i] || `option-${i}`}
            value={option}
            onChange={(value) => updateOption(i, value)}
            onRemove={() => removeOption(i)}
            autoFocus={optionIds[i] === focusId}
          />
        ),
      )}

      <div className='flex flex-wrap gap-2'>
        <TokenPicker
          tokens={tokens}
          offering={offering}
          taken={assets.tokens.map((t) => t.asset_id)}
          onOpen={loadTokens}
          onSelect={addToken}
        >
          <Button variant='outline' size='sm'>
            <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
            <Trans>Token</Trans>
          </Button>
        </TokenPicker>

        {offering ? (
          <NftPicker
            selected={assets.nfts.filter((n) => n)}
            onToggle={toggleOfferedNft}
          >
            <Button variant='outline' size='sm'>
              <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
              <Trans>NFT</Trans>
            </Button>
          </NftPicker>
        ) : (
          <Button variant='outline' size='sm' onClick={addNft}>
            <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
            <Trans>NFT</Trans>
          </Button>
        )}

        {optionsEnabled &&
          (offering ? (
            <OptionPicker
              selected={assets.options.filter((o) => o)}
              onToggle={toggleOfferedOption}
            >
              <Button variant='outline' size='sm'>
                <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
                <Trans>Option</Trans>
              </Button>
            </OptionPicker>
          ) : (
            <Button variant='outline' size='sm' onClick={addOption}>
              <PlusIcon className='mr-1 h-3 w-3' aria-hidden='true' />
              <Trans>Option</Trans>
            </Button>
          ))}
      </div>
    </div>
  );
}

// The inputs sit inside an OfferAssetRow, which draws the border.
const rowInputClassName =
  'h-12 rounded-none border-0 bg-transparent font-mono text-sm shadow-none focus-visible:ring-0';

function tokenLabel(token: TokenRecord | undefined, assetId: string | null) {
  if (token) return token.ticker ?? token.name ?? t`Unknown token`;
  if (assetId) return formatAddress(assetId, 4);
  return t`Choose token`;
}

interface OfferedNftRowProps {
  nftId: string;
  inOpenOffer: boolean;
  onRemove: () => void;
}

function OfferedNftRow({ nftId, inOpenOffer, onRemove }: OfferedNftRowProps) {
  const { nft, thumbnail } = useOwnedNft(nftId);

  return (
    <OfferAssetRow
      onRemove={onRemove}
      removeLabel={t`Remove NFT`}
      tone={inOpenOffer ? 'warning' : undefined}
      footer={
        inOpenOffer && (
          <OfferAssetRowFooter tone='warning'>
            <AlertTriangle className='h-3 w-3' aria-hidden='true' />
            <Trans>This NFT is part of an open offer</Trans>
          </OfferAssetRowFooter>
        )
      }
    >
      <div className='flex h-12 min-w-0 flex-1 items-center gap-3 px-3'>
        <NftSummary nftId={nftId} nft={nft} thumbnail={thumbnail} />
      </div>
    </OfferAssetRow>
  );
}

interface OfferedOptionRowProps {
  optionId: string;
  inOpenOffer: boolean;
  onRemove: () => void;
}

function OfferedOptionRow({
  optionId,
  inOpenOffer,
  onRemove,
}: OfferedOptionRowProps) {
  const option = useOwnedOption(optionId);

  return (
    <OfferAssetRow
      onRemove={onRemove}
      removeLabel={t`Remove option`}
      tone={inOpenOffer ? 'warning' : undefined}
      footer={
        inOpenOffer && (
          <OfferAssetRowFooter tone='warning'>
            <AlertTriangle className='h-3 w-3' aria-hidden='true' />
            <Trans>This option is part of an open offer</Trans>
          </OfferAssetRowFooter>
        )
      }
    >
      <div className='flex h-12 min-w-0 flex-1 items-center gap-3 px-3'>
        <OptionSummary option={option} />
      </div>
    </OfferAssetRow>
  );
}

interface RequestedOptionRowProps {
  value: string;
  onChange: (value: string) => void;
  onRemove: () => void;
  autoFocus?: boolean;
}

function RequestedOptionRow({
  value,
  onChange,
  onRemove,
  autoFocus,
}: RequestedOptionRowProps) {
  const invalid = value !== '' && !isValidAddress(value, 'option');

  return (
    <OfferAssetRow
      onRemove={onRemove}
      removeLabel={t`Remove option`}
      tone={invalid ? 'error' : undefined}
      footer={
        invalid && (
          <OfferAssetRowFooter tone='error'>
            <Trans>This isn&apos;t a valid option ID.</Trans>
          </OfferAssetRowFooter>
        )
      }
    >
      <div className='min-w-0 flex-1'>
        <Input
          aria-label={t`Option ID`}
          className={rowInputClassName}
          placeholder={t`Option ID (option1…)`}
          autoComplete='off'
          spellCheck={false}
          value={value}
          // Pasted IDs often bring a space or newline, which the offer
          // wouldn't accept.
          onChange={(e) => onChange(e.target.value.trim())}
          autoFocus={autoFocus}
        />
      </div>
    </OfferAssetRow>
  );
}

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
  autoFocus?: boolean;
}

function RequestedNftRow({
  value,
  onChange,
  onRemove,
  autoFocus,
}: RequestedNftRowProps) {
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
          autoFocus={autoFocus}
        />
      </div>
    </OfferAssetRow>
  );
}
