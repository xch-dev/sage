import { useMemo, useRef } from 'react';
import { useAppRuntimes } from '@/hooks/useAppRuntimes';
import { useRuntimeWebviewBounds } from '@/hooks/useRuntimeWebviewBounds';

export function SystemAppModalLayer() {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const runtimes = useAppRuntimes({ includeInternal: true });

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
