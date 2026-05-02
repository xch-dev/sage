import { useEffect, useMemo, useState } from 'react';
import type {
  SageAppPackageManifest,
  SageAppUrlPreview,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  UserSageAppView,
} from '@/bindings';
import {
  networkKey,
  sortCapabilities,
  sortNetwork,
} from '@/lib/apps/permissionCollections';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { PermissionsEditor } from '@/components/apps/permissions/PermissionsEditor';
import {
  getAppUpdatePermissionsDelta,
  type AppUpdatePermissionsDelta,
} from '@/lib/apps/updatePermissionsDelta';

interface Props {
  open: boolean;
  app: UserSageAppView | null;
  preview: SageAppUrlPreview | null;
  submitting: boolean;
  error: string | null;
  onCancel: () => void;
  onConfirm: (nextGranted: SageGrantedPermissionsInput) => void;
}

function fullPreviewManifest(
  preview: SageAppUrlPreview,
): SageAppPackageManifest {
  if (preview.manifest.kind !== 'full') {
    throw new Error('Expected full app update manifest');
  }

  return preview.manifest.manifest;
}

function buildReviewManifest(
  preview: SageAppUrlPreview,
  delta: AppUpdatePermissionsDelta,
): SageAppPackageManifest {
  const manifest = fullPreviewManifest(preview);

  return {
    ...manifest,
    permissions: {
      capabilities: {
        required: delta.requiredCapabilitiesToGrant,
        optional: delta.addedRequestedCapabilities.optional,
      },
      network: {
        whitelist: {
          required: delta.requiredNetworkToGrant,
          optional: delta.addedRequestedNetwork.optional,
        },
      },
    },
  };
}

function buildReviewApp(
  app: UserSageAppView,
  preview: SageAppUrlPreview,
  delta: AppUpdatePermissionsDelta,
  grantedPermissions: SageGrantedPermissionsView,
): UserSageAppView {
  const reviewManifest = buildReviewManifest(preview, delta);

  return {
    ...app,
    common: {
      ...app.common,
      grantedPermissions,
      activeSnapshot: {
        ...app.common.activeSnapshot,
        manifest: reviewManifest,
      },
    },
  };
}

function buildRemovedPermissionsApp(
  app: UserSageAppView,
  preview: SageAppUrlPreview,
  delta: AppUpdatePermissionsDelta,
): UserSageAppView | null {
  const hasRemoved =
    delta.removedGrantedCapabilities.length > 0 ||
    delta.removedGrantedNetwork.length > 0;

  if (!hasRemoved) {
    return null;
  }

  const manifest: SageAppPackageManifest = {
    ...fullPreviewManifest(preview),
    permissions: {
      capabilities: {
        required: delta.removedGrantedCapabilities,
        optional: [],
      },
      network: {
        whitelist: {
          required: delta.removedGrantedNetwork,
          optional: [],
        },
      },
    },
  };

  return {
    ...app,
    common: {
      ...app.common,
      grantedPermissions: {
        capabilities: delta.removedGrantedCapabilities,
        network: {
          whitelist: delta.removedGrantedNetwork,
        },
      },
      activeSnapshot: {
        ...app.common.activeSnapshot,
        manifest,
      },
    },
  };
}

export function AppUpdateDialog({
  open,
  app,
  preview,
  submitting,
  error,
  onCancel,
  onConfirm,
}: Props) {
  const [showRemoved, setShowRemoved] = useState(false);
  const [
    selectedOptionalGrantedPermissions,
    setSelectedOptionalGrantedPermissions,
  ] = useState<SageGrantedPermissionsInput>({
    capabilities: [],
    network: {
      whitelist: [],
    },
  });

  useEffect(() => {
    if (!open) {
      setShowRemoved(false);
      setSelectedOptionalGrantedPermissions({
        capabilities: [],
        network: {
          whitelist: [],
        },
      });
    }
  }, [open]);

  if (open && app && preview?.manifest.kind === 'partial') {
    const partial = preview.manifest;
    const header = partial.manifest_header;

    return (
      <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onCancel()}>
        <DialogContent className='max-w-2xl'>
          <DialogHeader>
            <DialogTitle>Update cannot be installed</DialogTitle>
          </DialogHeader>

          <div className='space-y-4 text-sm'>
            <div className='space-y-1 text-muted-foreground'>
              <div>{header.name}</div>
              <div>Manifest version {header.manifestVersion ?? 0}</div>
              <div>
                Requires Sage {header.sageVersion.min}
                {header.sageVersion.testedMax
                  ? ` · tested up to ${header.sageVersion.testedMax}`
                  : null}
              </div>
            </div>

            <div className='rounded-md border border-destructive/30 bg-destructive/10 p-3 text-destructive'>
              This app update uses manifest features this Sage version cannot
              understand, so it cannot be installed safely.
            </div>

            <div className='text-muted-foreground'>
              Try updating Sage if a newer version is available. If this is
              already the latest Sage version, the app developer needs to
              publish a compatible manifest.
            </div>

            <pre className='max-h-48 overflow-auto rounded-md bg-muted p-3 text-xs whitespace-pre-wrap'>
              {partial.parse_error}
            </pre>

            {error ? (
              <div className='text-sm text-destructive'>{error}</div>
            ) : null}
          </div>

          <DialogFooter>
            <Button variant='outline' onClick={onCancel} disabled={submitting}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  const delta = useMemo(() => {
    if (!app || !preview) {
      return null;
    }

    return getAppUpdatePermissionsDelta(app, preview);
  }, [app, preview]);

  const nextManifest = useMemo(() => {
    if (!preview || preview.manifest.kind !== 'full') {
      return null;
    }

    return preview.manifest.manifest;
  }, [preview]);

  const reviewGrantedPermissions = useMemo(() => {
    if (!delta) {
      return null;
    }

    return {
      capabilities: sortCapabilities([
        ...delta.requiredCapabilitiesToGrant,
        ...selectedOptionalGrantedPermissions.capabilities,
      ]),
      network: {
        whitelist: sortNetwork([
          ...delta.requiredNetworkToGrant,
          ...selectedOptionalGrantedPermissions.network.whitelist,
        ]),
      },
    } satisfies SageGrantedPermissionsView;
  }, [delta, selectedOptionalGrantedPermissions]);

  const finalGranted = useMemo(() => {
    if (!delta) {
      return null;
    }

    const nextCapabilities = sortCapabilities([
      ...delta.nextGrantedPermissions.capabilities,
      ...selectedOptionalGrantedPermissions.capabilities,
    ]);

    const nextNetworkMap = new Map<string, SageNetworkWhitelistEntry>();

    for (const entry of delta.nextGrantedPermissions.network.whitelist) {
      nextNetworkMap.set(networkKey(entry), entry);
    }

    for (const entry of selectedOptionalGrantedPermissions.network.whitelist) {
      nextNetworkMap.set(networkKey(entry), entry);
    }

    return {
      capabilities: nextCapabilities,
      network: {
        whitelist: sortNetwork(nextNetworkMap.values()),
      },
    } satisfies SageGrantedPermissionsInput;
  }, [delta, selectedOptionalGrantedPermissions]);

  const reviewApp = useMemo(() => {
    if (!app || !preview || !delta || !reviewGrantedPermissions) {
      return null;
    }

    return buildReviewApp(app, preview, delta, reviewGrantedPermissions);
  }, [app, preview, delta, reviewGrantedPermissions]);

  const removedPermissionsApp = useMemo(() => {
    if (!app || !preview || !delta) {
      return null;
    }

    return buildRemovedPermissionsApp(app, preview, delta);
  }, [app, preview, delta]);

  if (
    !app ||
    !preview ||
    !nextManifest ||
    !delta ||
    !reviewGrantedPermissions ||
    !finalGranted ||
    !reviewApp
  ) {
    return (
      <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onCancel()}>
        <DialogContent />
      </Dialog>
    );
  }

  const addedCapabilityCount =
    delta.requiredCapabilitiesToGrant.length +
    delta.addedRequestedCapabilities.optional.length;

  const addedNetworkCount =
    delta.requiredNetworkToGrant.length +
    delta.addedRequestedNetwork.optional.length;

  const removedCount =
    delta.removedGrantedCapabilities.length +
    delta.removedGrantedNetwork.length;

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onCancel()}>
      <DialogContent className='max-w-2xl'>
        <DialogHeader>
          <DialogTitle>Review app update</DialogTitle>
        </DialogHeader>

        <div className='space-y-5'>
          <div className='space-y-1 text-sm text-muted-foreground'>
            <div>{app.common.activeSnapshot.manifest.name}</div>
            <div>
              v{app.common.activeSnapshot.manifest.version} → v
              {nextManifest.version}
            </div>
          </div>

          {addedCapabilityCount > 0 || addedNetworkCount > 0 ? (
            <div className='space-y-3'>
              <h3 className='text-sm font-medium'>
                New permissions requiring review
              </h3>

              <PermissionsEditor
                app={reviewApp}
                grantedPermissions={reviewGrantedPermissions}
                onGrantedPermissionsChange={(next) => {
                  const requiredCapabilitySet = new Set(
                    delta.requiredCapabilitiesToGrant,
                  );
                  const requiredNetworkSet = new Set(
                    delta.requiredNetworkToGrant.map(networkKey),
                  );

                  setSelectedOptionalGrantedPermissions({
                    capabilities: sortCapabilities(
                      next.capabilities.filter(
                        (key) => !requiredCapabilitySet.has(key),
                      ),
                    ),
                    network: {
                      whitelist: sortNetwork(
                        next.network.whitelist.filter(
                          (entry) => !requiredNetworkSet.has(networkKey(entry)),
                        ),
                      ),
                    },
                  });
                }}
              />
            </div>
          ) : null}

          {removedCount > 0 && removedPermissionsApp ? (
            <div className='space-y-2'>
              <button
                type='button'
                className='text-left text-sm font-medium underline-offset-4 hover:underline'
                onClick={() => {
                  setShowRemoved((prev) => !prev);
                }}
              >
                Removed permissions ({removedCount})
                {showRemoved ? ' — hide details' : ' — show details'}
              </button>

              {showRemoved ? (
                <div className='rounded-md border p-3'>
                  <PermissionsEditor
                    app={removedPermissionsApp}
                    grantedPermissions={
                      removedPermissionsApp.common.grantedPermissions
                    }
                    editable={false}
                  />
                </div>
              ) : null}
            </div>
          ) : null}

          {error ? (
            <div className='text-sm text-destructive'>{error}</div>
          ) : null}
        </div>

        <DialogFooter>
          <Button variant='outline' onClick={onCancel} disabled={submitting}>
            Cancel
          </Button>

          <Button
            onClick={() => {
              onConfirm(finalGranted);
            }}
            disabled={submitting}
          >
            {submitting ? 'Updating...' : 'Confirm update'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
