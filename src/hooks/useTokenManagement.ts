import { useState, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { useErrors } from '@/hooks/useErrors';
import { useWalletState } from '@/state';
import { usePrices } from '@/hooks/usePrices';
import { toDecimal, fromMojos } from '@/lib/utils';
import { RowSelectionState } from '@tanstack/react-table';
import {
  CatRecord,
  CoinRecord,
  commands,
  events,
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

export function useTokenManagement(assetId: string | undefined) {
  const navigate = useNavigate();
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

  const precision = useMemo(
    () => (assetId === 'xch' ? walletState.sync.unit.decimals : 3),
    [assetId, walletState.sync.unit.decimals],
  );

  const balanceInUsd = useMemo(() => {
    if (!asset) return '0';
    return getBalanceInUsd(asset.asset_id, toDecimal(asset.balance, precision));
  }, [asset, precision, getBalanceInUsd]);

  const updateAllCoins = useMemo(
    () => () => {
      const getCoins =
        assetId === 'xch'
          ? commands.getXchCoins({ offset: 0, limit: 10 })
          : commands.getCatCoins({ asset_id: assetId!, offset: 0, limit: 10 });

      getCoins.then((res) => setCoins(res.coins)).catch(addError);
    },
    [assetId, addError],
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
    updateAllCoins();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
        updateAllCoins();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateAllCoins]);

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

    commands
      .updateCat({ record: updatedAsset })
      .then(() => navigate('/wallet'))
      .catch(addError);
  };

  const updateCatDetails = async (updatedAsset: CatRecord) => {
    return commands
      .updateCat({ record: updatedAsset })
      .then(() => updateCat())
      .catch(addError);
  };

  return {
    asset,
    coins,
    precision,
    balanceInUsd,
    response,
    selectedCoins,
    receive_address,
    setResponse,
    setSelectedCoins,
    redownload,
    setVisibility,
    updateCatDetails,
  };
}
