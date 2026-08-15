import { useEffect, useMemo, useRef } from 'react';
import { useApps } from '@/contexts/AppsContext';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';

export function AppHost() {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const { activeTaskbarRuntime, getTaskbarRuntime } = useApps();

  const appId = activeTaskbarRuntime?.appId ?? null;

  const runtime = useMemo(() => {
    return appId ? getTaskbarRuntime(appId) : null;
  }, [getTaskbarRuntime, appId]);

  const webviewLabel = runtime?.webviewLabel ?? null;

  const { scheduleSyncBounds } = useRuntimeWebviewBounds({
    webviewLabel,
    containerRef,
    enabled: !!webviewLabel,
  });

  useEffect(() => {
    if (!webviewLabel) {
      return;
    }

    scheduleSyncBounds();
  }, [webviewLabel, scheduleSyncBounds]);

  if (!runtime) {
    return null;
  }

  return (
    <div className='flex h-full min-h-0 w-full flex-col overflow-hidden'>
      <div className='min-h-0 flex-1'>
        <div
          ref={containerRef}
          className='h-full w-full overflow-hidden bg-background'
        />
      </div>
    </div>
  );
}
