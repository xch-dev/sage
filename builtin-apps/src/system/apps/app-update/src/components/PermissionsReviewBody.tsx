import { useState } from 'react';
import {
  appIconFromCommonView,
  AppModalShell,
  AppPermissionEditor,
  WalletScopeEditor,
} from '@sage-app/ui';
import {
  formatSageError,
  getSageSystemClient,
  type SageAppWalletScope,
  type SageGrantedPermissionsInput,
} from '@sage-system-app/sdk';
import type { LoadState } from '../types';

type PermissionsReadyState = Extract<LoadState, { kind: 'ready' }>;
type ReviewTab = 'permissions' | 'wallets';

export function PermissionsReviewBody({
  state,
}: {
  state: PermissionsReadyState;
}) {
  const [tab, setTab] = useState<ReviewTab>('permissions');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [grantedPermissions, setGrantedPermissions] =
    useState<SageGrantedPermissionsInput>(state.app.common.grantedPermissions);

  const [walletScope, setWalletScope] = useState<SageAppWalletScope>(
    state.app.common.walletScope,
  );

  async function close() {
    const client = await getSageSystemClient();
    await client.runtimeManager.closeSelf();
  }

  async function submit() {
    setSubmitting(true);
    setError(null);

    try {
      const client = await getSageSystemClient();

      await client.appPermissions.applyPermissions({
        appId: state.app.common.identity.id,
        grantedPermissions,
        walletScope,
      });

      await close();
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <AppModalShell
      appName={state.app.common.activeSnapshot.manifest.name}
      appIcon={appIconFromCommonView(state.app.common)}
      title='Change app access'
      footer={
        <div className='flex justify-end gap-2'>
          <button
            className='rounded-md border px-4 py-2 text-sm'
            disabled={submitting}
            onClick={close}
          >
            Cancel
          </button>

          <button
            className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
            disabled={submitting}
            onClick={submit}
          >
            {submitting ? 'Saving…' : 'Save changes'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <div className='inline-flex rounded-lg border border-border bg-background p-1'>
          <button
            type='button'
            disabled={submitting}
            onClick={() => setTab('permissions')}
            className={[
              'rounded-md px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-60',
              tab === 'permissions'
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
            ].join(' ')}
          >
            Permissions
          </button>

          <button
            type='button'
            disabled={submitting}
            onClick={() => setTab('wallets')}
            className={[
              'rounded-md px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-60',
              tab === 'wallets'
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
            ].join(' ')}
          >
            Wallets
          </button>
        </div>

        {tab === 'permissions' ? (
          <AppPermissionEditor
            app={state.app}
            grantedPermissions={state.app.common.grantedPermissions}
            capabilityDefinitions={state.definitions}
            editable={!submitting}
            onGrantedPermissionsChange={setGrantedPermissions}
          />
        ) : (
          <WalletScopeEditor
            wallets={state.wallets}
            walletScope={walletScope}
            disabled={submitting}
            onWalletScopeChange={setWalletScope}
          />
        )}

        {error ? (
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}
