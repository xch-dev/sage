import { commands, events, OfferRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useCallback, useEffect, useRef, useState } from 'react';
import { OfferQueryParams } from './useOfferParams';

export function useOfferData({
  page,
  pageSize,
  query,
  status,
  findSide,
  sort,
  ascending,
}: OfferQueryParams) {
  const { addError } = useErrors();
  const [offers, setOffers] = useState<OfferRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [loadedPage, setLoadedPage] = useState(page);
  const [loadedParamsKey, setLoadedParamsKey] = useState<string | null>(null);
  const requestId = useRef(0);

  // Identifies which request the currently loaded data (offers/total) came
  // from, so callers can tell whether it reflects the params they're
  // rendering for right now (see `isCurrent` below) rather than a
  // just-superseded request whose params have already changed (e.g.
  // clearing filters, or the initial empty-wallet load).
  const paramsKey = JSON.stringify({
    page,
    pageSize,
    query,
    status,
    findSide,
    sort,
    ascending,
  });

  const refresh = useCallback(() => {
    const id = ++requestId.current;
    setIsLoading(true);

    return commands
      .getOffers({
        offset: (page - 1) * pageSize,
        limit: pageSize,
        status: status === 'all' ? null : status,
        find_value: query,
        find_side: findSide,
        sort_mode: sort,
        ascending,
      })
      .then((data) => {
        // A newer request has been issued; drop this response.
        if (id !== requestId.current) return;
        setOffers(data.offers);
        setTotal(data.total);
        setLoadedPage(page);
        setLoadedParamsKey(paramsKey);
        setLoaded(true);
      })
      .catch((error) => {
        // A newer request has been issued; drop this error.
        if (id !== requestId.current) return;
        addError(error);
      })
      .finally(() => {
        if (id === requestId.current) setIsLoading(false);
      });
  }, [
    page,
    pageSize,
    status,
    query,
    findSide,
    sort,
    ascending,
    addError,
    paramsKey,
  ]);

  useEffect(() => {
    refresh();

    const unlisten = events.syncEvent.listen((data) => {
      if (data.payload.type === 'coin_state') {
        refresh();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [refresh]);

  const isPastEnd =
    loaded &&
    !isLoading &&
    loadedPage === page &&
    page > 1 &&
    offers.length === 0;

  // True once the loaded offers/total reflect the params currently being
  // rendered for, rather than a previous request (e.g. before-filters or
  // before-this-page data still sitting in state while a new request is
  // in flight).
  const isCurrent = loadedParamsKey === paramsKey;

  return { offers, total, isLoading, loaded, isPastEnd, isCurrent, refresh };
}
