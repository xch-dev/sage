import { Assets, commands } from '@/bindings';
import { NftSelector } from '@/components/selectors/NftSelector';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { usePrices } from '@/hooks/usePrices';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowUpToLine,
  HandCoins,
  ImageIcon,
  PlusIcon,
  TrashIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { CatRecord } from '@/bindings';
import { TokenAmountInput } from '@/components/ui/masked-input';

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
  const [currentState] = useOfferStateWithDefault();
  const [includeAmount, setIncludeAmount] = useState(!!assets.xch);
  const [tokens, setTokens] = useState<CatRecord[]>([]);
  const { getCatAskPriceInXch } = usePrices();

  useEffect(() => {
    if (!offering) return;
    commands
      .getCats({})
      .then((data) => setTokens(data.cats))
      .catch(console.error);
  }, [offering]);

  const calculateXchEquivalent = (catAmount: number, assetId: string) => {
    const catPriceInXch = getCatAskPriceInXch(assetId);
    if (catPriceInXch === null) return '0';
    return (catAmount * catPriceInXch).toFixed(9);
  };

  return (
    <>
      <div className='mt-4 flex gap-2 w-full items-center'>
        <Button
          variant='outline'
          className='flex-grow'
          disabled={includeAmount}
          onClick={() => setIncludeAmount(true)}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' />
          XCH
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
              cats: [...assets.cats, { asset_id: '', amount: '' }],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> <Trans>Token</Trans>
        </Button>
      </div>

      {includeAmount && (
        <div className='mt-4 flex flex-col space-y-1.5'>
          <Label htmlFor={`${prefix}-amount`}>XCH</Label>
          <div className='flex'>
            <TokenAmountInput
              id={`${prefix}-amount`}
              type='text'
              className='rounded-r-none z-10'
              placeholder={t`Enter amount`}
              value={assets.xch}
              onValueChange={(values) => {
                setAssets({
                  ...assets,
                  xch: values.value,
                });
              }}
            />
            {!offering &&
              currentState.offered.cats.length === 1 &&
              currentState.offered.cats[0].amount && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='outline'
                        size='icon'
                        className='border-l-0 rounded-none flex-shrink-0'
                        onClick={() => {
                          const cat = currentState.offered.cats[0];
                          const xchAmount = calculateXchEquivalent(
                            Number(cat.amount),
                            cat.asset_id,
                          );
                          setAssets({ ...assets, xch: xchAmount });
                        }}
                      >
                        <ArrowUpToLine className='h-4 w-4 rotate-90' />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <Trans>Convert to XCH at current asking price</Trans>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            <Button
              variant='outline'
              size='icon'
              className='border-l-0 rounded-l-none flex-shrink-0'
              onClick={() => {
                setAssets({
                  ...assets,
                  xch: '0',
                });
                setIncludeAmount(false);
              }}
            >
              <TrashIcon className='h-4 w-4' />
            </Button>
          </div>
        </div>
      )}

      {assets.nfts.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <ImageIcon className='h-4 w-4' />
            <span>NFTs</span>
          </Label>
          {offering && assets.nfts.filter((n) => n).length > 1 && (
            <div className='flex items-center gap-2 mb-2'>
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
            <div key={i} className='flex h-14 z-20 mb-1'>
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

      {assets.cats.length > 0 && (
        <div className='flex flex-col mt-4'>
          <Label className='flex items-center gap-1 mb-2'>
            <HandCoins className='h-4 w-4' />
            <span>Tokens</span>
          </Label>
          {assets.cats.map((cat, i) => (
            <div key={i} className='flex h-14 mb-1'>
              <TokenSelector
                value={cat.asset_id}
                onChange={(assetId) => {
                  const newCats = [...assets.cats];
                  newCats[i] = { ...newCats[i], asset_id: assetId };
                  setAssets({ ...assets, cats: newCats });
                }}
                disabled={assets.cats
                  .filter((c, idx) => c.asset_id !== '' && idx !== i)
                  .map((c) => c.asset_id)}
                className='rounded-r-none'
                hideZeroBalance={offering === true}
              />
              <div className='flex flex-grow-0'>
                <TokenAmountInput
                  id={`${prefix}-cat-${i}-amount`}
                  className='border-l-0 z-10 rounded-l-none rounded-r-none w-[100px] h-12'
                  placeholder={t`Amount`}
                  value={cat.amount}
                  onValueChange={(values) => {
                    const newCats = [...assets.cats];
                    newCats[i] = { ...newCats[i], amount: values.value };
                    setAssets({ ...assets, cats: newCats });
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
                            const token = tokens.find(
                              (t) => t.asset_id === cat.asset_id,
                            );
                            if (token) {
                              const newCats = [...assets.cats];
                              newCats[i] = {
                                ...newCats[i],
                                amount: (
                                  Number(token.balance) / 1000
                                ).toString(),
                              };
                              setAssets({ ...assets, cats: newCats });
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
                    const newCats = [...assets.cats];
                    newCats.splice(i, 1);
                    setAssets({ ...assets, cats: newCats });
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
