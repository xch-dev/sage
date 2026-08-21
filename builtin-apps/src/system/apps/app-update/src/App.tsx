import { SystemModalShell } from 'sage-app-ui';
import { useLoadState } from './hooks/useLoadState';
import { UpdateReviewBody } from './components/UpdateReviewBody';
import { PermissionsReviewBody } from './components/PermissionsReviewBody';
import { useSageSystemClient } from 'sage-system-app-sdk';

export function App() {
  const sage = useSageSystemClient();
  const { state, reload } = useLoadState();

  if (state.kind === 'loading') {
    return (
      <SystemModalShell>
        <div className='text-sm text-muted-foreground'>Loading review…</div>
      </SystemModalShell>
    );
  }

  if (state.kind === 'error') {
    return (
      <SystemModalShell>
        <div className='text-destructive'>{state.error}</div>
      </SystemModalShell>
    );
  }

  if (state.mode === 'review-permissions') {
    return <PermissionsReviewBody state={state} />;
  }

  const pendingUpdate =
    state.updateContext.target.kind === 'installed'
      ? state.updateContext.target.app.pendingUpdate
      : state.updateContext.target.pendingUpdate;

  return (
    <UpdateReviewBody
      key={pendingUpdate?.manifestHash ?? 'no-pending-update'}
      state={state}
      onReload={reload}
    />
  );
}
