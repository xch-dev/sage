import { commands, TokenRecord } from '@/bindings';
import Container from '@/components/Container';
import { MakeOfferConfirmationDialog } from '@/components/dialogs/MakeOfferConfirmationDialog';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';
import Header from '@/components/Header';
import { TokenSelector } from '@/components/selectors/TokenSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, TokenAmountInput } from '@/components/ui/masked-input';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { toDecimal, toMojos } from '@/lib/utils';
import { OfferState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { ArrowUpToLine, HandCoins, Handshake } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function Swap() {
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [ownedTokens, setOwnedTokens] = useState<TokenRecord[]>([]);

  const [payAssetId, setPayAssetId] = useState<string | null | undefined>();
  const [payAmount, setPayAmount] = useState('');

  const [receiveAssetId, setReceiveAssetId] = useState<
    string | null | undefined
  >();
  const [receiveAmount, setReceiveAmount] = useState('');

  const [fee, setFee] = useState('');
  const [hasUserInputFee, setHasUserInputFee] = useState(false);

  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [isProgressDialogOpen, setIsProgressDialogOpen] = useState(false);

  const updateCats = useCallback(() => {
    Promise.all([commands.getCats({}), commands.getToken({ asset_id: null })])
      .then(([data, xch]) =>
        setOwnedTokens([...(xch.token ? [xch.token] : []), ...data.cats]),
      )
      .catch(console.error);
  }, []);

  useEffect(() => {
    updateCats();

    const interval = setInterval(updateCats, 15_000);

    return () => clearInterval(interval);
  }, [updateCats]);

  const setMaxTokenAmount = () => {
    const token = ownedTokens.find((t) => t.asset_id === payAssetId);
    if (token) {
      setPayAmount(toDecimal(token.balance, token.precision));
    }
  };

  const updateReceiveAmount = useCallback(
    async (receiveAssetId: string | null, payAmount: string) => {
      const mojoAmount = toMojos(payAmount, payAssetId === null ? 12 : 3);

      if (
        payAssetId === undefined ||
        receiveAssetId === undefined ||
        !mojoAmount ||
        mojoAmount === 'NaN' ||
        BigNumber(mojoAmount).lte(0)
      ) {
        setReceiveAmount('');
        if (!hasUserInputFee) setFee('');
        return;
      }

      const quote = await getDexieQuote(
        payAssetId,
        receiveAssetId,
        mojoAmount,
        'pay',
      );

      if (!quote) {
        addError({
          kind: 'dexie',
          reason: 'Failed to get quote from Dexie. Please try again later.',
        });
        return;
      }

      setReceiveAmount(
        toDecimal(quote.amount, receiveAssetId === null ? 12 : 3),
      );

      if (!hasUserInputFee) {
        setFee(toDecimal(quote.networkFee, 12));
      }
    },
    [payAssetId, hasUserInputFee, addError],
  );

  const updatePayAmount = useCallback(
    async (payAssetId: string | null, receiveAmount: string) => {
      const mojoAmount = toMojos(
        receiveAmount,
        receiveAssetId === null ? 12 : 3,
      );

      if (
        payAssetId === undefined ||
        receiveAssetId === undefined ||
        !mojoAmount ||
        mojoAmount === 'NaN' ||
        BigNumber(mojoAmount).lte(0)
      ) {
        setPayAmount('');
        if (!hasUserInputFee) setFee('');
        return;
      }

      const quote = await getDexieQuote(
        payAssetId,
        receiveAssetId,
        mojoAmount,
        'receive',
      );

      if (!quote) {
        addError({
          kind: 'dexie',
          reason: 'Failed to get quote from Dexie. Please try again later.',
        });
        return;
      }

      setPayAmount(toDecimal(quote.amount, payAssetId === null ? 12 : 3));

      if (!hasUserInputFee) {
        setFee(toDecimal(quote.networkFee, 12));
      }
    },
    [receiveAssetId, hasUserInputFee, addError],
  );

  const offerState = useMemo<OfferState>(() => {
    if (
      payAssetId === undefined ||
      receiveAssetId === undefined ||
      !payAmount ||
      !receiveAmount
    )
      return {
        offered: { tokens: [], nfts: [], options: [] },
        requested: { tokens: [], nfts: [], options: [] },
        fee: '0',
        expiration: null,
      };

    return {
      offered: {
        tokens: [{ asset_id: payAssetId, amount: payAmount }],
        nfts: [],
        options: [],
      },
      requested: {
        tokens: [{ asset_id: receiveAssetId, amount: receiveAmount }],
        nfts: [],
        options: [],
      },
      fee: fee || '0',
      expiration: { days: '0', hours: '0', minutes: '15' },
    };
  }, [payAssetId, receiveAssetId, payAmount, receiveAmount, fee]);

  return (
    <>
      <Header title={t`Combined Swap`} />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                <Trans>You Pay</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Select the asset that you want to pay.</Trans>
              </div>
              <div className='flex mt-4'>
                <TokenSelector
                  value={payAssetId}
                  onChange={(value) => {
                    setPayAssetId(value);
                    updatePayAmount(value, receiveAmount);
                  }}
                  className='!rounded-r-none'
                  hideZeroBalance={true}
                  showAllCats={false}
                  includeXch={true}
                  disabled={
                    receiveAssetId === undefined ? undefined : [receiveAssetId]
                  }
                />
                <div className='flex flex-grow-0'>
                  <TokenAmountInput
                    id='underlying-amount'
                    className='!border-l-0 z-10 !rounded-l-none !rounded-r-none w-[150px] h-12'
                    placeholder={t`Amount`}
                    value={payAmount}
                    onChange={(e) => {
                      setPayAmount(e.target.value);
                      if (receiveAssetId) {
                        updateReceiveAmount(receiveAssetId, e.target.value);
                      }
                    }}
                    precision={payAssetId === null ? 12 : 3}
                  />

                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant='outline'
                          className='!border-l-0 !rounded-l-none h-12 px-2 text-xs'
                          onClick={setMaxTokenAmount}
                          disabled={payAssetId === undefined}
                        >
                          <ArrowUpToLine className='h-3 w-3 mr-1' />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <Trans>Use maximum balance</Trans>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
              </div>

              <div className='text-sm text-muted-foreground mt-2'>
                <Trans>Includes a 1% swap fee</Trans>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                <Trans>You Receive</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                <Trans>Select the asset that you want to receive.</Trans>
              </div>

              <div className='flex mt-4'>
                <TokenSelector
                  value={receiveAssetId}
                  onChange={(value) => {
                    setReceiveAssetId(value);
                    updateReceiveAmount(value, payAmount);
                  }}
                  className='!rounded-r-none'
                  hideZeroBalance={false}
                  showAllCats={true}
                  includeXch={true}
                  disabled={payAssetId === undefined ? undefined : [payAssetId]}
                />
                <div className='flex flex-grow-0'>
                  <TokenAmountInput
                    id='strike-amount'
                    className='!border-l-0 z-10 !rounded-l-none w-[184px] h-12'
                    placeholder={t`Amount`}
                    value={receiveAmount}
                    onChange={(e) => {
                      setReceiveAmount(e.target.value);
                      if (payAssetId) {
                        updatePayAmount(payAssetId, e.target.value);
                      }
                    }}
                    precision={receiveAssetId === null ? 12 : 3}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          <div className='flex flex-col space-y-1.5'>
            <Label htmlFor='fee'>
              <Trans>Network Fee</Trans>
            </Label>
            <div className='relative'>
              <FeeAmountInput
                id='fee'
                className='pr-12'
                value={fee}
                onChange={(e) => {
                  setFee(e.target.value);
                  // Hack since it's not specified for the initial value
                  if (e.currentTarget) setHasUserInputFee(true);
                }}
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
        </div>
        <div className='mt-4'>
          <Button
            disabled={
              payAssetId === undefined ||
              receiveAssetId === undefined ||
              !receiveAmount ||
              !payAmount
            }
            onClick={() => setIsConfirmDialogOpen(true)}
          >
            <Trans>Swap</Trans>
          </Button>
        </div>
        <MakeOfferConfirmationDialog
          open={isConfirmDialogOpen}
          onOpenChange={setIsConfirmDialogOpen}
          onConfirm={() => {
            setIsProgressDialogOpen(true);
          }}
          offerState={offerState}
          splitNftOffers={false}
          fee={fee || '0'}
        />

        <OfferCreationProgressDialog
          open={isProgressDialogOpen}
          onOpenChange={(open) => {
            if (!open) {
              setIsProgressDialogOpen(false);
            }
          }}
          offerState={offerState}
          splitNftOffers={false}
          clearOfferState={async (offers) => {
            if (offers.length === 1) {
              if (!(await executeDexieSwap(offers[0], addError))) return;
            }
            navigate('/offers');
          }}
          isSwap
        />
      </Container>
    </>
  );
}

async function getDexieQuote(
  payAssetId: string | null,
  receiveAssetId: string | null,
  amount: string,
  amountKind: 'pay' | 'receive',
) {
  try {
    const response = await fetch(
      `https://api.dexie.space/v1/swap/quote?from=${payAssetId ?? 'XCH'}&to=${receiveAssetId ?? 'XCH'}&${amountKind === 'pay' ? 'from_amount' : 'to_amount'}=${amount || '0'}`,
    );
    const data = await response.json();
    return {
      amount: (amountKind === 'pay'
        ? data.quote.to_amount
        : data.quote.from_amount) as number,
      networkFee: data.quote.suggested_tx_fee as number,
    };
  } catch (error: unknown) {
    console.error(error);
    return null;
  }
}

async function executeDexieSwap(
  offer: string,
  addError: (error: CustomError) => void,
) {
  try {
    const response = await fetch('https://api.dexie.space/v1/swap', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        offer,
        fee_destination:
          'xch1zs4p5vvusras7h0c3ztyulp7qnyrs5c54lnl4km3ur5musmf3e2qyzlr5e',
      }),
      // …
    });
    const data = await response.json();
    if (!data.success) {
      addError({
        kind: 'dexie',
        reason: 'Failed to execute Dexie swap. Please try again later.',
      });
      return false;
    }
    return true;
  } catch (error: unknown) {
    addError({
      kind: 'dexie',
      reason: `Failed to execute Dexie swap: ${error}`,
    });
    return false;
  }
}
