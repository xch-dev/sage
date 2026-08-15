import { useMemo, useRef } from 'react';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';
import { useApps } from '@/contexts/AppsContext.tsx';

export function SystemAppModalLayer() {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const { runtimes } = useApps();

  const modalRuntime = useMemo(() => {
    return runtimes.find(
      (runtime) =>
        runtime.app.kind === 'system' &&
        runtime.presentation.kind === 'Modal' &&
        runtime.visibility === 'Visible',
    );
  }, [runtimes]);

  useRuntimeWebviewBounds({
    webviewLabel: modalRuntime?.webviewLabel ?? null,
    containerRef,
    enabled: !!modalRuntime,
  });

  if (!modalRuntime) {
    return null;
  }

  return (
    <div
      ref={containerRef}
      className='absolute inset-0 z-50 pointer-events-none'
    />
  );
}
