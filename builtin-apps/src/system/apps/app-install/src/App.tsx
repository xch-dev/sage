import { useEffect, useState } from 'react';
import { formatSageError, getSageSystemClient } from '@sage-system-app/sdk';
import { previewUrl } from './api';
import { ErrorState } from './components/ErrorState';
import { LoadingState } from './components/LoadingState';
import { ReviewInstallView } from './components/ReviewInstallView';
import { SelectSourceView } from './components/SelectSourceView';
import type { LoadState } from './types';

export function App() {
  const [state, setState] = useState<LoadState>({ kind: 'loading' });

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const client = await getSageSystemClient();

        const definitions = await client.capabilities.listUserDefinitions();

        const params = new URLSearchParams(window.location.search);
        const mode = params.get('mode') ?? 'select-source';
        const appUrl = params.get('appUrl');

        if (mode === 'url' && appUrl) {
          const source = await previewUrl(appUrl);

          if (!cancelled) {
            setState({
              kind: 'review',
              definitions,
              source,
            });
          }

          return;
        }

        if (!cancelled) {
          setState({ kind: 'selecting', definitions });
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

  if (state.kind === 'loading') return <LoadingState />;
  if (state.kind === 'error') return <ErrorState error={state.error} />;

  if (state.kind === 'selecting') {
    return (
      <SelectSourceView
        onReview={(source) =>
          setState({
            kind: 'review',
            definitions: state.definitions,
            source,
          })
        }
      />
    );
  }

  return (
    <ReviewInstallView source={state.source} definitions={state.definitions} />
  );
}
