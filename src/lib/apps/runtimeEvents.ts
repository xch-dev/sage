import { listen } from '@tauri-apps/api/event';
import type { SageAppRuntimeRecordView } from '@/bindings';

export const SAGE_RUNTIME_EVENT_NAME = 'apps:runtime-event';

export interface RuntimeManagerRuntimesChangedEvent {
  type: 'runtimeManager.runtimesChanged';
  payload: {
    runtimes: SageAppRuntimeRecordView[];
  };
}

export interface ActiveRuntimeChangedEvent {
  type: 'runtimeManager.activeRuntimeChanged';
  payload: {
    hostWindowLabel: string;
    appId: string | null;
    runtimeId: string | null;
  };
}

export type SageRuntimeEvent =
  | RuntimeManagerRuntimesChangedEvent
  | ActiveRuntimeChangedEvent;

export function subscribeRuntimeEvents(
  callback: (event: SageRuntimeEvent) => void,
) {
  return listen<SageRuntimeEvent>(SAGE_RUNTIME_EVENT_NAME, (event) => {
    callback(event.payload);
  });
}

export function subscribeActiveRuntime(
  callback: (event: ActiveRuntimeChangedEvent['payload']) => void,
) {
  return subscribeRuntimeEvents((event) => {
    if (event.type !== 'runtimeManager.activeRuntimeChanged') {
      return;
    }

    callback(event.payload);
  });
}

