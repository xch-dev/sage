import CoinList from '@/components/CoinList';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { useNetwork } from '@/hooks/useNetwork';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { RowSelectionState } from '@tanstack/react-table';
import { CheckIcon, HandCoins, UndoIcon, XIcon } from 'lucide-react';
import {
  Dispatch,
  SetStateAction,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import {
  CoinRecord,
  CoinSortMode,
  commands,
  events,
  TokenRecord,
  TransactionResponse,
} from '../bindings';

interface ClawbackCoinsCardProps {
  asset: TokenRecord;
  setResponse: (response: TransactionResponse) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: Dispatch<SetStateAction<RowSelectionState>>;
}

function isClawBackEligible(coin: CoinRecord, now: number): boolean {
  // Unconfirmed coins can't be spent yet.
  if (!coin.clawback_is_sender || coin.created_height == null) return false;
  // V1 sender can claw until claimed; V2 only before absolute expiry.
  if (coin.clawback_version === 1) return true;
  if (coin.clawback_version === 2) {
    return (
      coin.clawback_timestamp != null && now < Number(coin.clawback_timestamp)
    );
  }
  return false;
}

// Pushing through an expired V2 clawback needs no key, so either party can finalize.
function isFinalizeEligible(coin: CoinRecord, now: number): boolean {
  return (
    coin.clawback_version === 2 &&
    coin.created_height != null &&
    coin.clawback_timestamp != null &&
    now >= Number(coin.clawback_timestamp)
  );
}

function isClaimEligible(coin: CoinRecord, now: number): boolean {
  if (
    coin.clawback_version !== 1 ||
    !coin.clawback_is_receiver ||
    coin.clawback_timestamp == null ||
    coin.created_height == null
  ) {
    return false;
  }
  // Match backend: if birth time is not synced yet, allow the attempt and
  // let claim_clawback / the chain enforce the relative lock.
  if (coin.created_timestamp == null) {
    return true;
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
  const { isTransactionDisabled } = useWallet();

  const { addError } = useErrors();
  const { isTestnet } = useNetwork();

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
        (c) => !c.transaction_id && !c.spent_height && !c.offer_id,
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

  const submit =
    (
      type: 'clawback' | 'finalize_clawback' | 'claim_clawback',
      title: string,
      request: (coinIds: string[], fee: string) => Promise<TransactionResponse>,
      setOpen: (open: boolean) => void,
    ) =>
    (fee: string) => {
      request(
        selectedCoinRecords.map((record) => record.coin_id),
        toMojos(fee || '0', walletState.sync.unit.precision),
      )
        .then((result) =>
          setResponse(
            Object.assign({}, result, {
              additionalData: {
                title,
                content: {
                  type,
                  coins: selectedCoinRecords,
                  ticker: asset.ticker,
                  precision: asset.precision,
                },
              },
            }),
          ),
        )
        .catch(addError)
        .finally(() => setOpen(false));
    };

  const pageCount = Math.ceil(totalCoins / pageSize);
  const selectedCoinCount = selectedCoinIds.length;
  const selectedCoinLabel = selectedCoinCount === 1 ? t`coin` : t`coins`;
  const ticker = asset.ticker ?? '';

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
                disabled={isTransactionDisabled || !canClawBack}
                onClick={() => {
                  if (canClawBack) setClawBackOpen(true);
                }}
              >
                <UndoIcon className='mr-2 h-4 w-4' />
                <Trans>Claw Back</Trans>
              </Button>

              <Button
                variant='outline'
                disabled={isTransactionDisabled || !canFinalize}
                onClick={() => {
                  if (canFinalize) setFinalizeOpen(true);
                }}
              >
                <CheckIcon className='mr-2 h-4 w-4' />
                <Trans>Finalize</Trans>
              </Button>

              <Button
                variant='outline'
                disabled={isTransactionDisabled || !canClaim}
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

      <FeeOnlyDialog
        title={t`Claw Back ${ticker}`}
        open={clawBackOpen}
        setOpen={setClawBackOpen}
        onSubmit={submit(
          'clawback',
          t`Claw Back Details`,
          (coin_ids, fee) => commands.combine({ coin_ids, fee }),
          setClawBackOpen,
        )}
        submitButtonLabel={t`Claw Back`}
      >
        <Trans>This will claw back all of the selected coins.</Trans>
      </FeeOnlyDialog>

      <FeeOnlyDialog
        title={t`Finalize ${ticker} Clawback`}
        open={finalizeOpen}
        setOpen={setFinalizeOpen}
        onSubmit={submit(
          'finalize_clawback',
          t`Finalize Clawback Details`,
          (coin_ids, fee) => commands.finalizeClawback({ coin_ids, fee }),
          setFinalizeOpen,
        )}
        submitButtonLabel={t`Finalize`}
      >
        <Trans>
          This will complete the clawback for all of the selected coins, and
          send the funds to the original recipient (even if the recipient wallet
          does not support clawbacks).
        </Trans>
      </FeeOnlyDialog>

      <FeeOnlyDialog
        title={t`Claim ${ticker} Clawback`}
        open={claimOpen}
        setOpen={setClaimOpen}
        onSubmit={submit(
          'claim_clawback',
          t`Claim Clawback Details`,
          (coin_ids, fee) =>
            commands.claimClawback({ coin_ids, fee, auto_submit: false }),
          setClaimOpen,
        )}
        submitButtonLabel={t`Claim`}
      >
        <Trans>
          This will claim all of the selected coins from a v1 clawback. This
          will send the funds to your wallet, and the original sender will no
          longer be able to claw it back.
        </Trans>
      </FeeOnlyDialog>
    </Card>
  );
}
