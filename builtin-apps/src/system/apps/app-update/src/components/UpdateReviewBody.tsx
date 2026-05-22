import { useMemo, useState } from 'react';
import {
  appIconFromCommonView,
  AppModalShell,
  UpdateDecisionPermissionEditor,
} from '@sage-app/ui';
import {
  formatSageError,
  getSageSystemClient,
  type SageGrantedPermissionsInput,
} from '@sage-system-app/sdk';
import { NoUpdateBody } from './NoUpdateBody';
import { PartialUpdateBody } from './PartialUpdateBody';

export function UpdateReviewBody({ state, onReload }: any) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [permissionsViewed, setPermissionsViewed] = useState(false);

  const preview = state.updateContext?.preview ?? null;

  const additionalGrantedPermissions =
    useMemo<SageGrantedPermissionsInput>(() => {
      const decision = state.app.pendingUpdate?.decision;

      if (decision?.kind !== 'review') {
        return {
          capabilities: [],
          network: {
            whitelist: [],
            whitelistByNetwork: {},
          },
        };
      }

      return {
        capabilities: decision.requiredUserGrantableCapabilities ?? [],
        network: {
          whitelist: decision.requiredNetworkWhitelist ?? [],
          whitelistByNetwork: decision.requiredNetworkWhitelistByNetwork ?? {},
        },
      };
    }, [state.app.pendingUpdate?.decision]);

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
        additionalGrantedPermissions,
      });

      await close();
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setSubmitting(false);
    }
  }

  if (!preview) {
    return (
      <AppModalShell
        title='Review app update'
        appIcon={appIconFromCommonView(state.app.common)}
        appName={state.app.common.activeSnapshot.manifest.name}
        footer={<UpdateIssueFooter onReload={onReload} onClose={close} />}
      >
        <NoUpdateBody
          name={state.app.common.activeSnapshot.manifest.name}
          onClose={close}
        />
      </AppModalShell>
    );
  }

  if (preview.manifest.kind === 'partial') {
    return (
      <AppModalShell
        title='Update cannot be installed'
        appIcon={appIconFromCommonView(state.app.common)}
        appName={state.app.common.activeSnapshot.manifest.name}
        footer={<UpdateIssueFooter onReload={onReload} onClose={close} />}
      >
        <PartialUpdateBody
          header={preview.manifest.manifest_header}
          error={preview.manifest.parse_error}
          onClose={close}
        />
      </AppModalShell>
    );
  }

  return (
    <AppModalShell
      title='Review app update'
      appIcon={appIconFromCommonView(state.app.common)}
      appName={state.app.common.activeSnapshot.manifest.name}
      requireScrollEnd
      onScrollEndChange={setPermissionsViewed}
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
            disabled={submitting || !permissionsViewed}
            onClick={submit}
          >
            {submitting ? 'Updating…' : 'Confirm update'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <UpdateDecisionPermissionEditor
          app={state.app}
          capabilityDefinitions={state.definitions}
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

function UpdateIssueFooter({
  onReload,
  onClose,
}: {
  onReload: () => void;
  onClose: () => void;
}) {
  return (
    <div className='flex items-center justify-between gap-2'>
      <button
        className='rounded-md border border-border px-4 py-2 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground'
        onClick={onReload}
      >
        Re-check update
      </button>

      <button
        className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground transition-opacity hover:opacity-90'
        onClick={onClose}
      >
        Close
      </button>
    </div>
  );
}
