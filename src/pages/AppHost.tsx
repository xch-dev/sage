import { useEffect, useMemo, useRef } from 'react';
import { Navigate, useParams } from 'react-router-dom';
import { useApps } from '@/contexts/AppsContext';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';

export function AppHost() {
  const { appId = '' } = useParams();
  const containerRef = useRef<HTMLDivElement | null>(null);

  const {
    taskbarRuntimesByHostWindowLabel,
    currentHostWindowLabel,
  } = useApps();

  const taskbarRuntimesForWindow = useMemo(() => {
    if (!currentHostWindowLabel) {
      return [];
    }

    return taskbarRuntimesByHostWindowLabel[currentHostWindowLabel] ?? [];
  }, [currentHostWindowLabel, taskbarRuntimesByHostWindowLabel]);

  const runtime = useMemo(() => {
    return (
      taskbarRuntimesForWindow.find((runtime) => {
        return runtime.app.common.identity.id === appId;
      }) ?? null
    );
  }, [taskbarRuntimesForWindow, appId]);

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
    return <Navigate to='/apps' replace />;
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
