function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

export function formatSageError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;

  if (isObject(err)) {
    if (typeof err.message === 'string') return err.message;
    if (typeof err.reason === 'string') return err.reason;

    try {
      return JSON.stringify(err, null, 2);
    } catch {
      return 'Unknown Sage error';
    }
  }

  return String(err);
}
