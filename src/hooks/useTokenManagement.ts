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
import { SplitTokenConfirmation } from '@/components/confirmations/SplitTokenConfirmation';
import { CombineTokenConfirmation } from '@/components/confirmations/CombineTokenConfirmation';
import React from 'react';

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
    const [response, setResponse] = useState<EnhancedTransactionResponse | null>(null);
    const [selectedCoins, setSelectedCoins] = useState<RowSelectionState>({});
    const [isReceiveOpen, setReceiveOpen] = useState(false);
    const { receive_address } = walletState.sync;

    const precision = useMemo(
        () => (assetId === 'xch' ? walletState.sync.unit.decimals : 3),
        [assetId, walletState.sync.unit.decimals],
    );

    const balanceInUsd = useMemo(() => {
        if (!asset) return '0';
        return getBalanceInUsd(asset.asset_id, toDecimal(asset.balance, precision));
    }, [asset, precision, getBalanceInUsd]);

    const updateCoins = useMemo(
        () => () => {
            const getCoins =
                assetId === 'xch'
                    ? commands.getXchCoins({})
                    : commands.getCatCoins({ asset_id: assetId! });

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

    const getSelectedCoinsInfo = () => {
        const selectedCoinIds = Object.keys(selectedCoins).filter(
            (key) => selectedCoins[key]
        );

        const selectedCoinsList = selectedCoinIds.map(id =>
            coins.find(coin => coin.coin_id === id)
        ).filter(Boolean) as CoinRecord[];

        const totalAmount = selectedCoinsList.reduce(
            (sum, coin) => sum + BigInt(coin.amount),
            BigInt(0)
        );

        return {
            count: selectedCoinIds.length,
            totalAmount: fromMojos(totalAmount.toString(), precision),
            ticker: asset?.ticker || '',
            selectedCoins: selectedCoinsList
        };
    };

    const getSplitHandler = () => {
        const handler = asset?.asset_id === 'xch' ? commands.splitXch : commands.splitCat;

        return async (params: { coin_ids: string[], output_count: number, fee: string }) => {
            const { count, totalAmount, ticker, selectedCoins: selectedCoinsList } = getSelectedCoinsInfo();

            const result = await handler(params).catch(addError);

            if (result) {
                // Add additional context to the response
                const enhancedResult = result as EnhancedTransactionResponse;
                enhancedResult.additionalData = {
                    title: 'Split Details',
                    content: {
                        type: 'split',
                        coins: selectedCoinsList,
                        outputCount: params.output_count,
                        ticker,
                        precision
                    }
                };

                setResponse(enhancedResult);
            }

            return result;
        };
    };

    const getCombineHandler = () => {
        const handler = asset?.asset_id === 'xch' ? commands.combineXch : commands.combineCat;

        return async (params: { coin_ids: string[], fee: string }) => {
            const { count, totalAmount, ticker, selectedCoins: selectedCoinsList } = getSelectedCoinsInfo();

            const result = await handler(params).catch(addError);

            if (result) {
                // Add additional context to the response
                const enhancedResult = result as EnhancedTransactionResponse;
                enhancedResult.additionalData = {
                    title: 'Combine Details',
                    content: {
                        type: 'combine',
                        coins: selectedCoinsList,
                        ticker,
                        precision
                    }
                };

                setResponse(enhancedResult);
            }

            return result;
        };
    };

    return {
        asset,
        coins,
        precision,
        balanceInUsd,
        response,
        selectedCoins,
        isReceiveOpen,
        receive_address,
        setResponse,
        setSelectedCoins,
        setReceiveOpen,
        redownload,
        setVisibility,
        updateCatDetails,
        getSplitHandler,
        getCombineHandler,
    };
} 