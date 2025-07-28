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
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { UndoIcon, XIcon } from 'lucide-react';
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

export function ClawbackCoinsCard({
  asset,
  setResponse,
  selectedCoins,
  setSelectedCoins,
}: ClawbackCoinsCardProps) {
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

  const [canClawBack, setCanClawBack] = useState(false);

  useEffect(() => {
    let isMounted = true;

    const checkSpendable = async () => {
      if (selectedCoinIds.length === 0) {
        if (isMounted) {
          setCanClawBack(false);
        }
        return;
      }

      try {
        const isSpendable = await commands.getAreCoinsSpendable({
          coin_ids: selectedCoinIds,
        });

        if (isMounted) {
          setCanClawBack(selectedCoinIds.length > 0 && isSpendable.spendable);
        }
      } catch (error) {
        console.error('Error checking if coins are spendable:', error);
        if (isMounted) {
          setCanClawBack(false);
        }
      }
    };

    checkSpendable();

    return () => {
      isMounted = false;
    };
  }, [selectedCoinIds]);

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
          })
          .catch(addError);
      },
    [asset.asset_id, addError, pageSize, sortMode, sortDirection],
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

  const [clawBackOpen, setClawBackOpen] = useState(false);

  const clawBackFormSchema = z.object({
    clawBackFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const clawBackForm = useForm<z.infer<typeof clawBackFormSchema>>({
    resolver: zodResolver(clawBackFormSchema),
  });

  const onClawBackSubmit = (values: z.infer<typeof clawBackFormSchema>) => {
    const fee = toMojos(values.clawBackFee, walletState.sync.unit.decimals);

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
            title: t`Claw back Details`,
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
      .catch(addError)
      .finally(() => setClawBackOpen(false));
  };

  const pageCount = Math.ceil(totalCoins / pageSize);
  const selectedCoinCount = selectedCoinIds.length;
  const selectedCoinLabel = selectedCoinCount === 1 ? t`coin` : t`coins`;

  if (!totalCoins) return null;

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
    </Card>
  );
}
