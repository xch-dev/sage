import { useState, useMemo } from 'react';
import {
  appIconFromCommonView,
  AppModalShell,
  inputToGrantedPermissionsView,
  PermissionsEditor,
} from '@sage-app/ui';
import { formatSageError, getSageSystemClient } from '@sage-system-app/sdk';
import { useUpdatePermissions } from '../hooks/useUpdatePermissions';
import { NoUpdateBody } from './NoUpdateBody';
import { PartialUpdateBody } from './PartialUpdateBody';

export function UpdateReviewBody({ state }: any) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { grantedPermissions, setGrantedPermissions } = useUpdatePermissions({
    app: state.app,
    context: state.updateContext,
    definitions: state.definitions,
  });

  async function close() {
    const client = await getSageSystemClient();
    await client.runtimeManager.closeSelf();
  }

  async function submit() {
    setSubmitting(true);
    setError(null);

    try {
      const client = await getSageSystemClient();

      await client.appUpdate.applyUpdate({
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

  const preview = state.updateContext?.preview ?? null;

  // ---- No update
  if (!preview) {
    return (
      <NoUpdateBody
        name={state.app.common.activeSnapshot.manifest.name}
        onClose={close}
      />
    );
  }

  // ---- Partial / unsupported
  if (preview.manifest.kind === 'partial') {
    return (
      <PartialUpdateBody
        header={preview.manifest.manifest_header}
        error={preview.manifest.parse_error}
        onClose={close}
      />
    );
  }

  // ---- Build preview app
  const reviewApp = useMemo(() => {
    return {
      ...state.app,
      common: {
        ...state.app.common,
        grantedPermissions: inputToGrantedPermissionsView(grantedPermissions),
        activeSnapshot: {
          ...state.app.common.activeSnapshot,
          manifest: preview.manifest.manifest,
        },
      },
    };
  }, [state.app, grantedPermissions, preview]);

  return (
    <AppModalShell
      title='Review app update'
      appIcon={appIconFromCommonView(state.app.common)}
      appName={state.app.common.activeSnapshot.manifest.name}
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
            {submitting ? 'Updating…' : 'Confirm update'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <PermissionsEditor
          app={reviewApp}
          grantedPermissions={reviewApp.common.grantedPermissions}
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
