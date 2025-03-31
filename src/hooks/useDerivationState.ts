import { useState, useEffect, useCallback } from 'react';
import { DerivationRecord, commands, events } from '../bindings';
import { useErrors } from './useErrors';

export function useDerivationState(hardened: boolean = false) {
  const { addError } = useErrors();
  const [derivations, setDerivations] = useState<DerivationRecord[]>([]);
  const [currentPage, setCurrentPage] = useState(0);
  const [totalDerivations, setTotalDerivations] = useState(0);
  const pageSize = 100;

  const fetchDerivations = useCallback(() => {
    const offset = currentPage * pageSize;

    commands
      .getDerivations({
        hardened,
        offset,
        limit: pageSize,
      })
      .then((result) => {
        setDerivations(result.derivations);
        setTotalDerivations(result.total);
      })
      .catch(addError);
  }, [addError, currentPage, hardened, pageSize]);

  useEffect(() => {
    fetchDerivations();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'derivation') {
        fetchDerivations();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [fetchDerivations]);

  // Reset to page 0 when hardened changes
  useEffect(() => {
    setCurrentPage(0);
  }, [hardened]);

  return {
    derivations,
    currentPage,
    totalDerivations,
    pageSize,
    setCurrentPage,
    fetchDerivations,
  };
}
