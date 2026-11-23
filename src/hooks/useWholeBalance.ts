import { TokenRecord } from '@/bindings';
import { toDecimal } from '@/lib/utils';
import BigNumber from 'bignumber.js';
import { useCallback, useEffect, useState } from 'react';

interface UseWholeBalanceProps {
  /** The asset being spent, once it's loaded. */
  asset: TokenRecord | null;
  /**
   * Whether the whole balance can be spent, with the fee taken out of it. That's only the case
   * when the fee is paid in the same asset, which is XCH.
   */
  enabled: boolean;
  /** Sets the amount field, which holds the balance while the whole balance is spent. */
  setAmount: (amount: string) => void;
}

export interface WholeBalance {
  /** Whether the whole balance is being spent. */
  active: boolean;
  /** For the amount input's max button, if the whole balance can be spent. */
  onWholeBalanceChange?: (wholeBalance: boolean) => void;
  /** The balance to spend, in mojos, while the whole balance is being spent. */
  mojos: string | null;
  /** Goes back to an entered amount, without changing the amount field. */
  reset: () => void;
}

/**
 * Lets an XCH amount be the whole balance, with a fee that isn't known until the transaction is
 * built taken out of it. The amount field holds the balance so that it validates, and follows
 * it as it changes. Spending the whole balance stops, clearing the amount, as soon as it's no
 * longer possible, such as when the balance runs out.
 */
export function useWholeBalance({
  asset,
  enabled,
  setAmount,
}: UseWholeBalanceProps): WholeBalance {
  const [wholeBalance, setWholeBalance] = useState(false);

  const available =
    enabled && asset !== null && BigNumber(asset.selectable_balance).gt(0);
  const active = wholeBalance && available;
  const balance = asset
    ? toDecimal(asset.selectable_balance, asset.precision)
    : '';

  useEffect(() => {
    if (wholeBalance && !available) {
      setWholeBalance(false);
      setAmount('');
    }
  }, [wholeBalance, available, setAmount]);

  useEffect(() => {
    if (active) {
      setAmount(balance);
    }
  }, [active, balance, setAmount]);

  const change = useCallback(
    (whole: boolean) => {
      setWholeBalance(whole);

      if (!whole) {
        setAmount('');
      }
    },
    [setAmount],
  );

  const reset = useCallback(() => setWholeBalance(false), []);

  return {
    active,
    onWholeBalanceChange: available ? change : undefined,
    mojos:
      active && asset ? BigNumber(asset.selectable_balance).toFixed() : null,
    reset,
  };
}
