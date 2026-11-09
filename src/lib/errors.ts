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
