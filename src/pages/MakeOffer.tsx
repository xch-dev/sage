import { Assets, commands, NetworkKind } from '@/bindings';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { NftSelector } from '@/components/selectors/NftSelector';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { IntegerInput, TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { usePrices } from '@/hooks/usePrices';
import { uploadToDexie, uploadToMintGarden } from '@/lib/offerUpload';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  ArrowUpToLine,
  HandCoins,
  Handshake,
  ImageIcon,
  LoaderCircleIcon,
  PlusIcon,
  TrashIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { CatRecord } from '../bindings';

export function MakeOffer() {
  const [state, setState] = useOfferStateWithDefault();

  const { expiry } = useDefaultOfferExpiry();

  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [offer, setOffer] = useState('');
  const [pending, setPending] = useState(false);
  const [dexieLink, setDexieLink] = useState('');
  const [mintGardenLink, setMintGardenLink] = useState('');
  const [canUploadToMintGarden, setCanUploadToMintGarden] = useState(false);
  const [network, setNetwork] = useState<NetworkKind | null>(null);

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  useEffect(() => {
    setDexieLink('');
    setMintGardenLink('');
  }, [offer]);

  const handleMake = async () => {
    setPending(true);

    const mintgardenSupported =
      (state.offered.xch === '0' || !state.offered.xch) &&
      state.offered.cats.length === 0 &&
      state.offered.nfts.length === 1;

    let expiresAtSecond = null;
    if (state.expiration !== null) {
      const days = parseInt(state.expiration.days) || 0;
      const hours = parseInt(state.expiration.hours) || 0;
      const minutes = parseInt(state.expiration.minutes) || 0;
      const totalSeconds = days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60;
      if (totalSeconds <= 0) {
        addError({
          kind: 'invalid',
          reason: t`Expiration must be at least 1 second in the future`,
        });
        setPending(false);
        return;
      }
      expiresAtSecond = Math.ceil(Date.now() / 1000) + totalSeconds;
    }

    const data = await commands.makeOffer({
      offered_assets: {
        xch: toMojos(
          (state.offered.xch || '0').toString(),
          walletState.sync.unit.decimals,
        ),
        cats: state.offered.cats.map((cat) => ({
          asset_id: cat.asset_id,
          amount: toMojos((cat.amount || '0').toString(), 3),
        })),
        nfts: state.offered.nfts,
      },
      requested_assets: {
        xch: toMojos(
          (state.requested.xch || '0').toString(),
          walletState.sync.unit.decimals,
        ),
        cats: state.requested.cats.map((cat) => ({
          asset_id: cat.asset_id,
          amount: toMojos((cat.amount || '0').toString(), 3),
        })),
        nfts: state.requested.nfts,
      },
      fee: toMojos(
        (state.fee || '0').toString(),
        walletState.sync.unit.decimals,
      ),
      expires_at_second: expiresAtSecond,
    });

    setState(null);
    setOffer(data.offer);
    setPending(false);
    setCanUploadToMintGarden(mintgardenSupported);
  };

  const make = () => handleMake().catch(addError);

  const invalid =
    state.expiration !== null &&
    (parseInt(state.expiration.days) || 0) === 0 &&
    (parseInt(state.expiration.hours) || 0) === 0 &&
    (parseInt(state.expiration.minutes) || 0) === 0;

  return (
    <>
      <Header title={t`New Offer`} />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                <Trans>Offered</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Add the assets you are offering.</Trans>
              </div>

              <AssetSelector
                offering
                prefix='offer'
                assets={state.offered}
                setAssets={(assets) => setState({ offered: assets })}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                <Trans>Requested</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Add the assets you are requesting.</Trans>
              </div>

              <AssetSelector
                prefix='requested'
                assets={state.requested}
                setAssets={(assets) => setState({ requested: assets })}
              />
            </CardContent>
          </Card>

          <div className='flex flex-col gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='fee'>
                <Trans>Network Fee</Trans>
              </Label>
              <div className='relative'>
                <TokenAmountInput
                  id='fee'
                  type='text'
                  placeholder={'0.00'}
                  className='pr-12'
                  value={state.fee}
                  onValueChange={(values) => {
                    setState({
                      fee: values.value,
                    });
                  }}
                />
                <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                  <span className='text-gray-500 text-sm' id='price-currency'>
                    {walletState.sync.unit.ticker}
                  </span>
                </div>
              </div>
            </div>

            <div className='flex flex-col gap-2'>
              <div className='flex items-center gap-2'>
                <label htmlFor='expiring'>
                  <Trans>Expiring offer</Trans>
                </label>
                <Switch
                  id='expiring'
                  checked={state.expiration !== null}
                  onCheckedChange={(value) => {
                    if (value) {
                      setState({
                        expiration: {
                          days: expiry.days.toString(),
                          hours: expiry.hours.toString(),
                          minutes: expiry.minutes.toString(),
                        },
                      });
                    } else {
                      setState({ expiration: null });
                    }
                  }}
                />
              </div>

              {state.expiration !== null && (
                <div className='flex gap-2'>
                  <div className='relative'>
                    <IntegerInput
                      className='pr-12'
                      value={state.expiration.days}
                      placeholder='0'
                      min={0}
                      onValueChange={(values) => {
                        if (!state.expiration) return;
                        setState({
                          expiration: {
                            ...state.expiration,
                            days: values.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>
                        <Trans>Days</Trans>
                      </span>
                    </div>
                  </div>

                  <div className='relative'>
                    <IntegerInput
                      className='pr-12'
                      value={state.expiration.hours}
                      placeholder='0'
                      min={0}
                      onValueChange={(values) => {
                        if (!state.expiration) return;
                        setState({
                          expiration: {
                            ...state.expiration,
                            hours: values.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>
                        <Trans>Hours</Trans>
                      </span>
                    </div>
                  </div>

                  <div className='relative'>
                    <IntegerInput
                      className='pr-12'
                      value={state.expiration.minutes}
                      placeholder='0'
                      min={0}
                      onValueChange={(values) => {
                        if (!state.expiration) return;
                        setState({
                          expiration: {
                            ...state.expiration,
                            minutes: values.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>
                        <Trans>Minutes</Trans>
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className='mt-4 flex gap-2'>
          <Button
            variant='outline'
            onClick={() => {
              setState(null);
              navigate('/offers', { replace: true });
            }}
          >
            <Trans>Cancel Offer</Trans>
          </Button>
          <Button disabled={invalid || pending} onClick={make}>
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {pending ? t`Creating Offer` : t`Create Offer`}
          </Button>
        </div>

        <Dialog open={!!offer} onOpenChange={() => setOffer('')}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                <Trans>Offer Created</Trans>
              </DialogTitle>
              <DialogDescription>
                <Trans>
                  The offer has been created and imported successfully. You can
                  copy the offer file below and send it to the intended
                  recipient or make it public to be accepted by anyone.
                </Trans>
                <CopyBox title='Offer File' value={offer} className='mt-2' />
                {network !== 'unknown' && (
                  <div className='flex flex-col gap-2 mt-2'>
                    <div className='grid grid-cols-2 gap-2'>
                      <Button
                        variant='outline'
                        className='text-neutral-800 dark:text-neutral-200'
                        onClick={() => {
                          if (dexieLink) return openUrl(dexieLink);
                          uploadToDexie(offer, network === 'testnet')
                            .then(setDexieLink)
                            .catch((error) =>
                              addError({
                                kind: 'upload',
                                reason: `${error}`,
                              }),
                            );
                        }}
                      >
                        <img
                          src='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                          className='h-4 w-4 mr-2'
                          alt='Dexie logo'
                        />
                        {dexieLink ? t`Dexie Link` : t`Upload to Dexie`}
                      </Button>

                      {canUploadToMintGarden && (
                        <Button
                          variant='outline'
                          className='text-neutral-800 dark:text-neutral-200'
                          onClick={() => {
                            if (mintGardenLink) return openUrl(mintGardenLink);
                            uploadToMintGarden(offer, network === 'testnet')
                              .then(setMintGardenLink)
                              .catch((error) =>
                                addError({
                                  kind: 'upload',
                                  reason: `${error}`,
                                }),
                              );
                          }}
                        >
                          <img
                            src='https://mintgarden.io/mint-logo.svg'
                            className='h-4 w-4 mr-2'
                            alt='MintGarden logo'
                          />
                          {mintGardenLink
                            ? t`MintGarden Link`
                            : t`Upload to MintGarden`}
                        </Button>
                      )}
                    </div>
                  </div>
                )}
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button
                onClick={() => {
                  setState(null);
                  navigate('/offers', { replace: true });
                }}
              >
                <Trans>Ok</Trans>
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </>
  );
}

interface AssetSelectorProps {
  offering?: boolean;
  prefix: string;
  assets: Assets;
  setAssets: (value: Assets) => void;
}

function AssetSelector({
  offering,
  prefix,
  assets,
  setAssets,
}: AssetSelectorProps) {
  const [state] = useOfferStateWithDefault();
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
              state.offered.cats.length === 1 &&
              state.offered.cats[0].amount && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='outline'
                        size='icon'
                        className='border-l-0 rounded-none flex-shrink-0'
                        onClick={() => {
                          const cat = state.offered.cats[0];
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
          {assets.nfts.map((nft, i) => (
            <div key={i} className='flex h-14 z-20'>
              {offering === true ? (
                <NftSelector
                  value={nft || null}
                  onChange={(nftId) => {
                    assets.nfts[i] = nftId || '';
                    setAssets({ ...assets });
                  }}
                  disabled={assets.nfts.filter((id) => id !== nft)}
                  className='rounded-r-none'
                />
              ) : (
                <Input
                  className='flex-grow rounded-r-none h-12 z-10'
                  placeholder='Enter NFT id'
                  value={nft}
                  onChange={(e) => {
                    assets.nfts[i] = e.target.value;
                    setAssets({ ...assets });
                  }}
                />
              )}
              <Button
                variant='outline'
                className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0 h-12 px-3'
                onClick={() => {
                  assets.nfts.splice(i, 1);
                  setAssets({ ...assets });
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
            <div key={i} className='flex h-14'>
              <TokenSelector
                value={cat.asset_id}
                onChange={(assetId) => {
                  assets.cats[i].asset_id = assetId;
                  setAssets({ ...assets });
                }}
                disabled={assets.cats
                  .filter((amount) => amount.asset_id !== cat.asset_id)
                  .map((amount) => amount.asset_id)}
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
                    assets.cats[i].amount = values.value;
                    setAssets({ ...assets });
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
                              assets.cats[i].amount = (
                                Number(token.balance) / 1000
                              ).toString();
                              setAssets({ ...assets });
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
                    assets.cats.splice(i, 1);
                    setAssets({ ...assets });
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
