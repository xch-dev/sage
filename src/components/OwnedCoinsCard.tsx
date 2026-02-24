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
import { fromMojos, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { MergeIcon, SplitIcon, XIcon } from 'lucide-react';
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

interface OwnedCoinsCardProps {
  asset: TokenRecord;
  setResponse: (response: TransactionResponse) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: Dispatch<SetStateAction<RowSelectionState>>;
}

export function OwnedCoinsCard({
  asset,
  setResponse,
  selectedCoins,
  setSelectedCoins,
}: OwnedCoinsCardProps) {
  const walletState = useWalletState();

  const { addError } = useErrors();

  const [selectedCoinRecords, setSelectedCoinRecords] = useState<CoinRecord[]>(
    [],
  );
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [totalCoins, setTotalCoins] = useState<number>(0);
  const [sortMode, setSortMode] = useState<CoinSortMode>('created_height');
  const [sortDirection, setSortDirection] = useState<boolean>(false); // false = descending, true = ascending
  const [includeSpentCoins, setIncludeSpentCoins] = useState<boolean>(false);
  const pageSize = 10;

  // Use ref to track current page to avoid dependency issues
  const currentPageRef = useRef(currentPage);
  currentPageRef.current = currentPage;

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
      if (selectedCoinIds.length === 0) {
        try {
          const spendable = await commands.getSpendableCoinCount({
            asset_id: asset.asset_id,
          });
          if (isMounted) {
            setCanAutoCombine(spendable.count > 1);
          }
        } catch {
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
            filter_mode: includeSpentCoins ? 'spent' : 'owned',
          })
          .then((res) => {
            setCoins(res.coins);
            setTotalCoins(res.total);
          })
          .catch(addError);
      },
    [
      asset.asset_id,
      addError,
      pageSize,
      sortMode,
      sortDirection,
      includeSpentCoins,
    ],
  );

  useEffect(() => {
    updateCoins();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
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

  const [combineOpen, setCombineOpen] = useState(false);
  const [splitOpen, setSplitOpen] = useState(false);
  const [autoCombineOpen, setAutoCombineOpen] = useState(false);

  const combineFormSchema = z.object({
    combineFee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const combineForm = useForm<z.infer<typeof combineFormSchema>>({
    resolver: zodResolver(combineFormSchema),
  });

  const onCombineSubmit = (values: z.infer<typeof combineFormSchema>) => {
    const fee = toMojos(values.combineFee, walletState.sync.unit.precision);

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
            title: t`Combine Details`,
            content: {
              type: 'combine',
              coins: selectedCoinRecords,
              ticker: asset.ticker,
              precision: asset.precision,
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
    splitFee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const splitForm = useForm<z.infer<typeof splitFormSchema>>({
    resolver: zodResolver(splitFormSchema),
    defaultValues: {
      outputCount: 2,
    },
  });

  const onSplitSubmit = (values: z.infer<typeof splitFormSchema>) => {
    const fee = toMojos(values.splitFee, walletState.sync.unit.precision);

    // Get IDs from the selected coin records
    const coinIdsForRequest = selectedCoinRecords.map(
      (record) => record.coin_id,
    );

    commands
      .split({
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
              ticker: asset.ticker,
              precision: asset.precision,
            },
          },
        });

        setResponse(resultWithDetails);
      })
      .catch(addError)
      .finally(() => setSplitOpen(false));
  };

  const autoCombineFormSchema = z.object({
    autoCombineFee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
    maxCoins: amount(0),
    maxCoinAmount: amount(asset.precision).optional(),
  });

  const autoCombineForm = useForm<z.infer<typeof autoCombineFormSchema>>({
    resolver: zodResolver(autoCombineFormSchema),
    defaultValues: {
      maxCoins: '100',
      maxCoinAmount: '',
    },
  });

  const onAutoCombineSubmit = (
    values: z.infer<typeof autoCombineFormSchema>,
  ) => {
    const fee = toMojos(values.autoCombineFee, walletState.sync.unit.precision);
    const maxCoins = values.maxCoins;
    const maxCoinAmount = values.maxCoinAmount
      ? toMojos(values.maxCoinAmount, asset.precision)
      : null;

    (!asset?.asset_id
      ? commands.autoCombineXch
      : (...[req]: Parameters<typeof commands.autoCombineXch>) =>
          commands.autoCombineCat({
            ...req,
            asset_id: asset?.asset_id ?? '',
          }))({
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
              ticker: asset.ticker,
              precision: asset.precision,
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

  // Calculate total value of selected coins
  const selectedCoinsTotal = useMemo(() => {
    if (selectedCoinRecords.length === 0) return '0';

    const totalMojos = selectedCoinRecords.reduce((sum, coin) => {
      return sum.plus(coin.amount);
    }, new BigNumber(0));

    return fromMojos(totalMojos, asset.precision).toString();
  }, [selectedCoinRecords, asset.precision]);

  return (
    <Card className='max-w-full overflow-auto'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Owned Coins</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <CoinList
          clawback={false}
          precision={asset.precision}
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
                disabled={!canSplit}
                onClick={() => setSplitOpen(true)}
              >
                <SplitIcon className='mr-2 h-4 w-4' aria-hidden='true' />{' '}
                <Trans>Split</Trans>
              </Button>
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
                <MergeIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                {!canCombine && canAutoCombine ? (
                  <Trans>Sweep</Trans>
                ) : (
                  <Trans>Combine</Trans>
                )}
              </Button>
            </>
          }
        />
        {selectedCoinCount > 0 && (
          <div className='flex items-center gap-2 mt-2'>
            <Button variant='outline' onClick={() => setSelectedCoins({})}>
              <XIcon className='h-4 w-4 mr-2' aria-hidden='true' />
              <Trans>Clear Selection</Trans>
            </Button>

            <span className='text-muted-foreground text-sm flex items-center'>
              <Trans>
                {selectedCoinCount} {selectedCoinLabel} selected (
                {selectedCoinsTotal} {asset.ticker})
              </Trans>
            </span>
          </div>
        )}
      </CardContent>

      <Dialog open={combineOpen} onOpenChange={setCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Combine {asset.ticker}</Trans>
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

      <Dialog open={splitOpen} onOpenChange={setSplitOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Split {asset.ticker}</Trans>
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

      <Dialog open={autoCombineOpen} onOpenChange={setAutoCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Auto Combine {asset.ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will combine small enough coins automatically, so you
                don&apos;t have to manually select them.
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
                      <FeeAmountInput {...field} />
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
