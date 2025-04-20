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
import { Input } from '@/components/ui/input';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { OnChangeFn, RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { MergeIcon, SplitIcon, XIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import * as z from 'zod';
import {
  CatRecord,
  CoinRecord,
  CoinSortMode,
  commands,
  TransactionResponse,
} from '../bindings';

interface CoinsCardProps {
  precision: number;
  coins: CoinRecord[];
  asset: CatRecord | null;
  splitHandler: typeof commands.splitXch | null;
  combineHandler: typeof commands.combineXch | null;
  autoCombineHandler: typeof commands.autoCombineXch | null;
  setResponse: (response: TransactionResponse) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
  onRowSelectionChange?: OnChangeFn<RowSelectionState>;
  currentPage: number;
  totalCoins: number;
  pageSize: number;
  setCurrentPage: (page: number) => void;
  sortMode: CoinSortMode;
  sortDirection: boolean;
  includeSpentCoins: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  onIncludeSpentCoinsChange: (include: boolean) => void;
}

export function CoinsCard({
  precision,
  coins: pageCoins,
  asset,
  splitHandler,
  combineHandler,
  autoCombineHandler,
  setResponse,
  selectedCoins,
  setSelectedCoins,
  onRowSelectionChange,
  currentPage,
  totalCoins,
  pageSize,
  setCurrentPage,
  sortMode,
  sortDirection,
  includeSpentCoins,
  onSortModeChange,
  onSortDirectionChange,
  onIncludeSpentCoinsChange,
}: CoinsCardProps) {
  const walletState = useWalletState();
  const ticker = asset?.ticker;

  const { addError } = useErrors();

  // Store selected CoinRecords instead of just IDs
  const [selectedCoinRecords, setSelectedCoinRecords] = useState<CoinRecord[]>(
    [],
  );

  const selectedCoinIds = useMemo(() => {
    return Object.keys(selectedCoins).filter((key) => selectedCoins[key]);
  }, [selectedCoins]);

  // Update selectedCoinRecords when selection changes
  useEffect(() => {
    // Find records in current page
    const currentPageRecords = selectedCoinIds
      .map((id) => pageCoins.find((coin) => coin.coin_id === id))
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
  }, [selectedCoinIds, pageCoins]);

  const [canCombine, setCanCombine] = useState(false);
  const [canSplit, setCanSplit] = useState(false);
  const [canAutoCombine, setCanAutoCombine] = useState(false);

  useEffect(() => {
    let isMounted = true;

    const checkSpendable = async () => {
      if (selectedCoinIds.length === 0) {
        if (isMounted) {
          setCanSplit(false);
          setCanCombine(false);
        }
        return;
      }

      try {
        const isSpendable = await commands.getAreCoinsSpendable({
          coin_ids: selectedCoinIds,
        });

        if (isMounted) {
          setCanSplit(selectedCoinIds.length >= 1 && isSpendable.spendable);
          setCanCombine(selectedCoinIds.length >= 2 && isSpendable.spendable);
        }
      } catch (error) {
        console.error('Error checking if coins are spendable:', error);
        if (isMounted) {
          setCanSplit(false);
          setCanCombine(false);
        }
      }
    };

    checkSpendable();

    return () => {
      isMounted = false;
    };
  }, [selectedCoinIds]);

  useEffect(() => {
    let isMounted = true;

    const checkAutoCombine = async () => {
      if (selectedCoinIds.length === 0 && asset?.asset_id) {
        try {
          const spendable = await commands.getSpendableCoinCount({
            asset_id: asset.asset_id,
          });
          if (isMounted) {
            setCanAutoCombine(spendable.count > 1);
          }
        } catch (error) {
          if (isMounted) {
            setCanAutoCombine(false);
          }
        }
      } else {
        if (isMounted) {
          setCanAutoCombine(false);
        }
      }
    };

    checkAutoCombine();

    return () => {
      isMounted = false;
    };
  }, [selectedCoinIds, asset]);

  const [isCombineOpen, setCombineOpen] = useState(false);
  const [isSplitOpen, setSplitOpen] = useState(false);
  const [isAutoCombineOpen, setAutoCombineOpen] = useState(false);

  const combineFormSchema = z.object({
    combineFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const combineForm = useForm<z.infer<typeof combineFormSchema>>({
    resolver: zodResolver(combineFormSchema),
    defaultValues: {
      combineFee: '0',
    },
  });

  const onCombineSubmit = (values: z.infer<typeof combineFormSchema>) => {
    if (!combineHandler) return;

    const fee = toMojos(values.combineFee, walletState.sync.unit.decimals);

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    combineHandler({
      coin_ids: coinIdsForRequest,
      fee,
    })
      .then((result) => {
        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Combine Details`,
            content: {
              type: 'combine',
              coins: selectedCoinRecords,
              ticker: ticker || '',
              precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(addError)
      .finally(() => setCombineOpen(false));
  };

  const splitFormSchema = z.object({
    outputCount: z.number().int().min(2).max(4294967295),
    splitFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const splitForm = useForm<z.infer<typeof splitFormSchema>>({
    resolver: zodResolver(splitFormSchema),
    defaultValues: {
      outputCount: 2,
      splitFee: '0',
    },
  });

  const onSplitSubmit = (values: z.infer<typeof splitFormSchema>) => {
    if (!splitHandler) return;

    const fee = toMojos(values.splitFee, walletState.sync.unit.decimals);

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    splitHandler({
      coin_ids: coinIdsForRequest,
      output_count: values.outputCount,
      fee,
    })
      .then((result) => {
        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Split Details`,
            content: {
              type: 'split',
              coins: selectedCoinRecords,
              outputCount: values.outputCount,
              ticker: ticker || '',
              precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(addError)
      .finally(() => setSplitOpen(false));
  };

  const autoCombineFormSchema = z.object({
    autoCombineFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
    maxCoins: amount(0),
    maxCoinAmount: amount(precision).optional(),
  });

  const autoCombineForm = useForm<z.infer<typeof autoCombineFormSchema>>({
    resolver: zodResolver(autoCombineFormSchema),
    defaultValues: {
      autoCombineFee: '0',
      maxCoins: '100',
      maxCoinAmount: '',
    },
  });

  const onAutoCombineSubmit = (
    values: z.infer<typeof autoCombineFormSchema>,
  ) => {
    if (!autoCombineHandler) return;

    const fee = toMojos(values.autoCombineFee, walletState.sync.unit.decimals);
    const maxCoins = values.maxCoins;
    const maxCoinAmount = values.maxCoinAmount
      ? toMojos(values.maxCoinAmount, precision)
      : null;

    autoCombineHandler({
      max_coins: parseInt(toMojos(maxCoins, 0)),
      max_coin_amount: maxCoinAmount,
      fee,
    })
      .then(async (result) => {
        // Find coin records for the returned coin IDs
        const resultCoins = await commands.getCoinsByIds({
          coin_ids: result.coin_ids,
        });

        // Add confirmation data to the response
        const resultWithDetails = Object.assign({}, result, {
          additionalData: {
            title: t`Combine Details`,
            content: {
              type: 'combine',
              coins: resultCoins.coins,
              ticker: ticker || '',
              precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(addError)
      .finally(() => setAutoCombineOpen(false));
  };

  const pageCount = Math.ceil(totalCoins / pageSize);
  const selectedCoinCount = selectedCoinIds.length;
  const selectedCoinLabel = selectedCoinCount === 1 ? t`coin` : t`coins`;

  return (
    <Card className='max-w-full overflow-auto'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Coins</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <CoinList
          precision={precision}
          coins={pageCoins}
          selectedCoins={selectedCoins}
          setSelectedCoins={setSelectedCoins}
          onRowSelectionChange={onRowSelectionChange}
          currentPage={currentPage}
          totalPages={pageCount}
          setCurrentPage={setCurrentPage}
          maxRows={totalCoins}
          sortMode={sortMode}
          sortDirection={sortDirection}
          includeSpentCoins={includeSpentCoins}
          onSortModeChange={onSortModeChange}
          onSortDirectionChange={onSortDirectionChange}
          onIncludeSpentCoinsChange={onIncludeSpentCoinsChange}
          actions={
            <>
              {splitHandler && (
                <Button
                  variant='outline'
                  disabled={!canSplit}
                  onClick={() => setSplitOpen(true)}
                >
                  <SplitIcon className='mr-2 h-4 w-4' /> <Trans>Split</Trans>
                </Button>
              )}
              {(combineHandler || autoCombineHandler) && (
                <Button
                  variant='outline'
                  disabled={!(canCombine || canAutoCombine)}
                  onClick={() => {
                    if (canCombine) {
                      setCombineOpen(true);
                    } else if (canAutoCombine) {
                      setAutoCombineOpen(true);
                    }
                  }}
                >
                  <MergeIcon className='mr-2 h-4 w-4' />
                  {!canCombine && canAutoCombine ? (
                    <Trans>Sweep</Trans>
                  ) : (
                    <Trans>Combine</Trans>
                  )}
                </Button>
              )}
            </>
          }
        />

        {selectedCoinCount > 0 && (
          <div className='flex items-center gap-2'>
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

      <Dialog open={isCombineOpen} onOpenChange={setCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Combine {ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will combine all of the selected coins into one.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...combineForm}>
            <form
              onSubmit={combineForm.handleSubmit(onCombineSubmit)}
              className='space-y-4'
            >
              <FormField
                control={combineForm.control}
                name='combineFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setCombineOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Combine</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={isSplitOpen} onOpenChange={setSplitOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Split {ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>This will split all of the selected coins.</Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...splitForm}>
            <form
              onSubmit={splitForm.handleSubmit(onSplitSubmit)}
              className='space-y-4'
            >
              <FormField
                control={splitForm.control}
                name='outputCount'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Output Count</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input
                        type='number'
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseInt(e.target.value))
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={splitForm.control}
                name='splitFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setSplitOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Split</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={isAutoCombineOpen} onOpenChange={setAutoCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Auto Combine {ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will combine small enough coins automatically, so you don't
                have to manually select them.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Form {...autoCombineForm}>
            <form
              onSubmit={autoCombineForm.handleSubmit(onAutoCombineSubmit)}
              className='space-y-4'
            >
              <FormField
                control={autoCombineForm.control}
                name='autoCombineFee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={autoCombineForm.control}
                name='maxCoins'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Maximum Number of Coins</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={autoCombineForm.control}
                name='maxCoinAmount'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Maximum Coin Amount</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder='Leave blank for no limit'
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setAutoCombineOpen(false)}
                >
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Combine</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
