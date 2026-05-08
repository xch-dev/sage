import { useCallback, useEffect, useState } from 'react';
import { formatSageError, useSageSystemClient } from '@sage-system-app/sdk';
import type { LoadState, Mode } from '../types';

export function useLoadState() {
  const [state, setState] = useState<LoadState>({ kind: 'loading' });
  const sage = useSageSystemClient();

  const reload = useCallback(() => {
    let cancelled = false;

    async function load() {
      setState({ kind: 'loading' });

      try {
        const params = new URLSearchParams(window.location.search);
        const appId = params.get('appId');
        const mode = (params.get('mode') ?? 'review-update') as Mode;

        if (!appId) {
          setState({ kind: 'error', error: 'Missing appId' });
          return;
        }

        const [definitions, walletsResult] = await Promise.all([
          sage.capabilities.listUserDefinitions(),
          sage.wallet.listWallets(),
        ]);

        if (mode === 'review-permissions') {
          const permissionsContext = await sage.appPermissions.getReviewContext(
            {
              appId,
            },
          );

          if (!cancelled) {
            setState({
              kind: 'ready',
              mode,
              app: permissionsContext.app,
              permissionsContext,
              updateContext: null,
              definitions,
              wallets: walletsResult.wallets,
            });
          }

          return;
        }

        const updateContext = await sage.appUpdate.getReviewContext({ appId });

        if (!cancelled) {
          setState({
            kind: 'ready',
            mode: 'review-update',
            app: updateContext.app,
            updateContext,
            permissionsContext: null,
            definitions,
            wallets: walletsResult.wallets,
          });
        }
      } catch (err) {
        if (!cancelled) {
          setState({ kind: 'error', error: formatSageError(err) });
        }
      }
    }

    void load();

    return () => {
      cancelled = true;
    };
  }, [sage]);

  useEffect(() => reload(), [reload]);

  return { state, reload };
}
