import React, { useEffect, useState } from 'react';
import {
  ensureInlineRuntime,
  getRuntimeWebview,
  markRuntimeVisible,
} from '@/lib/apps/runtimeRegistry';
import type { SageAppView } from '@/bindings';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';

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
  const [webviewLabel, setWebviewLabel] = useState<string | null>(null);

  const appId = app?.common.identity.id ?? null;

  const { scheduleSyncBounds } = useRuntimeWebviewBounds({
    webviewLabel,
    containerRef,
    enabled: !!appId && !!webviewLabel,
  });

  useEffect(() => {
    setAttachError(null);
    setWebviewLabel(null);

    if (!app || !containerRef.current) {
      setAttaching(false);
      return;
    }

    const installedApp = app;
    const installedAppId = installedApp.common.identity.id;

    let disposed = false;
    let runtimeCreated = false;
    let showAttachingTimer: number | null = null;

    const clearShowAttachingTimer = () => {
      if (showAttachingTimer !== null) {
        window.clearTimeout(showAttachingTimer);
        showAttachingTimer = null;
      }
    };

    const mount = async () => {
      showAttachingTimer = window.setTimeout(() => {
        if (!disposed) {
          setAttaching(true);
        }
      }, 200);

      await ensureInlineRuntime(installedApp);
      runtimeCreated = true;

      if (disposed) return;

      const webview = await getRuntimeWebview(installedAppId);

      if (!webview) {
        throw new Error(`webview not found for app ${installedAppId}`);
      }

      setWebviewLabel(webview.label);

      await markRuntimeVisible(installedAppId, true);

      if (disposed) return;

      clearShowAttachingTimer();
      setAttachError(null);
      setAttaching(false);
    };

    void mount().catch((err) => {
      if (disposed) return;

      clearShowAttachingTimer();

      setAttachError(formatError(err));
      setAttaching(false);

      console.error('Failed to attach app runtime:', err);
    });

    return () => {
      disposed = true;
      clearShowAttachingTimer();
      setAttaching(false);
      setWebviewLabel(null);

      if (runtimeCreated) {
        void markRuntimeVisible(installedAppId, false).catch(() => {
          //
        });
      }
    };
  }, [app, containerRef]);

  useEffect(() => {
    if (!webviewLabel) {
      return;
    }

    scheduleSyncBounds();
  }, [webviewLabel, scheduleSyncBounds]);

  return {
    attaching,
    attachError,
    scheduleSyncBounds,
  };
}
