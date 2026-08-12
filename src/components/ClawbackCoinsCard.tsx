import CoinList from '@/components/CoinList';
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
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { useErrors } from '@/hooks/useErrors';
import { useNetwork } from '@/hooks/useNetwork';
import { amount } from '@/lib/formTypes';
import { fromMojos, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import type { CustomError } from '@/contexts/ErrorContext';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { CheckIcon, HandCoins, UndoIcon, XIcon } from 'lucide-react';
import {
  Dispatch,
  SetStateAction,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { useForm } from 'react-hook-form';
import * as z from 'zod';
import {
  CoinRecord,
  CoinSortMode,
  commands,
  events,
  TokenRecord,
  TransactionResponse,
} from '../bindings';
import { FeeAmountInput } from './ui/masked-input';

interface ClawbackCoinsCardProps {
  asset: TokenRecord;
  setResponse: (response: TransactionResponse) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: Dispatch<SetStateAction<RowSelectionState>>;
}

function isClawBackEligible(coin: CoinRecord, now: number): boolean {
  if (!coin.clawback_is_sender) return false;
  // V1 sender can claw until claimed; V2 only before absolute expiry.
  if (coin.clawback_version === 1) return true;
  if (coin.clawback_version === 2) {
    return (
      coin.clawback_timestamp != null && now < Number(coin.clawback_timestamp)
    );
  }
  return false;
}

function isFinalizeEligible(coin: CoinRecord, now: number): boolean {
  return (
    coin.clawback_version === 2 &&
    !!coin.clawback_is_sender &&
    coin.clawback_timestamp != null &&
    now >= Number(coin.clawback_timestamp)
  );
}

function isClaimEligible(coin: CoinRecord, now: number): boolean {
  if (
    coin.clawback_version !== 1 ||
    !coin.clawback_is_receiver ||
    coin.created_timestamp == null ||
    coin.clawback_timestamp == null
  ) {
    return false;
  }
  return (
    now >= Number(coin.created_timestamp) + Number(coin.clawback_timestamp)
  );
}

export function ClawbackCoinsCard({
  asset,
  setResponse,
  selectedCoins,
  setSelectedCoins,
}: ClawbackCoinsCardProps) {
  const walletState = useWalletState();

  const { addError } = useErrors();
  const { isTestnet } = useNetwork();

  const reportError = (error: unknown) => {
    const e = error as {
      kind?: CustomError['kind'];
      reason?: string;
      message?: string;
    };
    addError({
      kind: e?.kind ?? 'internal',
      reason:
        e?.reason ||
        e?.message ||
        (typeof error === 'string' ? error : null) ||
        t`Something went wrong. Check the logs for details.`,
    });
  };

  const [selectedCoinRecords, setSelectedCoinRecords] = useState<CoinRecord[]>(
    [],
  );
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [totalCoins, setTotalCoins] = useState<number>(0);
  const [hasLoaded, setHasLoaded] = useState(false);
  const [sortMode, setSortMode] = useState<CoinSortMode>('created_height');
  const [sortDirection, setSortDirection] = useState<boolean>(false); // false = descending, true = ascending
  const [includeSpentCoins, setIncludeSpentCoins] = useState<boolean>(false);
  const [canClawBack, setCanClawBack] = useState(false);
  const [clawBackOpen, setClawBackOpen] = useState(false);
  const [finalizeOpen, setFinalizeOpen] = useState(false);
  const [canFinalize, setCanFinalize] = useState(false);
  const [canClaim, setCanClaim] = useState(false);
  const [claimOpen, setClaimOpen] = useState(false);

  const pageSize = 10;

  // Use ref to track current page to avoid dependency issues
  const currentPageRef = useRef(currentPage);
  currentPageRef.current = currentPage;
  const prevSelectedCountRef = useRef(0);

  const selectedCoinIds = useMemo(() => {
    return Object.keys(selectedCoins).filter((key) => selectedCoins[key]);
  }, [selectedCoins]);

  // Update selectedCoinRecords when selection changes
  useEffect(() => {
    // Find records in current page
    const currentPageRecords = selectedCoinIds
      .map((id) => coins.find((coin) => coin.coin_id === id))
      .filter(Boolean) as CoinRecord[];

    // Use functional update to avoid dependency on selectedCoinRecords
    setSelectedCoinRecords((prevRecords) => {
      // Keep existing records that are still selected but not on current page
      const existingSelectedRecords = prevRecords.filter(
        (record) =>
          selectedCoinIds.includes(record.coin_id) &&
          !currentPageRecords.some((r) => r.coin_id === record.coin_id),
      );

      // Combine records from current page with previously selected records
      return [...currentPageRecords, ...existingSelectedRecords];
    });
  }, [selectedCoinIds, coins]);

  useEffect(() => {
    let isMounted = true;

    const checkEligibility = async () => {
      if (
        selectedCoinIds.length === 0 ||
        selectedCoinRecords.length !== selectedCoinIds.length
      ) {
        if (isMounted) {
          setCanClawBack(false);
          setCanFinalize(false);
          setCanClaim(false);
        }
        return;
      }

      const now = Math.floor(Date.now() / 1000);

      const nonePending = selectedCoinRecords.every(
        (c) => !c.transaction_id && !c.spent_height,
      );

      const allClawBack = selectedCoinRecords.every((c) =>
        isClawBackEligible(c, now),
      );
      const allFinalize = selectedCoinRecords.every((c) =>
        isFinalizeEligible(c, now),
      );
      const allClaim = selectedCoinRecords.every((c) =>
        isClaimEligible(c, now),
      );

      let spendable = nonePending;
      if (allClawBack && nonePending) {
        const anyV2 = selectedCoinRecords.some((c) => c.clawback_version === 2);
        if (anyV2) {
          try {
            const result = await commands.getAreCoinsSpendable({
              coin_ids: selectedCoinIds,
            });
            spendable = result.spendable;
          } catch (error) {
            console.error('Error checking if coins are spendable:', error);
            spendable = false;
          }
        }
      }

      if (isMounted) {
        setCanClawBack(allClawBack && spendable);
        setCanFinalize(allFinalize && nonePending);
        setCanClaim(allClaim && nonePending);
      }
    };

    checkEligibility();

    return () => {
      isMounted = false;
    };
  }, [selectedCoinIds, selectedCoinRecords]);

  const updateCoins = useMemo(
    () =>
      (page: number = currentPageRef.current) => {
        const offset = page * pageSize;

        commands
          .getCoins({
            asset_id: asset.asset_id,
            offset,
            limit: pageSize,
            sort_mode: sortMode,
            ascending: sortDirection,
            filter_mode: 'clawback',
          })
          .then((res) => {
            setCoins(res.coins);
            setTotalCoins(res.total);
            setHasLoaded(true);
          })
          .catch(addError);
      },
    [asset.asset_id, addError, pageSize, sortMode, sortDirection],
  );

  useEffect(() => {
    setHasLoaded(false);
    setCoins([]);
    setTotalCoins(0);
    setCurrentPage(0);
  }, [asset.asset_id]);

  useEffect(() => {
    updateCoins();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'transaction_failed'
      ) {
        updateCoins();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateCoins]);

  // Reset to page 0 when sort parameters change
  useEffect(() => {
    setCurrentPage(0);
  }, [sortMode, sortDirection, includeSpentCoins]);

  // Update coins when page changes
  useEffect(() => {
    updateCoins(currentPage);
  }, [currentPage, updateCoins]);

  // Refresh after confirm clears selection (non-empty → empty).
  useEffect(() => {
    const prev = prevSelectedCountRef.current;
    prevSelectedCountRef.current = selectedCoinIds.length;
    if (prev > 0 && selectedCoinIds.length === 0) {
      updateCoins();
    }
  }, [selectedCoinIds.length, updateCoins]);

  const clawBackFormSchema = z.object({
    clawBackFee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const clawBackForm = useForm<z.infer<typeof clawBackFormSchema>>({
    resolver: zodResolver(clawBackFormSchema),
    defaultValues: { clawBackFee: '0' },
  });

  const onClawBackSubmit = (values: z.infer<typeof clawBackFormSchema>) => {
    const fee = toMojos(
      values.clawBackFee || '0',
      walletState.sync.unit.precision,
    );

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    commands
      .combine({
        coin_ids: coinIdsForRequest,
        fee,
      })
      .then((result) => {
        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Claw Back Details`,
            content: {
              type: 'clawback',
              coins: selectedCoinRecords,
              ticker: asset.ticker,
              precision: asset.precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(reportError)
      .finally(() => setClawBackOpen(false));
  };

  const finalizeFormSchema = z.object({
    finalizeFee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const finalizeForm = useForm<z.infer<typeof finalizeFormSchema>>({
    resolver: zodResolver(finalizeFormSchema),
    defaultValues: { finalizeFee: '0' },
  });

  const onFinalizeSubmit = (values: z.infer<typeof finalizeFormSchema>) => {
    const fee = toMojos(
      values.finalizeFee || '0',
      walletState.sync.unit.precision,
    );

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    commands
      .finalizeClawback({
        coin_ids: coinIdsForRequest,
        fee,
      })
      .then((result) => {
        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Finalize Clawback Details`,
            content: {
              type: 'finalize_clawback',
              coins: selectedCoinRecords,
              ticker: asset.ticker,
              precision: asset.precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(reportError)
      .finally(() => setFinalizeOpen(false));
  };

  const claimFormSchema = z.object({
    claimFee: amount(walletState.sync.unit.precision).refine(
      (fee) => {
        const feeDisplay = fee || '0';
        if (BigNumber(feeDisplay).isLessThanOrEqualTo(0)) return true;
        return BigNumber(
          fromMojos(
            walletState.sync.selectable_balance,
            walletState.sync.unit.precision,
          ),
        ).gte(feeDisplay);
      },
      t`Not enough funds to cover the fee`,
    ),
  });

  const claimForm = useForm<z.infer<typeof claimFormSchema>>({
    resolver: zodResolver(claimFormSchema),
    defaultValues: { claimFee: '0' },
  });

  const onClaimSubmit = (values: z.infer<typeof claimFormSchema>) => {
    const fee = toMojos(
      values.claimFee || '0',
      walletState.sync.unit.precision,
    );

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    commands
      .claimClawback({
        coin_ids: coinIdsForRequest,
        fee,
        auto_submit: false,
      })
      .then((result) => {
        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Claim Clawback Details`,
            content: {
              type: 'claim_clawback',
              coins: selectedCoinRecords,
              ticker: asset.ticker,
              precision: asset.precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(reportError)
      .finally(() => setClaimOpen(false));
  };

  const pageCount = Math.ceil(totalCoins / pageSize);
  const selectedCoinCount = selectedCoinIds.length;
  const selectedCoinLabel = selectedCoinCount === 1 ? t`coin` : t`coins`;

  if (!hasLoaded || totalCoins === 0) {
    return null;
  }

  return (
    <Card className='max-w-full overflow-auto'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Clawback Coins</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <CoinList
          clawback={true}
          precision={asset.precision}
          isTestnet={isTestnet}
          coins={coins}
          selectedCoins={selectedCoins}
          setSelectedCoins={setSelectedCoins}
          currentPage={currentPage}
          totalPages={pageCount}
          setCurrentPage={setCurrentPage}
          maxRows={totalCoins}
          sortMode={sortMode}
          sortDirection={sortDirection}
          includeSpentCoins={includeSpentCoins}
          onSortModeChange={setSortMode}
          onSortDirectionChange={setSortDirection}
          onIncludeSpentCoinsChange={setIncludeSpentCoins}
          actions={
            <>
              <Button
                variant='outline'
                disabled={!canClawBack}
                onClick={() => {
                  if (canClawBack) setClawBackOpen(true);
                }}
              >
                <UndoIcon className='mr-2 h-4 w-4' />
                <Trans>Claw Back</Trans>
              </Button>

              <Button
                variant='outline'
                disabled={selectedCoinIds.length === 0 || canClawBack || !canFinalize}
                onClick={() => {
                  if (canFinalize) setFinalizeOpen(true);
                }}
              >
                <CheckIcon className='mr-2 h-4 w-4' />
                <Trans>Finalize</Trans>
              </Button>

              <Button
                variant='outline'
                disabled={selectedCoinIds.length === 0 || canClawBack || !canClaim}
                onClick={() => {
                  if (canClaim) setClaimOpen(true);
                }}
              >
                <HandCoins className='mr-2 h-4 w-4' />
                <Trans>Claim</Trans>
              </Button>
            </>
          }
        />
        {selectedCoinCount > 0 && (
          <div className='flex items-center gap-2 mt-2'>
            <Button variant='outline' onClick={() => setSelectedCoins({})}>
              <XIcon className='h-4 w-4 mr-2' />
              <Trans>Clear Selection</Trans>
            </Button>

            <span className='text-muted-foreground text-sm flex items-center'>
              <Trans>
                {selectedCoinCount} {selectedCoinLabel} selected
              </Trans>
            </span>
          </div>
        )}
      </CardContent>

      <Dialog open={clawBackOpen} onOpenChange={setClawBackOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Claw Back {asset.ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>This will claw back all of the selected coins.</Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...clawBackForm}>
            <form
              onSubmit={clawBackForm.handleSubmit(onClawBackSubmit)}
              className='space-y-4'
            >
              <FormField
                control={clawBackForm.control}
                name='clawBackFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <FeeAmountInput {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setClawBackOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Claw Back</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={finalizeOpen} onOpenChange={setFinalizeOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Finalize {asset.ticker} Clawback</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will complete the clawback for all of the selected coins,
                and send the funds to the original recipient (even if the
                recipient wallet does not support clawbacks).
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...finalizeForm}>
            <form
              onSubmit={finalizeForm.handleSubmit(onFinalizeSubmit)}
              className='space-y-4'
            >
              <FormField
                control={finalizeForm.control}
                name='finalizeFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <FeeAmountInput {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setFinalizeOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Finalize</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      {/* @TODO: Decide whether this should be wrapped up in the Finalize flow, or an additional flow. */}
      <Dialog open={claimOpen} onOpenChange={setClaimOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Claim {asset.ticker} Clawback</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will claim all of the selected coins from an early type of
                clawback. This will send the funds to your wallet, and the original
                sender will no longer be able to claw it back.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...claimForm}>
            <form
              onSubmit={claimForm.handleSubmit(onClaimSubmit)}
              className='space-y-4'
            >
              <FormField
                control={claimForm.control}
                name='claimFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <FeeAmountInput {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setClaimOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Claim</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
