import { useState } from 'react';
import {
  appIconFromCommonView,
  AppModalShell,
  PermissionsEditor,
} from '@sage-app/ui';
import { formatSageError, getSageSystemClient } from '@sage-system-app/sdk';

export function PermissionsReviewBody({ state }: any) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [grantedPermissions, setGrantedPermissions] = useState(
    state.app.common.grantedPermissions,
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
      title='Change app permissions'
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
            className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground'
            disabled={submitting}
            onClick={submit}
          >
            {submitting ? 'Saving…' : 'Save permissions'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <PermissionsEditor
          app={state.app}
          grantedPermissions={grantedPermissions}
          capabilityDefinitions={state.definitions}
          editable={!submitting}
          onGrantedPermissionsChange={setGrantedPermissions}
        />

        {error && (
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {error}
          </div>
        )}
      </div>
    </AppModalShell>
  );
}
