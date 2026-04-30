import React, { useCallback, useEffect, useState } from 'react';
import { platform } from '@tauri-apps/plugin-os';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import {
  ensureInlineRuntime,
  getRuntimeWebview,
  markRuntimeVisible,
} from '@/lib/apps/runtimeRegistry';
import type { SageAppView } from '@/bindings';
import { getCurrentWindow } from '@tauri-apps/api/window';

async function getMacWindowedTopInsetPx(): Promise<number> {
  const isMac = platform() === 'macos';
  const isMaximized = await getCurrentWindow().isMaximized();

  return isMac && !isMaximized ? 30 : 0;
}

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
  app: SageAppView | null | undefined;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

export function useAppEmbeddedRuntime({ app, containerRef }: Args) {
  const [attachError, setAttachError] = useState<string | null>(null);
  const [attaching, setAttaching] = useState(false);

  const syncBounds = useCallback(
    async (appId: string) => {
      const webview = await getRuntimeWebview(appId);
      const container = containerRef.current;

      if (!webview || !container) {
        return;
      }

      const inset = await getMacWindowedTopInsetPx();
      const rect = container.getBoundingClientRect();
      const width = Math.max(1, Math.round(rect.width));
      const height = Math.max(1, Math.round(rect.height - inset));
      const x = Math.round(rect.left);
      const y = Math.round(rect.top + inset);

      await webview.setPosition(new LogicalPosition(x, y));
      await webview.setSize(new LogicalSize(width, height));
    },
    [containerRef],
  );

  const scheduleSyncBounds = useCallback(
    (appId: string) => {
      requestAnimationFrame(() => {
        void syncBounds(appId).catch((err) => {
          const message = formatError(err);

          if (message.includes('webview not found')) {
            return;
          }

          console.error('Failed to sync embedded app webview bounds:', err);
        });
      });
    },
    [syncBounds],
  );

  useEffect(() => {
    setAttachError(null);
    if (!app || !containerRef.current) {
      setAttaching(false);
      return;
    }

    const installedApp = app;
    let disposed = false;
    let runtimeCreated = false;
    let resizeObserver: ResizeObserver | null = null;
    let removeWindowResize: (() => void) | null = null;
    let delayedSyncTimers: number[] = [];
    let showAttachingTimer: number | null = null;

    const clearShowAttachingTimer = () => {
      if (showAttachingTimer !== null) {
        window.clearTimeout(showAttachingTimer);
        showAttachingTimer = null;
      }
    };

    const clearDelayedSyncTimers = () => {
      delayedSyncTimers.forEach((id) => window.clearTimeout(id));
      delayedSyncTimers = [];
    };

    const mount = async () => {
      showAttachingTimer = window.setTimeout(() => {
        if (!disposed) {
          setAttaching(true);
        }
      }, 200);
      await ensureInlineRuntime(installedApp);
      runtimeCreated = true;

      if (disposed) {
        return;
      }

      await syncBounds(installedApp.common.identity.id);

      if (disposed) {
        return;
      }

      await markRuntimeVisible(installedApp.common.identity.id, true);

      if (disposed) {
        return;
      }

      clearShowAttachingTimer();
      setAttachError(null);
      setAttaching(false);

      scheduleSyncBounds(installedApp.common.identity.id);

      delayedSyncTimers = [0, 50, 150, 300].map((delay) =>
        window.setTimeout(() => {
          if (!disposed) {
            scheduleSyncBounds(installedApp.common.identity.id);
          }
        }, delay),
      );
      resizeObserver = new ResizeObserver(() => {
        if (!disposed) {
          scheduleSyncBounds(installedApp.common.identity.id);
        }
      });
      const container = containerRef.current;

      if (!container) {
        return;
      }
      resizeObserver.observe(container);
      const handleWindowResize = () => {
        if (!disposed) {
          scheduleSyncBounds(installedApp.common.identity.id);
        }
      };
      window.addEventListener('resize', handleWindowResize);

      removeWindowResize = () => {
        window.removeEventListener('resize', handleWindowResize);
      };
    };

    void mount().catch((err) => {
      if (disposed) {
        return;
      }
      clearShowAttachingTimer();

      const message = formatError(err);
      setAttachError(message);

      setAttaching(false);
      console.error('Failed to attach app runtime:', err);
    });

    return () => {
      disposed = true;
      clearShowAttachingTimer();
      setAttaching(false);

      if (runtimeCreated) {
        void markRuntimeVisible(installedApp.common.identity.id, false).catch(() => {
          //
        });
      }

      resizeObserver?.disconnect();
      removeWindowResize?.();
      clearDelayedSyncTimers();
    };
  }, [app, containerRef, scheduleSyncBounds, syncBounds]);

  return {
    attaching,
    attachError,
    scheduleSyncBounds,
  };
}
