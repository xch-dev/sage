import { useEffect, useMemo, useState } from 'react';
import {
  AppModalShell,
  AppPermissionEditor,
  WalletScopeEditor,
} from 'sage-app-ui';
import {
  formatSageError,
  type SageAppCapabilityDefinitionView,
  type SageAppWalletScope,
  type SageGrantedPermissionsInput,
  type SystemWalletView,
} from 'sage-system-app-sdk';
import { closeSelf, installSource, listWallets } from '../api';
import type { InstallSource } from '../types';
import { resolveInstallIcon } from '../utils/icons';
import {
  buildPreviewApp,
  emptyGrantedPermissions,
  hasRequiredPermissions,
  initialGrantedPermissions,
  installManifest,
} from '../utils/permissions';
import { UnsupportedManifestView } from './UnsupportedManifestView';

type Step = 'permissions' | 'wallets';

export function ReviewInstallView({
  source,
  definitions,
}: {
  source: InstallSource;
  definitions: SageAppCapabilityDefinitionView[];
}) {
  const manifest = installManifest(source);
  const reviewsPermissions = manifest
    ? hasRequiredPermissions(manifest, definitions)
    : false;
  const compatibility = source.compatibility;

  const [step, setStep] = useState<Step>(() =>
    reviewsPermissions ? 'permissions' : 'wallets',
  );
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wallets, setWallets] = useState<SystemWalletView[]>([]);
  const [walletsLoading, setWalletsLoading] = useState(true);
  const [permissionsViewed, setPermissionsViewed] = useState(false);

  const [walletScope, setWalletScope] = useState<SageAppWalletScope>({
    kind: 'allWallets',
  });

  const [grantedPermissions, setGrantedPermissions] =
    useState<SageGrantedPermissionsInput>(() =>
      manifest
        ? initialGrantedPermissions(manifest, definitions)
        : emptyGrantedPermissions(),
    );

  useEffect(() => {
    let disposed = false;

    async function loadWallets() {
      try {
        setWalletsLoading(true);
        const result = await listWallets();

        if (!disposed) {
          setWallets(result.wallets);
        }
      } catch (err) {
        if (!disposed) {
          setError(formatSageError(err));
        }
      } finally {
        if (!disposed) {
          setWalletsLoading(false);
        }
      }
    }

    void loadWallets();

    return () => {
      disposed = true;
    };
  }, []);

  const previewApp = useMemo(() => {
    if (!manifest) return null;
    return buildPreviewApp(manifest, grantedPermissions);
  }, [manifest, grantedPermissions]);

  const canInstall =
    !installing &&
    !walletsLoading &&
    compatibility.status.kind !== 'requiresNewerSage' &&
    compatibility.status.kind !== 'invalid' &&
    (walletScope.kind === 'allWallets' || walletScope.fingerprints.length > 0);

  async function install() {
    if (!manifest || !canInstall) return;

    setInstalling(true);
    setError(null);

    try {
      await installSource(source, grantedPermissions, walletScope);
      await closeSelf();
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setInstalling(false);
    }
  }

  if (!manifest || !previewApp) {
    return <UnsupportedManifestView source={source} error={error} />;
  }

  if (
    compatibility.status.kind === 'requiresNewerSage' ||
    compatibility.status.kind === 'invalid'
  ) {
    return (
      <AppModalShell
        appName={manifest.name}
        appIcon={resolveInstallIcon(source)}
        title='App cannot be installed'
        footer={
          <div className='flex justify-end'>
            <button
              className='rounded-md border border-border px-4 py-2 text-sm'
              onClick={closeSelf}
            >
              Close
            </button>
          </div>
        }
      >
        <div className='space-y-3 text-sm'>
          <h1 className='text-lg font-semibold'>
            {compatibility.status.kind === 'requiresNewerSage'
              ? 'Requires a newer Sage'
              : 'Invalid Sage version requirement'}
          </h1>
          {compatibility.status.kind === 'requiresNewerSage' ? (
            <p className='text-muted-foreground'>
              This app requires Sage {compatibility.status.minimumVersion} or
              newer. You are running Sage {compatibility.currentVersion}.
            </p>
          ) : (
            <p className='text-destructive'>{compatibility.status.reason}</p>
          )}
        </div>
      </AppModalShell>
    );
  }

  return (
    <AppModalShell
      appName={manifest.name}
      appIcon={resolveInstallIcon(source)}
      title={step === 'permissions' ? 'Review permissions' : 'Select wallets'}
      requireScrollEnd={step === 'permissions'}
      onScrollEndChange={setPermissionsViewed}
      footer={
        <div className='flex justify-between gap-2'>
          <button
            className='rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60'
            disabled={installing}
            onClick={closeSelf}
          >
            Cancel
          </button>

          <div className='flex gap-2'>
            {step === 'wallets' && reviewsPermissions ? (
              <button
                className='rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60'
                disabled={installing}
                onClick={() => setStep('permissions')}
              >
                Back
              </button>
            ) : null}

            {step === 'permissions' ? (
              <button
                className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
                disabled={installing || !permissionsViewed}
                onClick={() => {
                  setError(null);
                  setStep('wallets');
                }}
              >
                Continue
              </button>
            ) : (
              <button
                className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
                disabled={!canInstall}
                onClick={install}
              >
                {installing ? 'Installing…' : 'Install'}
              </button>
            )}
          </div>
        </div>
      }
    >
      <div className='space-y-5'>
        {compatibility.status.kind === 'untestedNewerSage' ? (
          <div className='rounded-xl border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300'>
            This app has only been tested through Sage{' '}
            {compatibility.status.testedMaxVersion}. You are running Sage{' '}
            {compatibility.currentVersion}, so some features may not work as
            expected.
          </div>
        ) : null}

        {step === 'permissions' ? (
          <AppPermissionEditor
            app={previewApp}
            grantedPermissions={previewApp.common.grantedPermissions}
            capabilityDefinitions={definitions}
            editable={!installing}
            onGrantedPermissionsChange={setGrantedPermissions}
          />
        ) : walletsLoading ? (
          <div className='rounded-xl border border-border p-4 text-sm text-muted-foreground'>
            Loading wallets…
          </div>
        ) : (
          <>
            <WalletScopeEditor
              wallets={wallets}
              walletScope={walletScope}
              disabled={installing}
              onWalletScopeChange={setWalletScope}
            />
          </>
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
