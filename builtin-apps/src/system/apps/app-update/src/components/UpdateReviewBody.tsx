import { useMemo, useState } from 'react';
import {
  appIconFromCommonView,
  AppModalShell,
  UpdateDecisionPermissionEditor,
} from 'sage-app-ui';
import {
  formatSageError,
  getSageSystemClient,
  type SageGrantedPermissionsInput,
} from 'sage-system-app-sdk';
import { NoUpdateBody } from './NoUpdateBody';
import { PartialUpdateBody } from './PartialUpdateBody';

export function UpdateReviewBody({ state, onReload }: any) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [needsReload, setNeedsReload] = useState(false);
  const [permissionsViewed, setPermissionsViewed] = useState(false);

  const preview = state.updateContext?.preview ?? null;
  const compatibility = state.updateContext?.compatibility ?? null;
  const target = state.updateContext.target;
  const installedApp = target.kind === 'installed' ? target.app : null;
  const recoverableApp = target.kind === 'recoverable' ? target.app : null;
  const pendingUpdate =
    target.kind === 'installed'
      ? target.app.pendingUpdate
      : target.pendingUpdate;
  const appId = installedApp?.common.identity.id ?? recoverableApp?.id ?? '';
  const appName =
    installedApp?.common.activeSnapshot.manifest.name ??
    recoverableApp?.manifestHeader?.name ??
    recoverableApp?.id ??
    'App';
  const appIcon = installedApp
    ? appIconFromCommonView(installedApp.common)
    : recoverableApp?.icon
      ? {
          kind: 'bytes' as const,
          icon: {
            bytes: recoverableApp.icon.bytes,
            mime: recoverableApp.icon.mime,
          },
        }
      : null;

  const additionalGrantedPermissions =
    useMemo<SageGrantedPermissionsInput>(() => {
      const decision = pendingUpdate?.decision;

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
    }, [pendingUpdate?.decision]);

  async function close() {
    const client = await getSageSystemClient();
    await client.runtimeManager.closeSelf();
  }

  async function submit() {
    setSubmitting(true);
    setError(null);

    try {
      const reviewedManifestHash = pendingUpdate?.manifestHash;

      if (!reviewedManifestHash) {
        throw new Error(
          'The pending update changed before it could be applied. Review the latest update and try again.',
        );
      }

      const client = await getSageSystemClient();

      await client.appUpdate.applyUpdate({
        appId,
        additionalGrantedPermissions,
        reviewedManifestHash,
      });

      await close();
    } catch (err) {
      setError(formatSageError(err));
      setNeedsReload(true);
    } finally {
      setSubmitting(false);
    }
  }

  if (!preview) {
    return (
      <AppModalShell
        title='Review app update'
        appIcon={appIcon}
        appName={appName}
        footer={<UpdateIssueFooter onReload={onReload} onClose={close} />}
      >
        <NoUpdateBody name={appName} onClose={close} />
      </AppModalShell>
    );
  }

  if (
    compatibility?.status.kind === 'requiresNewerSage' ||
    compatibility?.status.kind === 'invalid'
  ) {
    return (
      <AppModalShell
        title='Update cannot be installed'
        appIcon={appIcon}
        appName={appName}
        footer={<UpdateIssueFooter onReload={onReload} onClose={close} />}
      >
        <div className='space-y-3'>
          <h1 className='text-lg font-semibold'>
            {compatibility.status.kind === 'requiresNewerSage'
              ? 'Requires a newer Sage'
              : 'Invalid Sage version requirement'}
          </h1>
          {compatibility.status.kind === 'requiresNewerSage' ? (
            <p className='text-sm text-muted-foreground'>
              This update requires Sage {compatibility.status.minimumVersion} or
              newer. You are running Sage {compatibility.currentVersion}. The
              currently installed version of the app has not been changed.
            </p>
          ) : (
            <p className='text-sm text-destructive'>
              {compatibility.status.reason}
            </p>
          )}
        </div>
      </AppModalShell>
    );
  }

  if (preview.manifest.kind === 'partial') {
    return (
      <AppModalShell
        title='Update cannot be installed'
        appIcon={appIcon}
        appName={appName}
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

  if (!pendingUpdate) {
    return (
      <AppModalShell
        title='Review app update'
        appIcon={appIcon}
        appName={appName}
        footer={<UpdateIssueFooter onReload={onReload} onClose={close} />}
      >
        <div className='space-y-2'>
          <h1 className='text-lg font-semibold'>Review needs refreshing</h1>
          <p className='text-sm text-muted-foreground'>
            The pending update changed while this review was loading. Re-check
            the update to review the latest version and permissions.
          </p>
        </div>
      </AppModalShell>
    );
  }

  return (
    <AppModalShell
      title='Review app update'
      appIcon={appIcon}
      appName={appName}
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
            disabled={submitting || needsReload || !permissionsViewed}
            onClick={submit}
          >
            {submitting ? 'Updating…' : 'Confirm update'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        {compatibility?.status.kind === 'untestedNewerSage' ? (
          <div className='rounded-xl border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300'>
            This update has only been tested through Sage{' '}
            {compatibility.status.testedMaxVersion}. You are running Sage{' '}
            {compatibility.currentVersion}.
          </div>
        ) : null}

        <UpdateDecisionPermissionEditor
          pendingUpdate={pendingUpdate}
          capabilityDefinitions={state.definitions}
        />

        {error && (
          <div className='space-y-3 rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            <p>{error}</p>
            <button
              className='rounded-md border border-destructive/40 px-3 py-2 font-medium transition-colors hover:bg-destructive/10'
              onClick={() => void onReload()}
            >
              Review latest update
            </button>
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
