import { commands, TokenRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { MintOptionConfirmation } from '@/components/confirmations/MintOptionConfirmation.tsx';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import {
  FeeAmountInput,
  IntegerInput,
  TokenAmountInput,
} from '@/components/ui/masked-input';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AlertCircleIcon, HandCoins, Handshake } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function MintOption() {
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [fee, setFee] = useState('');
  const [days, setDays] = useState('');
  const [hours, setHours] = useState('');
  const [minutes, setMinutes] = useState('');

  const [underlyingAssetId, setUnderlyingAssetId] = useState<
    string | null | undefined
  >();
  const [underlyingAmount, setUnderlyingAmount] = useState('');

  const [strikeAssetId, setStrikeAssetId] = useState<
    string | null | undefined
  >();
  const [strikeAmount, setStrikeAmount] = useState('');

  const [response, setResponse] = useState<TransactionResponse | null>(null);

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

    const daysInt = parseInt(days) || 0;
    const hoursInt = parseInt(hours) || 0;
    const minutesInt = parseInt(minutes) || 0;
    const expiration =
      Math.ceil(Date.now() / 1000) +
      (daysInt * 24 * 60 * 60 + hoursInt * 60 * 60 + minutesInt * 60);

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
        expiration_seconds: expiration,
      })
      .then(setResponse)
      .catch(addError);
  }, [
    addError,
    fee,
    underlyingAssetId,
    underlyingAmount,
    strikeAssetId,
    strikeAmount,
    walletState.sync.unit.precision,
    days,
    hours,
    minutes,
  ]);

  // Calculate expiration time for display
  const expirationSeconds = useMemo(() => {
    const daysInt = parseInt(days) || 0;
    const hoursInt = parseInt(hours) || 0;
    const minutesInt = parseInt(minutes) || 0;
    return (
      Math.ceil(Date.now() / 1000) +
      (daysInt * 24 * 60 * 60 + hoursInt * 60 * 60 + minutesInt * 60)
    );
  }, [days, hours, minutes]);

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
                  onChange={setUnderlyingAssetId}
                  className='rounded-r-none'
                  hideZeroBalance={true}
                  showAllCats={false}
                  includeXch={true}
                />
                <div className='flex flex-grow-0'>
                  <TokenAmountInput
                    id='underlying-amount'
                    className='border-l-0 z-10 rounded-l-none w-[100px] h-12'
                    placeholder={t`Amount`}
                    value={underlyingAmount}
                    onValueChange={(values) => {
                      setUnderlyingAmount(values.value);
                    }}
                  />
                </div>
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
                  onChange={setStrikeAssetId}
                  className='rounded-r-none'
                  hideZeroBalance={false}
                  showAllCats={true}
                  includeXch={true}
                />
                <div className='flex flex-grow-0'>
                  <TokenAmountInput
                    id='strike-amount'
                    className='border-l-0 z-10 rounded-l-none w-[100px] h-12'
                    placeholder={t`Amount`}
                    value={strikeAmount}
                    onValueChange={(values) => {
                      setStrikeAmount(values.value);
                    }}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          <div className='flex flex-col gap-1'>
            <Trans>Expiration</Trans>

            <div className='flex gap-2'>
              <div className='relative'>
                <IntegerInput
                  className='pr-12'
                  value={days}
                  placeholder='0'
                  min={0}
                  onValueChange={(values) => setDays(values.value)}
                />
                <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                  <span className='text-muted-foreground text-sm'>
                    <Trans>Days</Trans>
                  </span>
                </div>
              </div>

              <div className='relative'>
                <IntegerInput
                  className='pr-12'
                  value={hours}
                  placeholder='0'
                  min={0}
                  onValueChange={(values) => setHours(values.value)}
                />
                <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                  <span className='text-muted-foreground text-sm'>
                    <Trans>Hours</Trans>
                  </span>
                </div>
              </div>

              <div className='relative'>
                <IntegerInput
                  className='pr-12'
                  value={minutes}
                  placeholder='0'
                  min={0}
                  onValueChange={(values) => setMinutes(values.value)}
                />
                <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                  <span className='text-muted-foreground text-sm'>
                    <Trans>Minutes</Trans>
                  </span>
                </div>
              </div>
            </div>
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
              (!days && !hours && !minutes)
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
                expirationSeconds={expirationSeconds}
              />
            ),
          }}
        />
      </Container>
    </>
  );
}
