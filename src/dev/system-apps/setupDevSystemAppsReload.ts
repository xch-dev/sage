import { commands, type SageAppRuntimeRecordView } from '@/bindings';

type GetRuntimes = () => SageAppRuntimeRecordView[];

interface DevMessage {
  type?: string;
  ok?: boolean;
}

export function setupDevSystemAppsReload(getRuntimes: GetRuntimes): () => void {
  let disposed = false;
  let ws: WebSocket | null = null;
  let reconnectTimer: number | null = null;

  const connect = () => {
    if (disposed) return;

    ws = new WebSocket('ws://127.0.0.1:1421');

    ws.onmessage = (event) => {
      let payload: DevMessage;

      try {
        payload = JSON.parse(event.data) as DevMessage;
      } catch {
        return;
      }

      if (payload.type !== 'system-apps-built' || payload.ok !== true) {
        return;
      }

      for (const runtime of getRuntimes()) {
        if (runtime.app.kind !== 'system') {
          continue;
        }

        void commands
          .appsDevReloadRuntime({
            appId: runtime.app.common.identity.id,
          })
          .catch(() => {
            //
          });
      }
    };

    ws.onclose = () => {
      if (disposed) return;

      reconnectTimer = window.setTimeout(() => {
        reconnectTimer = null;
        connect();
      }, 1000);
    };
  };

  connect();

  return () => {
    disposed = true;

    if (reconnectTimer !== null) {
      window.clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }

    ws?.close();
    ws = null;
  };
}
