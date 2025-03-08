import { OfferState, useOfferState } from '@/state';
import { useCallback, useMemo } from 'react';
import { useDefaultOfferExpiry } from './useDefaultOfferExpiry';

export default function useOfferStateWithDefault() {
  const { expiry } = useDefaultOfferExpiry();

  const offerState = useOfferState();

  const state: OfferState = useMemo(
    () =>
      offerState ?? {
        fee: '',
        offered: {
          xch: '',
          cats: [],
          nfts: [],
        },
        requested: {
          xch: '',
          cats: [],
          nfts: [],
        },
        expiration: expiry.enabled ? expiry : null,
      },
    [expiry, offerState],
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
