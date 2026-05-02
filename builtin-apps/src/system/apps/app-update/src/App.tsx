import { useEffect, useMemo, useState } from 'react';
import { PermissionsEditor } from '@sage-app/ui/permissions';
import {
  formatSageError,
  getSageSystemClient,
  type AppPermissionsReviewContext,
  type AppUpdateReviewContext,
  type SageAppCapabilityDefinitionView,
  type SageGrantedPermissionsInput,
  type SageGrantedPermissionsView,
  type SageNetworkWhitelistEntry,
  type SageRequestedPermissions,
  type UserBridgeCapability,
  type UserSageAppView,
} from '@sage-system-app/sdk';
import { SystemModalShell } from '@sage-app/ui/system-modal';

type Mode = 'review-update' | 'review-permissions';

type LoadState =
  | { kind: 'loading' }
  | { kind: 'error'; error: string }
  | {
      kind: 'ready';
      mode: Mode;
      app: UserSageAppView;
      updateContext: AppUpdateReviewContext | null;
      permissionsContext: AppPermissionsReviewContext | null;
      definitions: SageAppCapabilityDefinitionView[];
    };

function networkKey(entry: SageNetworkWhitelistEntry): string {
  return `${entry.scheme}://${entry.host}`;
}

function sortCapabilities(
  values: Iterable<UserBridgeCapability>,
): UserBridgeCapability[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

function sortNetwork(
  values: Iterable<SageNetworkWhitelistEntry>,
): SageNetworkWhitelistEntry[] {
  return [...values].sort((a, b) => networkKey(a).localeCompare(networkKey(b)));
}

function getRequestedCapabilities(permissions: SageRequestedPermissions) {
  return {
    required: permissions.capabilities.required ?? [],
    optional: permissions.capabilities.optional ?? [],
  };
}

function getRequestedNetwork(permissions: SageRequestedPermissions) {
  return {
    required: permissions.network.whitelist.required ?? [],
    optional: permissions.network.whitelist.optional ?? [],
  };
}

function definitionMap(definitions: SageAppCapabilityDefinitionView[]) {
  return new Map(
    definitions.map((definition) => [
      definition.key as UserBridgeCapability,
      definition,
    ]),
  );
}

function isUserGrantable(
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
  capability: UserBridgeCapability,
): boolean {
  return definitionsByKey.get(capability)?.flags.userGrantable === true;
}

function buildPermissionReviewApp(
  app: UserSageAppView,
  grantedPermissions: SageGrantedPermissionsView,
): UserSageAppView {
  return {
    ...app,
    common: {
      ...app.common,
      grantedPermissions,
    },
  };
}

function buildUpdateReviewApp(
  app: UserSageAppView,
  context: AppUpdateReviewContext,
  grantedPermissions: SageGrantedPermissionsView,
): UserSageAppView | null {
  if (!context.preview || context.preview.manifest.kind !== 'full') {
    return null;
  }

  return {
    ...app,
    common: {
      ...app.common,
      grantedPermissions,
      activeSnapshot: {
        ...app.common.activeSnapshot,
        manifest: context.preview.manifest.manifest,
      },
    },
  };
}

function nextPermissionsForUpdate(args: {
  app: UserSageAppView;
  context: AppUpdateReviewContext;
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>;
}): SageGrantedPermissionsView | null {
  if (!args.context.preview || args.context.preview.manifest.kind !== 'full') {
    return null;
  }

  const nextRequested = args.context.preview.manifest.manifest.permissions;
  const nextCaps = getRequestedCapabilities(nextRequested);
  const nextNetwork = getRequestedNetwork(nextRequested);

  const nextAllowedCaps = new Set([...nextCaps.required, ...nextCaps.optional]);
  const nextAllowedNetwork = new Set([
    ...nextNetwork.required.map(networkKey),
    ...nextNetwork.optional.map(networkKey),
  ]);

  const retainedCapabilities = (
    args.app.common.grantedPermissions.capabilities ?? []
  )
    .filter((capability) => nextAllowedCaps.has(capability))
    .filter((capability) => isUserGrantable(args.definitionsByKey, capability));

  const requiredGrantable = nextCaps.required.filter((capability) =>
    isUserGrantable(args.definitionsByKey, capability),
  );

  const retainedNetwork = (
    args.app.common.grantedPermissions.network.whitelist ?? []
  ).filter((entry) => nextAllowedNetwork.has(networkKey(entry)));

  const networkMap = new Map<string, SageNetworkWhitelistEntry>();

  for (const entry of retainedNetwork) {
    networkMap.set(networkKey(entry), entry);
  }

  for (const entry of nextNetwork.required) {
    networkMap.set(networkKey(entry), entry);
  }

  return {
    capabilities: sortCapabilities([
      ...retainedCapabilities,
      ...requiredGrantable,
    ]),
    network: {
      whitelist: sortNetwork(networkMap.values()),
    },
  };
}

function AppBody({ state }: { state: Extract<LoadState, { kind: 'ready' }> }) {
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [grantedPermissions, setGrantedPermissions] =
    useState<SageGrantedPermissionsInput>(state.app.common.grantedPermissions);

  const definitionsByKey = useMemo(
    () => definitionMap(state.definitions),
    [state.definitions],
  );

  useEffect(() => {
    if (state.mode !== 'review-update' || !state.updateContext) {
      setGrantedPermissions(state.app.common.grantedPermissions);
      return;
    }

    const next = nextPermissionsForUpdate({
      app: state.app,
      context: state.updateContext,
      definitionsByKey,
    });

    if (next) {
      setGrantedPermissions(next);
    }
  }, [definitionsByKey, state]);

  const reviewApp = useMemo(() => {
    if (state.mode === 'review-permissions') {
      return buildPermissionReviewApp(state.app, grantedPermissions);
    }

    if (!state.updateContext) {
      return null;
    }

    return buildUpdateReviewApp(
      state.app,
      state.updateContext,
      grantedPermissions,
    );
  }, [grantedPermissions, state]);

  async function closeModal() {
    const client = await getSageSystemClient();
    await client.runtimeManager.killRuntime({ appId: 'app-update' });
  }

  async function submit() {
    setSubmitting(true);
    setSubmitError(null);

    try {
      const client = await getSageSystemClient();

      if (state.mode === 'review-permissions') {
        await client.appPermissions.applyPermissions({
          appId: state.app.common.identity.id,
          grantedPermissions,
        });
      } else {
        await client.appUpdate.applyUpdate({
          appId: state.app.common.identity.id,
          grantedPermissions,
        });
      }

      await closeModal();
    } catch (err) {
      setSubmitError(formatSageError(err));
    } finally {
      setSubmitting(false);
    }
  }

  if (state.mode === 'review-update') {
    const preview = state.updateContext?.preview ?? null;

    if (!preview) {
      return (
        <div className='space-y-4'>
          <h1 className='text-lg font-semibold'>App is up to date</h1>
          <p className='text-sm text-muted-foreground'>
            No installable update is available for{' '}
            {state.app.common.activeSnapshot.manifest.name}.
          </p>
          <div className='flex justify-end'>
            <button
              className='rounded-md border border-border px-4 py-2 text-sm'
              onClick={closeModal}
            >
              Close
            </button>
          </div>
        </div>
      );
    }

    if (preview.manifest.kind === 'partial') {
      const header = preview.manifest.manifest_header;

      return (
        <div className='space-y-4'>
          <h1 className='text-lg font-semibold'>Update cannot be installed</h1>
          <div className='text-sm text-muted-foreground'>
            {header.name} requires manifest features this Sage version cannot
            safely understand.
          </div>
          <pre className='max-h-48 overflow-auto rounded-xl bg-muted p-3 text-xs whitespace-pre-wrap'>
            {preview.manifest.parse_error}
          </pre>
          <div className='flex justify-end'>
            <button
              className='rounded-md border border-border px-4 py-2 text-sm'
              onClick={closeModal}
            >
              Close
            </button>
          </div>
        </div>
      );
    }
  }

  if (!reviewApp) {
    return (
      <div className='space-y-4'>
        <h1 className='text-lg font-semibold'>Nothing to review</h1>
        <button
          className='rounded-md border border-border px-4 py-2 text-sm'
          onClick={closeModal}
        >
          Close
        </button>
      </div>
    );
  }

  const title =
    state.mode === 'review-update'
      ? 'Review app update'
      : 'Change app permissions';

  const subtitle =
    state.mode === 'review-update' &&
    state.updateContext?.preview?.manifest.kind === 'full'
      ? `v${state.app.common.activeSnapshot.manifest.version} → v${state.updateContext.preview.manifest.manifest.version}`
      : state.app.common.activeSnapshot.manifest.version;

  return (
    <div className='space-y-5'>
      <div>
        <h1 className='text-lg font-semibold'>{title}</h1>
        <div className='mt-1 text-sm text-muted-foreground'>
          {state.app.common.activeSnapshot.manifest.name} · {subtitle}
        </div>
      </div>

      <PermissionsEditor
        app={reviewApp}
        grantedPermissions={grantedPermissions}
        capabilityDefinitions={state.definitions}
        editable={!submitting}
        onGrantedPermissionsChange={setGrantedPermissions}
      />

      {submitError ? (
        <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
          {submitError}
        </div>
      ) : null}

      <div className='flex justify-end gap-2'>
        <button
          className='rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60'
          disabled={submitting}
          onClick={closeModal}
        >
          Cancel
        </button>
        <button
          className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
          disabled={submitting}
          onClick={submit}
        >
          {submitting
            ? state.mode === 'review-update'
              ? 'Updating…'
              : 'Saving…'
            : state.mode === 'review-update'
              ? 'Confirm update'
              : 'Save permissions'}
        </button>
      </div>
    </div>
  );
}

export function App() {
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

  return (
    <SystemModalShell>
      {state.kind === 'loading' ? (
        <div className='text-sm text-muted-foreground'>Loading review…</div>
      ) : state.kind === 'error' ? (
        <div className='space-y-3'>
          <h1 className='text-lg font-semibold'>App update failed</h1>
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {state.error}
          </div>
        </div>
      ) : (
        <AppBody state={state} />
      )}
    </SystemModalShell>
  );
}
