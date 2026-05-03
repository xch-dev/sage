import { useMemo, useState } from 'react';
import { AppModalShell, PermissionsEditor } from '@sage-app/ui';
import {
  formatSageError,
  type SageAppCapabilityDefinitionView,
  type SageGrantedPermissionsInput,
} from '@sage-system-app/sdk';
import { closeSelf, installSource } from '../api';
import type { InstallSource } from '../types';
import { resolveInstallIcon } from '../utils/icons';
import {
  buildPreviewApp,
  emptyGrantedPermissions,
  initialGrantedPermissions,
  installManifest,
} from '../utils/permissions';
import { UnsupportedManifestView } from './UnsupportedManifestView';

export function ReviewInstallView({
  source,
  definitions,
}: {
  source: InstallSource;
  definitions: SageAppCapabilityDefinitionView[];
}) {
  const manifest = installManifest(source);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [grantedPermissions, setGrantedPermissions] =
    useState<SageGrantedPermissionsInput>(() =>
      manifest
        ? initialGrantedPermissions(manifest, definitions)
        : emptyGrantedPermissions(),
    );

  const previewApp = useMemo(() => {
    if (!manifest) return null;
    return buildPreviewApp(manifest, grantedPermissions);
  }, [manifest, grantedPermissions]);

  async function install() {
    if (!manifest) return;

    setInstalling(true);
    setError(null);

    try {
      await installSource(source, grantedPermissions);
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
      title='Install app'
      footer={
        <div className='flex justify-end gap-2'>
          <button
            className='rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60'
            disabled={installing}
            onClick={closeSelf}
          >
            Cancel
          </button>

          <button
            className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
            disabled={installing}
            onClick={install}
          >
            {installing ? 'Installing…' : 'Install'}
          </button>
        </div>
      }
    >
      <div className='space-y-5'>
        <PermissionsEditor
          app={previewApp}
          grantedPermissions={grantedPermissions}
          capabilityDefinitions={definitions}
          editable={!installing}
          onGrantedPermissionsChange={setGrantedPermissions}
        />

        {error ? (
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}
