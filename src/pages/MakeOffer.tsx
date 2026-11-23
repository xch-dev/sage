import Container from '@/components/Container';
import { ExpirationInput } from '@/components/ExpirationInput';
import {
  OfferChoice,
  OfferSettingCard,
  OfferSidesDivider,
} from '@/components/OfferSettings';
import { MakeOfferConfirmationDialog } from '@/components/dialogs/MakeOfferConfirmationDialog';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';
import Header from '@/components/Header';
import { ReadOnlyButton } from '@/components/ReadOnlyButton';
import { AssetSelector } from '@/components/selectors/AssetSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { FeeAmountInput } from '@/components/ui/masked-input';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useErrors } from '@/hooks/useErrors';
import { useNowSeconds } from '@/hooks/useNowSeconds';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { expirationError, futureExpirationTimestamp } from '@/lib/expiration';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { HandCoins, Handshake } from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function MakeOffer() {
  const [state, setState] = useOfferStateWithDefault();
  const { expiry } = useDefaultOfferExpiry();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const now = useNowSeconds();
  // Kept in the offer state so they survive leaving the page, like the assets.
  const { splitNftOffers, preSplit, preSplitFee } = state;

  // Each separate NFT offer would split coins of its own, and the change from
  // one split can't be spent on the next offer until it's confirmed.
  const offeredNftCount = state.offered.nfts.filter((n) => n).length;
  const isSplittingNftOffers = splitNftOffers && offeredNftCount > 1;
  const isPreSplitting = preSplit && !isSplittingNftOffers;
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [isProgressDialogOpen, setIsProgressDialogOpen] = useState(false);
  const [enabledMarketplaces, setEnabledMarketplaces] = useState<
    Record<string, boolean>
  >({});

  const makeAction = () => {
    if (state.expiration !== null) {
      try {
        futureExpirationTimestamp(state.expiration);
      } catch (error) {
        addError({ kind: 'invalid', reason: (error as Error).message });
        return;
      }
    }

    for (const cat of [...state.offered.tokens, ...state.requested.tokens]) {
      const amount = parseFloat(cat.amount?.toString() || '');

      if (isNaN(amount) || amount <= 0) {
        addError({
          kind: 'invalid',
          reason: t`Tokens must have a positive amount.`,
        });
        return;
      }
    }

    const hasOfferedTokens = state.offered.tokens.length > 0;
    const hasOfferedNfts = state.offered.nfts.filter((n) => n).length > 0;
    const hasRequestedTokens = state.requested.tokens.length > 0;
    const hasRequestedNfts = state.requested.nfts.filter((n) => n).length > 0;
    const hasOfferedOptions = state.offered.options.filter((o) => o).length > 0;
    const hasRequestedOptions =
      state.requested.options.filter((o) => o).length > 0;

    if (!(
      hasOfferedTokens ||
      hasOfferedNfts ||
      hasRequestedTokens ||
      hasRequestedNfts ||
      hasOfferedOptions ||
      hasRequestedOptions
    )) {
      addError({
        kind: 'invalid',
        reason: t`Offer must include at least one offered or requested asset.`,
      });
      return;
    }

    if (
      !hasOfferedTokens &&
      !hasOfferedNfts &&
      !hasOfferedOptions &&
      parseFloat(state.fee || '0') === 0
    ) {
      addError({
        kind: 'invalid',
        reason: t`A request-only offer requires a network fee.`,
      });
      return;
    }

    setIsConfirmDialogOpen(true);
  };

  // The split fee as it was confirmed, with an automatic one fixed at the rate it was shown at.
  const [confirmedPreSplitFee, setConfirmedPreSplitFee] = useState<
    string | null
  >(null);

  const handleConfirm = (fee: string | null) => {
    setConfirmedPreSplitFee(fee);
    setIsProgressDialogOpen(true);
  };

  const handleProgressDialogClose = (isOpen: boolean) => {
    if (!isOpen) {
      setIsProgressDialogOpen(false);
    }
  };

  const invalid =
    state.expiration !== null &&
    expirationError(state.expiration, now) !== undefined;

  return (
    <>
      <Header title={t`New Offer`} />

      <Container>
        <div className='flex max-w-(--breakpoint-lg) flex-col gap-6'>
          <div className='flex flex-col lg:flex-row lg:items-stretch'>
            <Card className='min-w-0 lg:flex-1'>
              <CardHeader className='pb-4'>
                <CardTitle className='flex items-center text-lg'>
                  <HandCoins className='mr-2 h-5 w-5' aria-hidden='true' />
                  <Trans>You offer</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <AssetSelector
                  offering
                  prefix='offer'
                  assets={state.offered}
                  setAssets={(assets) => setState({ offered: assets })}
                  emptyMessage={<Trans>Add the assets you are offering.</Trans>}
                  fee={state.fee}
                />
              </CardContent>
            </Card>

            <OfferSidesDivider />

            <Card className='min-w-0 lg:flex-1'>
              <CardHeader className='pb-4'>
                <CardTitle className='flex items-center text-lg'>
                  <Handshake className='mr-2 h-5 w-5' aria-hidden='true' />
                  <Trans>You request</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <AssetSelector
                  prefix='requested'
                  assets={state.requested}
                  setAssets={(assets) => setState({ requested: assets })}
                  emptyMessage={
                    <Trans>
                      Add the assets you are requesting, or leave this empty to
                      give the offered assets away.
                    </Trans>
                  }
                />
              </CardContent>
            </Card>
          </div>

          <div className='flex flex-col gap-3'>
            <h2 className='font-heading text-xl font-semibold tracking-tight'>
              <Trans>Offer Settings</Trans>
            </h2>

            <OfferSettingCard
              title={<Trans>Network Fee</Trans>}
              description={
                <Trans>
                  Paid by you when the offer is taken. Required if you are only
                  requesting assets.
                </Trans>
              }
            >
              <div className='max-w-xs'>
                <FeeAmountInput
                  id='fee'
                  allowAuto={false}
                  value={state.fee}
                  onValueChange={(values, sourceInfo) => {
                    // A prop change only echoes the stored fee. Writing it back
                    // would recreate the offer while Cancel is clearing it.
                    if (sourceInfo?.source === 'prop') return;
                    setState({ fee: values.value });
                  }}
                />
              </div>
            </OfferSettingCard>

            <OfferSettingCard
              title={<Trans>Expiration</Trans>}
              description={
                state.expiration !== null ? (
                  <Trans>
                    The offer closes automatically once the time runs out.
                  </Trans>
                ) : (
                  <Trans>
                    The offer stays open until it&apos;s taken or you cancel it.
                  </Trans>
                )
              }
              toggle={{
                checked: state.expiration !== null,
                onCheckedChange: (checked) => {
                  if (checked) {
                    setState({
                      expiration: {
                        days: expiry?.days?.toString() ?? '1',
                        hours: expiry?.hours?.toString() ?? '0',
                        minutes: expiry?.minutes?.toString() ?? '0',
                      },
                    });
                  } else {
                    setState({ expiration: null });
                  }
                },
              }}
            >
              {state.expiration !== null && (
                <div className='max-w-lg'>
                  <ExpirationInput
                    value={state.expiration}
                    onChange={(expiration) => setState({ expiration })}
                  />
                </div>
              )}
            </OfferSettingCard>

            {offeredNftCount > 1 && (
              <OfferSettingCard title={<Trans>NFT Offers</Trans>}>
                <OfferChoice
                  label={t`NFT offers`}
                  value={splitNftOffers ? 'split' : 'together'}
                  onValueChange={(value) =>
                    setState({ splitNftOffers: value === 'split' })
                  }
                  options={[
                    {
                      value: 'together',
                      label: <Trans>One offer</Trans>,
                      description: (
                        <Trans>All of the NFTs are taken together.</Trans>
                      ),
                    },
                    {
                      value: 'split',
                      label: <Trans>One offer per NFT</Trans>,
                      description: (
                        <Trans>
                          Each NFT gets its own offer, requesting the same
                          assets.
                        </Trans>
                      ),
                    },
                  ]}
                />
              </OfferSettingCard>
            )}

            <OfferSettingCard
              title={<Trans>Split coins first</Trans>}
              description={
                isSplittingNftOffers ? (
                  <Trans>Not available when each NFT gets its own offer.</Trans>
                ) : isPreSplitting ? (
                  <Trans>
                    Keeps the rest of your balance spendable. The offer is
                    shared once the split is confirmed.
                  </Trans>
                ) : (
                  <Trans>
                    Without a split, the offer may lock up more of your balance
                    than it needs.
                  </Trans>
                )
              }
              toggle={{
                checked: isPreSplitting,
                onCheckedChange: (checked) => setState({ preSplit: checked }),
                disabled: isSplittingNftOffers,
              }}
            >
              <div className='flex max-w-xs flex-col gap-1.5'>
                <Label htmlFor='pre-split-fee'>
                  <Trans>Split fee</Trans>
                </Label>
                <FeeAmountInput
                  id='pre-split-fee'
                  aria-label={t`Split fee`}
                  value={preSplitFee}
                  onValueChange={(values, sourceInfo) => {
                    if (sourceInfo?.source === 'prop') return;
                    setState({ preSplitFee: values.value });
                  }}
                />
              </div>
            </OfferSettingCard>
          </div>
        </div>

        <div className='mt-6 flex gap-2'>
          <Button
            variant='outline'
            onClick={() => {
              setState(null);
              navigate('/offers', { replace: true });
            }}
          >
            <Trans>Cancel</Trans>
          </Button>
          <ReadOnlyButton
            requiresSigning
            disabled={invalid}
            onClick={makeAction}
          >
            <Trans>Create Offer</Trans>
          </ReadOnlyButton>
        </div>

        <MakeOfferConfirmationDialog
          open={isConfirmDialogOpen}
          onOpenChange={setIsConfirmDialogOpen}
          onConfirm={handleConfirm}
          offerState={state}
          splitNftOffers={splitNftOffers}
          fee={state.fee || '0'}
          preSplitFee={isPreSplitting ? preSplitFee : null}
          enabledMarketplaces={enabledMarketplaces}
          setEnabledMarketplaces={setEnabledMarketplaces}
        />

        <OfferCreationProgressDialog
          open={isProgressDialogOpen}
          onOpenChange={handleProgressDialogClose}
          offerState={state}
          splitNftOffers={splitNftOffers}
          preSplitFee={confirmedPreSplitFee}
          enabledMarketplaces={enabledMarketplaces}
          clearOfferState={() => {
            setState(null);
            navigate('/offers', { replace: true });
          }}
        />
      </Container>
    </>
  );
}
