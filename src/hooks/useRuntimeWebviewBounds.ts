import React, { useCallback, useEffect } from 'react';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';
import { platform } from '@tauri-apps/plugin-os';
import { setWebviewBounds } from 'tauri-plugin-sage';

function formatError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;

  try {
    return JSON.stringify(err, null, 2);
  } catch {
    return String(err);
  }
}

interface Args {
  webviewLabel: string | null | undefined;
  containerRef: React.RefObject<HTMLElement | null>;
  enabled?: boolean;
}

export function useRuntimeWebviewBounds({
  webviewLabel,
  containerRef,
  enabled = true,
}: Args) {
  const syncBounds = useCallback(async () => {
    if (!enabled || !webviewLabel) {
      return;
    }

    const container = containerRef.current;
    if (!container) {
      return;
    }

    const rect = container.getBoundingClientRect();

    if (platform() === 'ios') {
      await setWebviewBounds({
        label: webviewLabel,
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.max(1, Math.round(rect.width)),
        height: Math.max(1, Math.round(rect.height)),
      });
      return;
    }

    const webview = await Webview.getByLabel(webviewLabel).catch(() => null);
    if (!webview) {
      return;
    }

    await webview.setPosition(
      new LogicalPosition(Math.round(rect.left), Math.round(rect.top)),
    );

    await webview.setSize(
      new LogicalSize(
        Math.max(1, Math.round(rect.width)),
        Math.max(1, Math.round(rect.height)),
      ),
    );
  }, [containerRef, enabled, webviewLabel]);

  const scheduleSyncBounds = useCallback(() => {
    requestAnimationFrame(() => {
      void syncBounds().catch((err) => {
        const message = formatError(err);

        if (message.includes('webview not found')) {
          return;
        }

        console.error('Failed to sync runtime webview bounds:', err);
      });
    });
  }, [syncBounds]);

  useEffect(() => {
    if (!enabled || !webviewLabel || !containerRef.current) {
      return;
    }

    let disposed = false;
    let delayedSyncTimers: number[] = [];
    let resizeObserver: ResizeObserver | null = null;

    const run = () => {
      if (!disposed) {
        scheduleSyncBounds();
      }
    };

    run();

    delayedSyncTimers = [0, 50, 150, 300].map((delay) =>
      window.setTimeout(run, delay),
    );

    resizeObserver = new ResizeObserver(run);
    resizeObserver.observe(containerRef.current);

    window.addEventListener('resize', run);

    return () => {
      disposed = true;
      delayedSyncTimers.forEach((id) => window.clearTimeout(id));
      resizeObserver?.disconnect();
      window.removeEventListener('resize', run);
    };
  }, [containerRef, enabled, scheduleSyncBounds, webviewLabel]);

  return {
    syncBounds,
    scheduleSyncBounds,
  };
}
