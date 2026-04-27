import type {
  SageAppPackageManifest,
  SageGrantedPermissions,
  SageNetworkWhitelistEntry,
} from '@/bindings';
import { sortCapabilities } from '@/lib/apps/permissionCollections.ts';

function sortNetworkEntries(
  entries: SageNetworkWhitelistEntry[],
): SageNetworkWhitelistEntry[] {
  return [...entries].sort((a, b) => {
    const aKey = `${a.scheme}://${a.host}`;
    const bKey = `${b.scheme}://${b.host}`;
    return aKey.localeCompare(bKey);
  });
}

export function buildEmptyGrantedPermissions(): SageGrantedPermissions {
  return {
    capabilities: [],
    network: {
      whitelist: [],
    },
  };
}

export function buildInitialGrantedPermissions(
  manifest: SageAppPackageManifest,
): SageGrantedPermissions {
  return {
    capabilities: sortCapabilities(
      manifest.permissions?.capabilities?.required ?? [],
    ),
    network: {
      whitelist: sortNetworkEntries(
        manifest.permissions?.network?.whitelist?.required ?? [],
      ),
    },
  };
}
