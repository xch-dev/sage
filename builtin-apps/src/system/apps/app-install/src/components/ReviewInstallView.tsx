import { useEffect, useMemo, useState } from 'react';
import {
  AppModalShell,
  PermissionsEditor,
  WalletScopeEditor,
} from '@sage-app/ui';
import {
  formatSageError,
  type SageAppCapabilityDefinitionView,
  type SageAppWalletScope,
  type SageGrantedPermissionsInput,
  type SystemWalletView,
} from '@sage-system-app/sdk';
import { closeSelf, installSource, listWallets } from '../api';
import type { InstallSource } from '../types';
import { resolveInstallIcon } from '../utils/icons';
import {
  buildPreviewApp,
  emptyGrantedPermissions,
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

  const [step, setStep] = useState<Step>('permissions');
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wallets, setWallets] = useState<SystemWalletView[]>([]);
  const [walletsLoading, setWalletsLoading] = useState(true);

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

  return (
    <AppModalShell
      appName={manifest.name}
      appIcon={resolveInstallIcon(source)}
      title={step === 'permissions' ? 'Review permissions' : 'Select wallets'}
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
            {step === 'wallets' ? (
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
                disabled={installing}
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
        {step === 'permissions' ? (
          <PermissionsEditor
            app={previewApp}
            grantedPermissions={grantedPermissions}
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
