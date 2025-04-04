import { useState, useEffect, useMemo } from 'react';
import { useErrors } from '@/hooks/useErrors';
import { useWalletState } from '@/state';
import { usePrices } from '@/hooks/usePrices';
import { toDecimal } from '@/lib/utils';
import { RowSelectionState } from '@tanstack/react-table';
import {
  CatRecord,
  CoinRecord,
  commands,
  events,
  TransactionResponse,
  CoinSortMode,
} from '../bindings';

// Extend the TransactionResponse type to include additionalData
interface EnhancedTransactionResponse extends TransactionResponse {
  additionalData?: {
    title: string;
    content: {
      type: 'split' | 'combine';
      coins: CoinRecord[];
      outputCount?: number;
      ticker: string;
      precision: number;
    };
  };
}

export function useTokenState(assetId: string | undefined) {
  const walletState = useWalletState();
  const { getBalanceInUsd } = usePrices();
  const { addError } = useErrors();

  const [asset, setAsset] = useState<CatRecord | null>(null);
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [response, setResponse] = useState<EnhancedTransactionResponse | null>(
    null,
  );
  const [selectedCoins, setSelectedCoins] = useState<RowSelectionState>({});
  const { receive_address } = walletState.sync;
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [totalCoins, setTotalCoins] = useState<number>(0);
  const [sortMode, setSortMode] = useState<CoinSortMode>('created_height');
  const [sortDirection, setSortDirection] = useState<boolean>(false); // false = descending, true = ascending
  const [includeSpentCoins, setIncludeSpentCoins] = useState<boolean>(false);
  const pageSize = 10;

  const precision = useMemo(
    () => (assetId === 'xch' ? walletState.sync.unit.decimals : 3),
    [assetId, walletState.sync.unit.decimals],
  );

  const balanceInUsd = useMemo(() => {
    if (!asset) return '0';
    return getBalanceInUsd(asset.asset_id, toDecimal(asset.balance, precision));
  }, [asset, precision, getBalanceInUsd]);

  const updateCoins = useMemo(
    () =>
      (page: number = currentPage) => {
        const offset = page * pageSize;

        const getCoins =
          assetId === 'xch'
            ? commands.getXchCoins({
                offset,
                limit: pageSize,
                sort_mode: sortMode,
                ascending: sortDirection,
                include_spent_coins: includeSpentCoins,
              })
            : commands.getCatCoins({
                asset_id: assetId!,
                offset,
                limit: pageSize,
                sort_mode: sortMode,
                ascending: sortDirection,
                include_spent_coins: includeSpentCoins,
              });

        getCoins
          .then((res) => {
            setCoins(res.coins);
            setTotalCoins(res.total);
          })
          .catch(addError);
      },
    [
      assetId,
      addError,
      pageSize,
      currentPage,
      sortMode,
      sortDirection,
      includeSpentCoins,
    ],
  );

  const updateCat = useMemo(
    () => () => {
      if (assetId === 'xch') return;

      commands
        .getCat({ asset_id: assetId! })
        .then((res) => setAsset(res.cat))
        .catch(addError);
    },
    [assetId, addError],
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

  useEffect(() => {
    if (assetId === 'xch') {
      setAsset({
        asset_id: 'xch',
        name: 'Chia',
        description: 'The native token of the Chia blockchain.',
        ticker: walletState.sync.unit.ticker,
        balance: walletState.sync.balance,
        icon_url: 'https://icons.dexie.space/xch.webp',
        visible: true,
      });
    } else {
      updateCat();

      const unlisten = events.syncEvent.listen((event) => {
        const type = event.payload.type;

        if (
          type === 'coin_state' ||
          type === 'puzzle_batch_synced' ||
          type === 'cat_info'
        ) {
          updateCat();
        }
      });

      return () => {
        unlisten.then((u) => u());
      };
    }
  }, [assetId, updateCat, walletState.sync]);

  const redownload = () => {
    if (!assetId || assetId === 'xch') return;

    commands
      .removeCat({ asset_id: assetId })
      .then(() => updateCat())
      .catch(addError);
  };

  const setVisibility = (visible: boolean) => {
    if (!asset || assetId === 'xch') return;
    const updatedAsset = { ...asset, visible };

    commands.updateCat({ record: updatedAsset }).catch(addError);
  };

  const updateCatDetails = async (updatedAsset: CatRecord) => {
    return commands
      .updateCat({ record: updatedAsset })
      .then(() => updateCat())
      .catch(addError);
  };

  // Add effect to update coins when page changes
  useEffect(() => {
    updateCoins(currentPage);
  }, [currentPage, updateCoins]);

  // Reset to page 0 when sort parameters change
  useEffect(() => {
    setCurrentPage(0);
  }, [sortMode, sortDirection, includeSpentCoins]);

  return {
    asset,
    coins,
    precision,
    balanceInUsd,
    response,
    selectedCoins,
    receive_address,
    currentPage,
    totalCoins,
    pageSize,
    sortMode,
    sortDirection,
    includeSpentCoins,
    setResponse,
    setSelectedCoins,
    setCurrentPage,
    setSortMode,
    setSortDirection,
    setIncludeSpentCoins,
    redownload,
    setVisibility,
    updateCatDetails,
    updateCoins,
  };
}
