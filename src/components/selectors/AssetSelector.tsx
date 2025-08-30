import { TokenRecord, commands } from '@/bindings';
import { NftSelector } from '@/components/selectors/NftSelector';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { toDecimal } from '@/lib/utils';
import { Assets } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  ArrowUpToLine,
  FilePenLine,
  HandCoins,
  ImageIcon,
  PlusIcon,
  TrashIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { OptionSelector } from './OptionSelector';

interface AssetSelectorProps {
  offering?: boolean;
  prefix: string;
  assets: Assets;
  setAssets: (value: Assets) => void;
  splitNftOffers?: boolean;
  setSplitNftOffers?: (value: boolean) => void;
}

export function AssetSelector({
  offering,
  prefix,
  assets,
  setAssets,
  splitNftOffers,
  setSplitNftOffers,
}: AssetSelectorProps) {
  const isIos = platform() === 'ios';

  const [ownedTokens, setOwnedTokens] = useState<TokenRecord[]>([]);
  const [tokenIds, setTokenIds] = useState<number[]>([]);
  const [nftIds, setNftIds] = useState<number[]>([]);
  const [optionIds, setOptionIds] = useState<number[]>([]);

  useEffect(() => {
    if (!offering) return;
    Promise.all([commands.getCats({}), commands.getToken({ asset_id: null })])
      .then(([data, xch]) =>
        setOwnedTokens([...(xch.token ? [xch.token] : []), ...data.cats]),
      )
      .catch(console.error);
  }, [offering]);

  // Generate unique IDs for new items
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

  const setMaxTokenAmount = (index: number, assetId: string | null) => {
    const token = ownedTokens.find((t) => t.asset_id === assetId);
    if (token) {
      updateToken(index, 'amount', toDecimal(token.balance, token.precision));
    }
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
    <>
      <div className='mt-4 flex gap-2 w-full items-center'>
        <Button variant='outline' className='flex-grow' onClick={addToken}>
          <PlusIcon className='mr-0.5 h-3 w-3' aria-hidden='true' />{' '}
          <Trans>Token</Trans>
        </Button>

        <Button variant='outline' className='flex-grow' onClick={addNft}>
          <PlusIcon className='mr-0.5 h-3 w-3' aria-hidden='true' />{' '}
          <Trans>NFT</Trans>
        </Button>

        {!isIos && (
          <Button variant='outline' className='flex-grow' onClick={addOption}>
            <PlusIcon className='mr-0.5 h-3 w-3' aria-hidden='true' />{' '}
            <Trans>Option</Trans>
          </Button>
        )}
      </div>

      {assets.tokens.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <HandCoins className='h-4 w-4' aria-hidden='true' />
            <span>Tokens</span>
          </Label>
          {assets.tokens.map(({ asset_id: assetId, amount }, i) => (
            <div
              key={tokenIds[i] || `token-${i}`}
              style={{
                zIndex:
                  assets.tokens.length -
                  i +
                  assets.nfts.length +
                  assets.options.length,
              }}
              className='flex h-14 mb-1 relative'
            >
              <TokenSelector
                value={assetId}
                onChange={(assetId) => updateToken(i, 'asset_id', assetId)}
                disabled={assets.tokens
                  .filter((t, idx) => t.asset_id !== '' && idx !== i)
                  .map((t) => t.asset_id)}
                className='rounded-r-none'
                hideZeroBalance={offering === true}
                showAllCats={offering !== true}
                includeXch={true}
              />
              <div className='flex flex-grow-0'>
                <TokenAmountInput
                  id={`${prefix}-cat-${i}-amount`}
                  className='border-l-0 z-10 rounded-l-none rounded-r-none w-[150px] h-12'
                  placeholder={t`Amount`}
                  value={amount}
                  onValueChange={(values) =>
                    updateToken(i, 'amount', values.value)
                  }
                />
                {offering && (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant='outline'
                          className='border-l-0 rounded-none h-12 px-2 text-xs'
                          onClick={() => setMaxTokenAmount(i, assetId)}
                        >
                          <ArrowUpToLine className='h-3 w-3 mr-1' />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <Trans>Use maximum balance</Trans>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                )}
                <Button
                  variant='outline'
                  className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                  onClick={() => removeToken(i)}
                >
                  <TrashIcon className='h-4 w-4' aria-hidden='true' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {assets.nfts.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <ImageIcon className='h-4 w-4' aria-hidden='true' />
            <span>NFTs</span>
          </Label>
          {offering && assets.nfts.filter((n) => n).length > 1 && (
            <div className='flex items-center gap-2 mt-1 mb-3'>
              <Switch
                id='split-offers'
                checked={splitNftOffers}
                onCheckedChange={setSplitNftOffers}
              />
              <Label htmlFor='split-offers' className='text-sm'>
                <Trans>Create individual offers for each NFT</Trans>
              </Label>
            </div>
          )}
          {assets.nfts.map((nft, i) => (
            <div
              key={nftIds[i] || `nft-${i}`}
              style={{ zIndex: assets.nfts.length - i + assets.options.length }}
              className='flex h-14 mb-1 relative'
            >
              {offering === true ? (
                <NftSelector
                  value={nft || null}
                  onChange={(nftId) => updateNft(i, nftId || '')}
                  disabled={assets.nfts.filter(
                    (id, idx) => id !== '' && idx !== i,
                  )}
                  className='rounded-r-none'
                />
              ) : (
                <Input
                  className='flex-grow rounded-r-none h-12 z-10'
                  placeholder='Enter NFT id'
                  value={nft}
                  onChange={(e) => updateNft(i, e.target.value)}
                />
              )}
              <Button
                variant='outline'
                className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                onClick={() => removeNft(i)}
              >
                <TrashIcon className='h-4 w-4' aria-hidden='true' />
              </Button>
            </div>
          ))}
        </div>
      )}

      {assets.options.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <FilePenLine className='h-4 w-4' aria-hidden='true' />
            <span>Options</span>
          </Label>
          {assets.options.map((option, i) => (
            <div
              key={optionIds[i] || `option-${i}`}
              style={{ zIndex: assets.options.length - i }}
              className='flex h-14 mb-1 relative'
            >
              {offering === true ? (
                <OptionSelector
                  value={option || null}
                  onChange={(optionId) => updateOption(i, optionId || '')}
                  disabled={assets.options.filter(
                    (id, idx) => id !== '' && idx !== i,
                  )}
                  className='rounded-r-none'
                />
              ) : (
                <Input
                  className='flex-grow rounded-r-none h-12 z-10'
                  placeholder='Enter option id'
                  value={option}
                  onChange={(e) => updateOption(i, e.target.value)}
                />
              )}
              <Button
                variant='outline'
                className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                onClick={() => removeOption(i)}
              >
                <TrashIcon className='h-4 w-4' aria-hidden='true' />
              </Button>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
