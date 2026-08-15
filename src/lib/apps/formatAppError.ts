export function formatAppError(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }

  if (typeof err === 'string') {
    return err;
  }

  if (err && typeof err === 'object') {
    const maybe = err as Record<string, unknown>;

    if (typeof maybe.message === 'string') {
      return maybe.message;
    }

    if (typeof maybe.error === 'string') {
      return maybe.error;
    }

    if (maybe.error && typeof maybe.error === 'object') {
      const nested = maybe.error as Record<string, unknown>;
      if (typeof nested.message === 'string') {
        return nested.message;
      }
    }

    try {
      return JSON.stringify(err, null, 2);
    } catch {
      return 'Unknown error';
    }
  }

  return String(err);
}
