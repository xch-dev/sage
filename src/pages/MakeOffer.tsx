import { commands, NetworkKind } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import {
  IntegerInput,
  TokenAmountInput,
  FeeAmountInput,
} from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { uploadToDexie, uploadToMintGarden } from '@/lib/offerUpload';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { HandCoins, Handshake, LoaderCircleIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { MakeOfferConfirmationDialog } from '@/components/confirmations/MakeOfferConfirmationDialog';
import { useOfferProcessor } from '@/hooks/useOfferProcessor';
import { AssetSelector } from '@/components/selectors/AssetSelector';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export function MakeOffer() {
  const [state, setState] = useOfferStateWithDefault();
  const location = useLocation();
  const { expiry } = useDefaultOfferExpiry();
  const walletState = useWalletState();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [splitNftOffers, setSplitNftOffers] = useState(
    location.state?.splitNftOffers || false,
  );
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [autoUploadToDexie, setAutoUploadToDexie] = useState(false);
  const [autoUploadToMintGarden, setAutoUploadToMintGarden] = useState(false);
  const {
    createdOffers,
    isProcessing,
    processOffer,
    clearProcessedOffers,
    cancelProcessing,
  } = useOfferProcessor({
    offerState: state,
    splitNftOffers,
    onProcessingEnd: () => {
      if (createdOffers.length > 0) {
        setState(null);
      }
    },
  });

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  useEffect(() => {
    if (autoUploadToDexie && createdOffers.length > 0 && network) {
      let isMounted = true;
      const uploadWithDelay = async () => {
        for (const [index, individualOffer] of createdOffers.entries()) {
          if (!isMounted) break;
          try {
            const link = await uploadToDexie(
              individualOffer,
              network === 'testnet',
            );
            if (isMounted) {
              console.log(
                `Successfully uploaded offer ${index + 1} to Dexie: ${link}`,
              );
            }
            // Add delay between uploads, but not after the last one
            if (index < createdOffers.length - 1) {
              await delay(500);
            }
          } catch (error) {
            if (isMounted) {
              addError({
                kind: 'upload',
                reason: `Failed to auto-upload offer ${index + 1} to Dexie: ${error}`,
              });
              console.error(
                `Failed to auto-upload offer ${index + 1} to Dexie: ${error}`,
              );
            }
          }
        }
        if (isMounted) {
          setAutoUploadToDexie(false);
        }
      };
      uploadWithDelay();
      return () => {
        isMounted = false;
      };
    }
  }, [createdOffers, autoUploadToDexie, network, addError]);

  useEffect(() => {
    if (autoUploadToMintGarden && createdOffers.length > 0 && network) {
      let isMounted = true;
      const uploadWithDelay = async () => {
        for (const [index, individualOffer] of createdOffers.entries()) {
          if (!isMounted) break;
          try {
            const link = await uploadToMintGarden(
              individualOffer,
              network === 'testnet',
            );
            if (isMounted) {
              console.log(
                `Successfully uploaded offer ${index + 1} to MintGarden: ${link}`,
              );
            }
            // Add delay between uploads, but not after the last one
            if (index < createdOffers.length - 1) {
              await delay(500);
            }
          } catch (error) {
            if (isMounted) {
              addError({
                kind: 'upload',
                reason: `Failed to auto-upload offer ${index + 1} to MintGarden: ${error}`,
              });
              console.error(
                `Failed to auto-upload offer ${index + 1} to MintGarden: ${error}`,
              );
            }
          }
        }
        if (isMounted) {
          setAutoUploadToMintGarden(false);
        }
      };
      uploadWithDelay();
      return () => {
        isMounted = false;
      };
    }
  }, [createdOffers, autoUploadToMintGarden, network, addError]);

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
                setAssets={(assets: any) => setState({ offered: assets })}
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
                setAssets={(assets: any) => setState({ requested: assets })}
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
                <FeeAmountInput
                  id='fee'
                  className='pr-12'
                  onValueChange={(values: { value: string }) => {
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
          autoUploadToMintGarden={autoUploadToMintGarden}
          setAutoUploadToMintGarden={setAutoUploadToMintGarden}
        />
        <OfferCreationProgressDialog
          open={createdOffers.length > 0 || isProcessing}
          onOpenChange={(isOpen) => {
            if (!isOpen && createdOffers.length > 0) {
              clearProcessedOffers();
              setState(null);
              navigate('/offers', { replace: true });
            }
          }}
          createdOffers={createdOffers}
          onOk={() => {
            clearProcessedOffers();
            setState(null);
            navigate('/offers', { replace: true });
          }}
          isProcessing={isProcessing}
          onCancel={() => {
            cancelProcessing();
          }}
        />
      </Container>
    </>
  );
}
