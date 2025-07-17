import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { RowSelectionState } from '@tanstack/react-table';
import { useEffect, useMemo, useState } from 'react';
import {
  CoinRecord,
  CoinSortMode,
  commands,
  events,
  TokenRecord,
  TransactionResponse,
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

  const [asset, setAsset] = useState<TokenRecord | null>(null);
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

  const balanceInUsd = useMemo(() => {
    if (!asset) return '0';
    return getBalanceInUsd(
      asset.asset_id,
      toDecimal(asset.balance, asset.precision),
    );
  }, [asset, getBalanceInUsd]);

  const updateCoins = useMemo(
    () =>
      (page: number = currentPage) => {
        const offset = page * pageSize;

        commands
          .getCoins({
            asset_id: assetId === 'xch' ? null : assetId,
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
        .getCat({ asset_id: assetId ?? '' })
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
      commands.getXchToken({}).then((res) => {
        setAsset(res.xch);
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
      .resyncCat({ asset_id: assetId })
      .then(() => updateCat())
      .catch(addError);
  };

  const setVisibility = (visible: boolean) => {
    if (!asset || assetId === 'xch') return;

    commands.updateCat({ record: { ...asset, visible } }).catch(addError);
  };

  const updateCatDetails = async (updatedAsset: TokenRecord) => {
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
