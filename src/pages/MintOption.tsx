import { commands, TokenRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { MintOptionConfirmation } from '@/components/confirmations/MintOptionConfirmation.tsx';
import Container from '@/components/Container';
import { ExpirationInput } from '@/components/ExpirationInput';
import Header from '@/components/Header';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, TokenAmountInput } from '@/components/ui/masked-input';
import { useErrors } from '@/hooks/useErrors';
import { Expiration, expirationTimestamp, nowSeconds } from '@/lib/expiration';
import { toDecimal, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon, HandCoins, Handshake } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function MintOption() {
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [fee, setFee] = useState('');
  const [expiration, setExpiration] = useState<Expiration>({
    days: '',
    hours: '',
    minutes: '',
  });

  const [underlyingAssetId, setUnderlyingAssetId] = useState<
    string | null | undefined
  >();
  const [underlyingAmount, setUnderlyingAmount] = useState('');

  const [strikeAssetId, setStrikeAssetId] = useState<
    string | null | undefined
  >();
  const [strikeAmount, setStrikeAmount] = useState('');

  const [response, setResponse] = useState<TransactionResponse | null>(null);
  // The expiration actually sent to mintOption, shown in the confirmation.
  const [mintedExpirationSeconds, setMintedExpirationSeconds] = useState(0);

  // State for token details
  const [underlyingAsset, setUnderlyingAsset] = useState<TokenRecord | null>(
    null,
  );
  const [strikeAsset, setStrikeAsset] = useState<TokenRecord | null>(null);

  // Fetch token details when asset IDs change
  useEffect(() => {
    if (underlyingAssetId !== undefined) {
      commands
        .getToken({ asset_id: underlyingAssetId })
        .then((res) => setUnderlyingAsset(res.token))
        .catch(() => setUnderlyingAsset(null));
    } else {
      setUnderlyingAsset(null);
    }
  }, [underlyingAssetId]);

  useEffect(() => {
    if (strikeAssetId !== undefined) {
      commands
        .getToken({ asset_id: strikeAssetId })
        .then((res) => setStrikeAsset(res.token))
        .catch(() => setStrikeAsset(null));
    } else {
      setStrikeAsset(null);
    }
  }, [strikeAssetId]);

  const mint = useCallback(() => {
    if (underlyingAssetId === undefined || strikeAssetId === undefined) {
      return;
    }

    // A picked date can pass while the form is open, so check again here.
    const expirationSeconds = expirationTimestamp(expiration);
    if (expirationSeconds <= nowSeconds()) {
      addError({
        kind: 'invalid',
        reason: t`Expiration must be in the future`,
      });
      return;
    }

    commands
      .mintOption({
        fee: toMojos(fee, walletState.sync.unit.precision),
        underlying: {
          asset_id: underlyingAssetId,
          amount: toMojos(
            underlyingAmount,
            underlyingAssetId === null ? walletState.sync.unit.precision : 3,
          ),
        },
        strike: {
          asset_id: strikeAssetId,
          amount: toMojos(
            strikeAmount,
            strikeAssetId === null ? walletState.sync.unit.precision : 3,
          ),
        },
        expiration_seconds: expirationSeconds,
      })
      .then((response) => {
        setMintedExpirationSeconds(expirationSeconds);
        setResponse(response);
      })
      .catch(addError);
  }, [
    addError,
    fee,
    underlyingAssetId,
    underlyingAmount,
    strikeAssetId,
    strikeAmount,
    walletState.sync.unit.precision,
    expiration,
  ]);

  return (
    <>
      <Header title={t`Mint Option`} />

      <Container>
        <Alert variant='warning' className='mb-4'>
          <AlertCircleIcon className='h-4 w-4' />
          <AlertTitle>
            <Trans>Experimental Feature</Trans>
          </AlertTitle>
          <AlertDescription>
            <Trans>
              Option Contracts are experimental and support in other apps is
              limited.
            </Trans>
          </AlertDescription>
        </Alert>

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                <Trans>Underlying Asset</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Select the asset that you want to lock up.</Trans>
              </div>

              <div className='flex mt-4'>
                <TokenSelector
                  value={underlyingAssetId}
                  onChange={(value) => {
                    setUnderlyingAssetId(value);
                    setUnderlyingAmount('');
                  }}
                  className='!rounded-r-none'
                  hideZeroBalance={true}
                  showAllCats={false}
                  includeXch={true}
                />
                <TokenAmountInput
                  id='underlying-amount'
                  className='!border-l-0 z-10 !rounded-l-none w-[150px] h-12'
                  placeholder={t`Amount`}
                  value={underlyingAmount}
                  onValueChange={(values) => {
                    setUnderlyingAmount(values.value);
                  }}
                  precision={underlyingAssetId === null ? 12 : 3}
                  maxValue={
                    underlyingAsset
                      ? BigNumber.max(
                          0,
                          BigNumber(
                            toDecimal(
                              underlyingAsset.selectable_balance,
                              underlyingAsset.precision,
                            ),
                          ).minus(underlyingAssetId === null ? fee || 0 : 0),
                        ).toString()
                      : undefined
                  }
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                <Trans>Strike Asset</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Select the asset that you want to request.</Trans>
              </div>

              <div className='flex mt-4'>
                <TokenSelector
                  value={strikeAssetId}
                  onChange={(value) => {
                    setStrikeAssetId(value);
                    setStrikeAmount('');
                  }}
                  className='!rounded-r-none'
                  hideZeroBalance={false}
                  showAllCats={true}
                  includeXch={true}
                />
                <div className='flex flex-grow-0'>
                  <TokenAmountInput
                    id='strike-amount'
                    className='!border-l-0 z-10 !rounded-l-none w-[150px] h-12'
                    placeholder={t`Amount`}
                    value={strikeAmount}
                    onValueChange={(values) => {
                      setStrikeAmount(values.value);
                    }}
                    precision={strikeAssetId === null ? 12 : 3}
                    hideMaxButton={true}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          <div className='flex flex-col gap-1'>
            <Trans>Expiration</Trans>

            <ExpirationInput value={expiration} onChange={setExpiration} />
          </div>
        </div>

        <div className='flex flex-col space-y-1.5 mt-4'>
          <Label htmlFor='fee'>
            <Trans>Network Fee</Trans>
          </Label>
          <div className='relative'>
            <FeeAmountInput
              id='fee'
              className='pr-12'
              onValueChange={(values) => setFee(values.value)}
            />
            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
              <span
                className='text-muted-foreground text-sm'
                id='price-currency'
              >
                {walletState.sync.unit.ticker}
              </span>
            </div>
          </div>
        </div>

        <div className='mt-5 flex gap-2'>
          <Button
            disabled={
              strikeAssetId === undefined ||
              underlyingAssetId === undefined ||
              !strikeAmount ||
              !underlyingAmount ||
              expirationTimestamp(expiration) <= nowSeconds()
            }
            onClick={mint}
          >
            <Trans>Mint Option</Trans>
          </Button>
        </div>

        <ConfirmationDialog
          response={response}
          close={() => setResponse(null)}
          onConfirm={() => navigate('/options')}
          showRecipientDetails={false}
          additionalData={{
            title: t`Option Details`,
            content: (
              <MintOptionConfirmation
                underlyingAsset={underlyingAsset}
                underlyingAmount={underlyingAmount}
                strikeAsset={strikeAsset}
                strikeAmount={strikeAmount}
                expirationSeconds={mintedExpirationSeconds}
              />
            ),
          }}
        />
      </Container>
    </>
  );
}
