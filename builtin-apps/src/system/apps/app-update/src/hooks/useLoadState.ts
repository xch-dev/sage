import { useEffect, useState } from 'react';
import { formatSageError, getSageSystemClient } from '@sage-system-app/sdk';
import type { LoadState, Mode } from '../types';

export function useLoadState() {
  const [state, setState] = useState<LoadState>({ kind: 'loading' });

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const params = new URLSearchParams(window.location.search);
        const appId = params.get('appId');
        const mode = (params.get('mode') ?? 'review-update') as Mode;

        if (!appId) {
          setState({ kind: 'error', error: 'Missing appId' });
          return;
        }

        const client = await getSageSystemClient();
        await client.environment.theme.mountCssVars();

        const definitions = await client.capabilities.listUserDefinitions();

        if (mode === 'review-permissions') {
          const permissionsContext =
            await client.appPermissions.getReviewContext({ appId });

          if (!cancelled) {
            setState({
              kind: 'ready',
              mode,
              app: permissionsContext.app,
              permissionsContext,
              updateContext: null,
              definitions,
            });
          }
          return;
        }

        const updateContext = await client.appUpdate.getReviewContext({
          appId,
        });

        if (!cancelled) {
          setState({
            kind: 'ready',
            mode: 'review-update',
            app: updateContext.app,
            updateContext,
            permissionsContext: null,
            definitions,
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
  }, []);

  return state;
}
