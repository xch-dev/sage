import { CustomError } from '@/contexts/ErrorContext';

// Tauri IPC rejects with a plain string (e.g. a missing-permission error)
// instead of the usual { kind, reason } shape when the command itself is
// unreachable, so callers can't assume `e` is already a CustomError.
export function toCustomError(e: unknown): CustomError {
  if (
    typeof e === 'object' &&
    e !== null &&
    'kind' in e &&
    'reason' in e &&
    typeof (e as CustomError).reason === 'string'
  ) {
    return e as CustomError;
  }

  return {
    kind: 'internal',
    reason: e instanceof Error ? e.message : String(e),
  };
}

/** Which database failed to migrate: the wallet's own, or the one shared by
 *  the wallets on its network, which holds the address book and handle
 *  mappings. Each needs a different database deleted to fix it. */
export type DatabaseMigration = 'wallet' | 'shared';

/** The database that failed to migrate, if that's why the request failed. */
export function databaseMigration(e: unknown): DatabaseMigration | null {
  if (typeof e !== 'object' || e === null || !('kind' in e)) {
    return null;
  }

  switch (e.kind) {
    case 'database_migration':
      return 'wallet';
    case 'shared_database_migration':
      return 'shared';
    default:
      return null;
  }
}
