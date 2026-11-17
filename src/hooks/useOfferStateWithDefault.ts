import { isAutoFee } from '@/lib/fees';
import { OfferState, useOfferState } from '@/state';
import { useCallback, useMemo } from 'react';
import { useDefaultFee } from './useDefaultFee';
import { useDefaultOfferExpiry } from './useDefaultOfferExpiry';
import { useDefaultOfferSplit } from './useDefaultOfferSplit';

export default function useOfferStateWithDefault() {
  const { expiry } = useDefaultOfferExpiry();
  const { splitByDefault } = useDefaultOfferSplit();
  const { fee: defaultFee } = useDefaultFee();

  const offerState = useOfferState();

  const state: OfferState = useMemo(
    () =>
      offerState ?? {
        fee: '',
        offered: {
          tokens: [],
          nfts: [],
          options: [],
        },
        requested: {
          tokens: [],
          nfts: [],
          options: [],
        },
        expiration: expiry.enabled ? expiry : null,
        splitNftOffers: false,
        preSplit: splitByDefault,
        // Offers pay an entered fee, since their cost isn't known until they're taken.
        preSplitFee: isAutoFee(defaultFee) ? '' : defaultFee,
      },
    [expiry, offerState, splitByDefault, defaultFee],
  );

  const setState = useCallback(
    (newState: Partial<OfferState> | null) => {
      if (!newState) {
        useOfferState.setState(null);
        return;
      }

      if (!offerState) {
        newState = {
          ...state,
          ...newState,
        };
      }
      useOfferState.setState(newState);
    },
    [offerState, state],
  );

  return [state, setState] as const;
}
