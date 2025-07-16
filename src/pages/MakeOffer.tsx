import Container from '@/components/Container';
import { MakeOfferConfirmationDialog } from '@/components/dialogs/MakeOfferConfirmationDialog';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';
import Header from '@/components/Header';
import { AssetSelector } from '@/components/selectors/AssetSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, IntegerInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { HandCoins, Handshake } from 'lucide-react';
import { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

export function MakeOffer() {
  const [state, setState] = useOfferStateWithDefault();
  const location = useLocation();
  const { expiry } = useDefaultOfferExpiry();
  const walletState = useWalletState();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [splitNftOffers, setSplitNftOffers] = useState(
    location.state?.splitNftOffers || false,
  );
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [isProgressDialogOpen, setIsProgressDialogOpen] = useState(false);
  const [enabledMarketplaces, setEnabledMarketplaces] = useState<
    Record<string, boolean>
  >({});
  const [hasOfferedXchAdded, setHasOfferedXchAdded] = useState(false);
  const [hasRequestedXchAdded, setHasRequestedXchAdded] = useState(false);

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

    for (const cat of [...state.offered.cats, ...state.requested.cats]) {
      const amount = parseFloat(cat.amount?.toString() || '');

      if (isNaN(amount) || amount <= 0) {
        addError({
          kind: 'invalid',
          reason: t`Tokens must have a positive amount.`,
        });
        return;
      }
    }

    // Check if XCH amounts are valid (positive numbers)
    const hasOfferedXchValid =
      hasOfferedXchAdded &&
      state.offered.xch !== '0' &&
      !isNaN(parseFloat(String(state.offered.xch))) &&
      parseFloat(String(state.offered.xch)) > 0;
    const hasRequestedXchValid =
      hasRequestedXchAdded &&
      state.requested.xch !== '0' &&
      !isNaN(parseFloat(String(state.requested.xch))) &&
      parseFloat(String(state.requested.xch)) > 0;

    // Validate XCH amounts if they've been added
    if (hasOfferedXchAdded && !hasOfferedXchValid) {
      addError({
        kind: 'invalid',
        reason: t`Offered XCH amount must be a positive number.`,
      });
      return;
    }

    if (hasRequestedXchAdded && !hasRequestedXchValid) {
      addError({
        kind: 'invalid',
        reason: t`Requested XCH amount must be a positive number.`,
      });
      return;
    }

    const hasOfferedCats = state.offered.cats.length > 0;
    const hasOfferedNfts = state.offered.nfts.filter((n) => n).length > 0;
    const hasRequestedCats = state.requested.cats.length > 0;
    const hasRequestedNfts = state.requested.nfts.filter((n) => n).length > 0;

    if (
      !(
        hasOfferedXchValid ||
        hasOfferedCats ||
        hasOfferedNfts ||
        hasRequestedXchValid ||
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

  const handleConfirm = () => {
    setIsProgressDialogOpen(true);
  };

  const handleProgressDialogClose = (isOpen: boolean) => {
    if (!isOpen) {
      setIsProgressDialogOpen(false);
    }
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
                onXchStateChange={setHasOfferedXchAdded}
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
                onXchStateChange={setHasRequestedXchAdded}
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
              navigate('/offers', { replace: true });
            }}
          >
            <Trans>Cancel Offer</Trans>
          </Button>
          <Button disabled={invalid} onClick={makeAction}>
            <Trans>Create Offer</Trans>
          </Button>
        </div>

        <MakeOfferConfirmationDialog
          open={isConfirmDialogOpen}
          onOpenChange={setIsConfirmDialogOpen}
          onConfirm={handleConfirm}
          offerState={state}
          splitNftOffers={splitNftOffers}
          fee={state.fee || '0'}
          walletUnit={walletState.sync.unit.ticker}
          walletDecimals={walletState.sync.unit.decimals}
          enabledMarketplaces={enabledMarketplaces}
          setEnabledMarketplaces={setEnabledMarketplaces}
        />

        <OfferCreationProgressDialog
          open={isProgressDialogOpen}
          onOpenChange={handleProgressDialogClose}
          offerState={state}
          splitNftOffers={splitNftOffers}
          enabledMarketplaces={enabledMarketplaces}
          clearOfferState={() => setState(null)}
        />
      </Container>
    </>
  );
}
