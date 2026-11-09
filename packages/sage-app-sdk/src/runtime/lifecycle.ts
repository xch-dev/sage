import type * as Generated from '../generated-types';

export function handleBeforeStopEvent(
  event: Generated.BeforeStopEvent,
  beforeStopHandlers: Set<
    (event: Generated.BeforeStopEvent) => void | Promise<void>
  >,
  callHost: <T>(method: string, params?: unknown) => Promise<T>,
  rejectAllPending: (reason: string) => void,
) {
  rejectAllPending('Sage runtime is stopping');

  if (!event?.requestId || beforeStopHandlers.size === 0) {
    return;
  }

  const handlers = Array.from(beforeStopHandlers);

  void Promise.allSettled(
    handlers.map((handler) => Promise.resolve(handler(event))),
  ).finally(() => {
    void callHost<Generated.RuntimeAckResult>('app.lifecycle.readyToStop', {
      requestId: event.requestId,
    } satisfies Generated.ReadyToStopParams).catch((error: unknown) => {
      console.error('Failed to acknowledge before-stop:', error);
    });
  });
}
