import { commands, events, TokenRecord, TransactionResponse } from '@/bindings';
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
import { useNowSeconds } from '@/hooks/useNowSeconds';
import { useWholeBalance } from '@/hooks/useWholeBalance';
import {
  Expiration,
  expirationError,
  futureExpirationTimestamp,
} from '@/lib/expiration';
import { feeParams } from '@/lib/fees';
import { toDecimal, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon, HandCoins, Handshake } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

/**
 * The mojo the option itself is made of, which minting spends on top of the underlying. It's
 * `OPTION_AMOUNT` in `sage-wallet`.
 */
const OPTION_MOJOS = 1;

export function MintOption() {
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();
  const now = useNowSeconds();

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
  // The underlying amount the minted option locks up, for the confirmation.
  const [mintedUnderlyingAmount, setMintedUnderlyingAmount] = useState('');

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

  // Fetch token details when asset IDs change, and keep the underlying balance current, since
  // the whole of it can be locked up.
  useEffect(() => {
    if (underlyingAssetId === undefined) {
      setUnderlyingAsset(null);
      return;
    }

    const update = () =>
      commands
        .getToken({ asset_id: underlyingAssetId })
        .then((res) => setUnderlyingAsset(res.token))
        .catch(() => setUnderlyingAsset(null));

    update();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
        update();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
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

  // Only an XCH underlying can be the whole balance, since that's the asset the fee is paid in.
  const wholeBalance = useWholeBalance({
    asset: underlyingAsset,
    enabled: underlyingAssetId === null,
    setAmount: setUnderlyingAmount,
  });

  const mint = useCallback(() => {
    if (underlyingAssetId === undefined || strikeAssetId === undefined) {
      return;
    }

    // A picked date can pass while the form is open, so check again here.
    let expirationSeconds: number;
    try {
      expirationSeconds = futureExpirationTimestamp(expiration);
    } catch (error) {
      addError({ kind: 'invalid', reason: (error as Error).message });
      return;
    }

    const precision =
      underlyingAssetId === null ? walletState.sync.unit.precision : 3;
    // The whole balance is locked up with the fee, and the option's own mojo, taken out of it,
    // since the fee isn't known until the transaction is built.
    const underlyingMojos =
      wholeBalance.mojos ?? toMojos(underlyingAmount, precision);

    feeParams(fee, walletState.sync.unit.precision)
      .then((fees) =>
        commands.mintOption({
          ...fees,
          deduct_fee: wholeBalance.active,
          underlying: {
            asset_id: underlyingAssetId,
            amount: underlyingMojos,
          },
          strike: {
            asset_id: strikeAssetId,
            amount: toMojos(
              strikeAmount,
              strikeAssetId === null ? walletState.sync.unit.precision : 3,
            ),
          },
          expiration_seconds: expirationSeconds,
        }),
      )
      .then((response) => {
        setMintedExpirationSeconds(expirationSeconds);
        setMintedUnderlyingAmount(
          wholeBalance.active
            ? toDecimal(
                BigNumber(underlyingMojos)
                  .minus(response.summary.fee)
                  .minus(OPTION_MOJOS)
                  .toFixed(),
                precision,
              )
            : underlyingAmount,
        );
        setResponse(response);
      })
      .catch(addError);
  }, [
    addError,
    fee,
    underlyingAssetId,
    underlyingAmount,
    wholeBalance.mojos,
    wholeBalance.active,
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

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-(--breakpoint-lg)'>
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
                    wholeBalance.reset();
                  }}
                  className='rounded-r-none!'
                  hideZeroBalance={true}
                  showAllCats={false}
                  includeXch={true}
                />
                <TokenAmountInput
                  id='underlying-amount'
                  className='border-l-0! z-10 rounded-l-none! w-[150px] h-12'
                  placeholder={t`Amount`}
                  value={underlyingAmount}
                  onValueChange={(values) => {
                    setUnderlyingAmount(values.value);
                  }}
                  precision={underlyingAssetId === null ? 12 : 3}
                  wholeBalance={wholeBalance.active}
                  onWholeBalanceChange={wholeBalance.onWholeBalanceChange}
                  // An XCH underlying is locked up as the whole balance minus the fee instead,
                  // so the fee doesn't need taking out here.
                  maxValue={
                    underlyingAsset
                      ? toDecimal(
                          underlyingAsset.selectable_balance,
                          underlyingAsset.precision,
                        )
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
                  className='rounded-r-none!'
                  hideZeroBalance={false}
                  showAllCats={true}
                  includeXch={true}
                />
                <div className='flex grow-0'>
                  <TokenAmountInput
                    id='strike-amount'
                    className='border-l-0! z-10 rounded-l-none! w-[150px] h-12'
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

        {/* The same columns as the cards above, so the fee is as wide as one of them. */}
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-(--breakpoint-lg) mt-4'>
          <div className='flex flex-col space-y-1.5'>
            <Label htmlFor='fee'>
              <Trans>Network Fee</Trans>
            </Label>
            <FeeAmountInput
              id='fee'
              value={fee}
              onValueChange={(values) => setFee(values.value)}
            />
          </div>
        </div>

        <div className='mt-5 flex gap-2'>
          <Button
            disabled={
              strikeAssetId === undefined ||
              underlyingAssetId === undefined ||
              !strikeAmount ||
              !underlyingAmount ||
              expirationError(expiration, now) !== undefined
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
                underlyingAmount={mintedUnderlyingAmount}
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
