import { useEffect, useMemo, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useApps } from '@/contexts/AppsContext';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';

export function AppHost() {
  const { appId = '' } = useParams();
  const containerRef = useRef<HTMLDivElement | null>(null);

  const { getTaskbarRuntime} = useApps();

  const runtime = useMemo(() => {
    return getTaskbarRuntime(appId);
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
