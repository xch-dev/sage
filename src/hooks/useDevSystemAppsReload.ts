import { useEffect } from 'react';
import { Webview } from '@tauri-apps/api/webview';
import type { SageAppRuntimeRecordView } from '@/bindings';

export function useDevSystemAppsReload(runtimes: SageAppRuntimeRecordView[]) {
  useEffect(() => {
    if (!import.meta.env.DEV) {
      return;
    }

    let disposed = false;
    let ws: WebSocket | null = null;
    let reconnectTimer: number | null = null;

    const connect = () => {
      if (disposed) return;

      ws = new WebSocket('ws://127.0.0.1:1421');

      ws.onmessage = (event) => {
        let payload: unknown;

        try {
          payload = JSON.parse(event.data);
        } catch {
          return;
        }

        if (
          !payload ||
          typeof payload !== 'object' ||
          (payload as { type?: string }).type !== 'system-apps-built' ||
          (payload as { ok?: boolean }).ok !== true
        ) {
          return;
        }

        for (const runtime of runtimes) {
          if (runtime.app.kind !== 'system') {
            continue;
          }

          void Webview.getByLabel(runtime.webviewLabel)
            .then((webview) => {
              if (!webview) return;
              return webview.emit('sage-dev:reload-system-app');
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
      }

      ws?.close();
    };
  }, [runtimes]);
}
