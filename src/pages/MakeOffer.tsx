import { Assets, commands, NetworkKind, Error as ChiaError } from '@/bindings';
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
import { useNavigate, useLocation } from 'react-router-dom';
import { CatRecord } from '../bindings';
import { MakeOfferConfirmationDialog } from '@/components/MakeOfferConfirmationDialog';
import { useOfferProcessor } from '@/hooks/useOfferProcessor';

export function MakeOffer() {
  const [state, setState] = useOfferStateWithDefault();
  const location = useLocation();
  const { expiry } = useDefaultOfferExpiry();
  const walletState = useWalletState();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [dexieLink, setDexieLink] = useState('');
  const [mintGardenLink, setMintGardenLink] = useState('');
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [splitNftOffers, setSplitNftOffers] = useState(
    location.state?.splitNftOffers || false,
  );
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [autoUploadToDexie, setAutoUploadToDexie] = useState(false);
  const {
    createdOffer,
    createdOffers,
    isProcessing,
    canUploadToMintGarden,
    processOffer,
    clearProcessedOffers,
  } = useOfferProcessor({
    offerState: state,
    splitNftOffers,
    onProcessingEnd: () => {
      if (createdOffer || createdOffers.length > 0) {
        setState(null);
      }
    },
  });
  const [selectedDialogOffer, setSelectedDialogOffer] = useState('');

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  useEffect(() => {
    if (!createdOffer && createdOffers.length === 0) {
      setDexieLink('');
      setMintGardenLink('');
      setSelectedDialogOffer('');
    }
  }, [createdOffer, createdOffers]);

  useEffect(() => {
    if (autoUploadToDexie && createdOffer && !createdOffers.length && network) {
      uploadToDexie(createdOffer, network === 'testnet')
        .then(setDexieLink)
        .catch((error) =>
          addError({
            kind: 'upload',
            reason: `Failed to auto-upload to Dexie: ${error}`,
          }),
        )
        .finally(() => {
          setAutoUploadToDexie(false);
        });
    } else if (autoUploadToDexie && createdOffers.length > 0 && network) {
      Promise.all(
        createdOffers.map((individualOffer) =>
          uploadToDexie(individualOffer, network === 'testnet')
            .then((link) => {
              if (createdOffers.indexOf(individualOffer) === 0) {
                setDexieLink(link);
              }
              console.log(
                `Successfully uploaded offer ${createdOffers.indexOf(individualOffer) + 1} to Dexie: ${link}`,
              );
            })
            .catch((error) => {
              addError({
                kind: 'upload',
                reason: `Failed to auto-upload offer ${createdOffers.indexOf(individualOffer) + 1} to Dexie: ${error}`,
              });
              console.error(
                `Failed to auto-upload offer ${createdOffers.indexOf(individualOffer) + 1} to Dexie: ${error}`,
              );
            }),
        ),
      ).finally(() => {
        setAutoUploadToDexie(false);
      });
    }
  }, [createdOffer, createdOffers, autoUploadToDexie, network, addError]);

  useEffect(() => {
    if (createdOffers.length > 0) {
      setSelectedDialogOffer(createdOffers[0]);
    } else if (createdOffer) {
      setSelectedDialogOffer(createdOffer);
    }
  }, [createdOffer, createdOffers]);

  const makeAction = () => {
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
        return;
      }
    }
    const hasOfferedXch = state.offered.xch && state.offered.xch !== '0';
    const hasOfferedCats = state.offered.cats.length > 0;
    const hasOfferedNfts = state.offered.nfts.filter((n) => n).length > 0;
    const hasRequestedXch = state.requested.xch && state.requested.xch !== '0';
    const hasRequestedCats = state.requested.cats.length > 0;
    const hasRequestedNfts = state.requested.nfts.filter((n) => n).length > 0;

    if (
      !(
        hasOfferedXch ||
        hasOfferedCats ||
        hasOfferedNfts ||
        hasRequestedXch ||
        hasRequestedCats ||
        hasRequestedNfts
      )
    ) {
      addError({
        kind: 'invalid',
        reason: t`Offer must include at least one offered or requested asset.`,
      });
      return;
    }
    setIsConfirmDialogOpen(true);
  };

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
                splitNftOffers={splitNftOffers}
                setSplitNftOffers={setSplitNftOffers}
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
                splitNftOffers={splitNftOffers}
                setSplitNftOffers={setSplitNftOffers}
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
              clearProcessedOffers();
              navigate('/offers', { replace: true });
            }}
          >
            <Trans>Cancel Offer</Trans>
          </Button>
          <Button disabled={invalid || isProcessing} onClick={makeAction}>
            {isProcessing && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {isProcessing ? t`Creating Offer` : t`Create Offer`}
          </Button>
        </div>

        <Dialog
          open={!!createdOffer || createdOffers.length > 0}
          onOpenChange={(isOpen) => {
            if (!isOpen) {
              clearProcessedOffers();
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {createdOffers.length > 1 ? (
                  <Trans>Offers Created</Trans>
                ) : (
                  <Trans>Offer Created</Trans>
                )}
              </DialogTitle>
              <DialogDescription>
                {createdOffers.length > 1 ? (
                  <Trans>
                    {createdOffers.length} offers have been created and imported
                    successfully. Select an offer to view its details or copy
                    it.
                  </Trans>
                ) : (
                  <Trans>
                    The offer has been created and imported successfully. You
                    can copy the offer file below and send it to the intended
                    recipient or make it public to be accepted by anyone.
                  </Trans>
                )}
                {(createdOffers.length > 1 ||
                  (createdOffers.length === 0 && createdOffer)) && (
                  <div className='mt-2'>
                    <Label>
                      {createdOffers.length > 1 ? (
                        <Trans>Select Offer</Trans>
                      ) : (
                        <Trans>Offer File</Trans>
                      )}
                    </Label>
                    {createdOffers.length > 1 ? (
                      <select
                        className='w-full mt-1 p-2 border rounded'
                        value={selectedDialogOffer}
                        onChange={(e) => setSelectedDialogOffer(e.target.value)}
                      >
                        {createdOffers.map((o, i) => (
                          <option key={i} value={o}>
                            {t`Offer ${i + 1}`}
                          </option>
                        ))}
                      </select>
                    ) : null}
                    <CopyBox
                      title={
                        createdOffers.length > 1
                          ? t`Offer ${createdOffers.indexOf(selectedDialogOffer) + 1}`
                          : t`Offer File`
                      }
                      value={selectedDialogOffer}
                      className='mt-2'
                    />
                  </div>
                )}

                {network !== 'unknown' &&
                  (createdOffers.length > 0 || createdOffer) && (
                    <div className='flex flex-col gap-2 mt-2'>
                      <div className='grid grid-cols-2 gap-2'>
                        <Button
                          variant='outline'
                          className='text-neutral-800 dark:text-neutral-200'
                          onClick={() => {
                            if (dexieLink) return openUrl(dexieLink);
                            uploadToDexie(
                              selectedDialogOffer,
                              network === 'testnet',
                            )
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
                              if (mintGardenLink)
                                return openUrl(mintGardenLink);
                              uploadToMintGarden(
                                selectedDialogOffer,
                                network === 'testnet',
                              )
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
                  clearProcessedOffers();
                  navigate('/offers', { replace: true });
                }}
              >
                <Trans>Ok</Trans>
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <MakeOfferConfirmationDialog
          open={isConfirmDialogOpen}
          onOpenChange={setIsConfirmDialogOpen}
          onConfirm={async () => {
            await processOffer();
            setIsConfirmDialogOpen(false);
          }}
          offerState={state}
          splitNftOffers={splitNftOffers}
          fee={state.fee || '0'}
          walletUnit={walletState.sync.unit.ticker}
          walletDecimals={walletState.sync.unit.decimals}
          autoUploadToDexie={autoUploadToDexie}
          setAutoUploadToDexie={setAutoUploadToDexie}
        />
      </Container>
    </>
  );
}

interface AssetSelectorProps {
  offering?: boolean;
  prefix: string;
  assets: Assets;
  setAssets: (value: Assets) => void;
  splitNftOffers?: boolean;
  setSplitNftOffers?: (value: boolean) => void;
}

function AssetSelector({
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
