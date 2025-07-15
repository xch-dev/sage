import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { useEffect, useMemo, useState } from 'react';
import { commands, events, TokenRecord } from '../bindings';
import { TokenRecordWithPrices } from '../types/TokenViewProps';

export function useXchToken() {
  const walletState = useWalletState();
  const { getBalanceInUsd, getPriceInUsd } = usePrices();
  const { addError } = useErrors();

  const [xchToken, setXchToken] = useState<TokenRecord | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const updateXchToken = useMemo(
    () => async () => {
      try {
        setIsLoading(true);
        const response = await commands.getXchToken({});
        setXchToken(response.xch);
      } catch (error) {
        addError({
          kind: 'api',
          reason:
            error instanceof Error
              ? error.message
              : 'Failed to fetch XCH token',
        });
      } finally {
        setIsLoading(false);
      }
    },
    [addError],
  );

  const xchTokenWithPrices = useMemo((): TokenRecordWithPrices | null => {
    if (!xchToken) return null;

    const balanceInUsd = Number(
      getBalanceInUsd(
        'xch',
        toDecimal(walletState.sync.balance, walletState.sync.unit.decimals),
      ),
    );

    return {
      ...xchToken,
      balanceInUsd,
      priceInUsd: getPriceInUsd('xch'),
      decimals: walletState.sync.unit.decimals,
      isXch: true,
    };
  }, [
    xchToken,
    walletState.sync.balance,
    walletState.sync.unit.decimals,
    getBalanceInUsd,
    getPriceInUsd,
  ]);

  useEffect(() => {
    updateXchToken();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'cat_info'
      ) {
        updateXchToken();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateXchToken]);

  return {
    xchToken,
    xchTokenWithPrices,
    isLoading,
    updateXchToken,
  };
}
