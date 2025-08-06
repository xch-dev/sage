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
import {
  ArrowUpToLine,
  FilePenLine,
  HandCoins,
  ImageIcon,
  PlusIcon,
  TrashIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
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
  const [ownedTokens, setOwnedTokens] = useState<TokenRecord[]>([]);

  useEffect(() => {
    if (!offering) return;
    Promise.all([commands.getCats({}), commands.getToken({ asset_id: null })])
      .then(([data, xch]) =>
        setOwnedTokens([...(xch.token ? [xch.token] : []), ...data.cats]),
      )
      .catch(console.error);
  }, [offering]);

  return (
    <>
      <div className='mt-4 flex gap-2 w-full items-center'>
        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setAssets({
              ...assets,
              tokens: [...assets.tokens, { asset_id: '', amount: '' }],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> <Trans>Token</Trans>
        </Button>

        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setAssets({
              ...assets,
              nfts: [...assets.nfts, ''],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> <Trans>NFT</Trans>
        </Button>

        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setAssets({
              ...assets,
              options: [...assets.options, ''],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> <Trans>Option</Trans>
        </Button>
      </div>

      {assets.tokens.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <HandCoins className='h-4 w-4' />
            <span>Tokens</span>
          </Label>
          {assets.tokens.map(({ asset_id: assetId, amount }, i) => (
            <div key={assetId} className='flex h-14 mb-1'>
              <TokenSelector
                value={assetId}
                onChange={(assetId) => {
                  const newTokens = [...assets.tokens];
                  newTokens[i] = { ...newTokens[i], asset_id: assetId };
                  setAssets({ ...assets, tokens: newTokens });
                }}
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
                  className='border-l-0 z-10 rounded-l-none rounded-r-none w-[100px] h-12'
                  placeholder={t`Amount`}
                  value={amount}
                  onValueChange={(values) => {
                    const newTokens = [...assets.tokens];
                    newTokens[i] = { ...newTokens[i], amount: values.value };
                    setAssets({ ...assets, tokens: newTokens });
                  }}
                />
                {offering && (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant='outline'
                          className='border-l-0 rounded-none h-12 px-2 text-xs'
                          onClick={() => {
                            const token = ownedTokens.find(
                              (t) => t.asset_id === assetId,
                            );
                            if (token) {
                              const newTokens = [...assets.tokens];
                              newTokens[i] = {
                                ...newTokens[i],
                                amount: toDecimal(
                                  token.balance,
                                  token.precision,
                                ),
                              };
                              setAssets({ ...assets, tokens: newTokens });
                            }
                          }}
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
                  onClick={() => {
                    const newTokens = [...assets.tokens];
                    newTokens.splice(i, 1);
                    setAssets({ ...assets, tokens: newTokens });
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {assets.nfts.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <ImageIcon className='h-4 w-4' />
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
            <div key={nft} className='flex h-14 z-20 mb-1'>
              {offering === true ? (
                <NftSelector
                  value={nft || null}
                  onChange={(nftId) => {
                    const newNfts = [...assets.nfts];
                    newNfts[i] = nftId || '';
                    setAssets({ ...assets, nfts: newNfts });
                  }}
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
                  onChange={(e) => {
                    const newNfts = [...assets.nfts];
                    newNfts[i] = e.target.value;
                    setAssets({ ...assets, nfts: newNfts });
                  }}
                />
              )}
              <Button
                variant='outline'
                className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                onClick={() => {
                  const newNfts = [...assets.nfts];
                  newNfts.splice(i, 1);
                  setAssets({ ...assets, nfts: newNfts });
                }}
              >
                <TrashIcon className='h-4 w-4' />
              </Button>
            </div>
          ))}
        </div>
      )}

      {assets.options.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <FilePenLine className='h-4 w-4' />
            <span>Options</span>
          </Label>
          {assets.options.map((option, i) => (
            <div key={option} className='flex h-14 z-20 mb-1'>
              {offering === true ? (
                <OptionSelector
                  value={option || null}
                  onChange={(optionId) => {
                    const newOptions = [...assets.options];
                    newOptions[i] = optionId || '';
                    setAssets({ ...assets, options: newOptions });
                  }}
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
                  onChange={(e) => {
                    const newOptions = [...assets.options];
                    newOptions[i] = e.target.value;
                    setAssets({ ...assets, options: newOptions });
                  }}
                />
              )}
              <Button
                variant='outline'
                className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                onClick={() => {
                  const newOptions = [...assets.options];
                  newOptions.splice(i, 1);
                  setAssets({ ...assets, options: newOptions });
                }}
              >
                <TrashIcon className='h-4 w-4' />
              </Button>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
