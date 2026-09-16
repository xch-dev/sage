import { useMemo, useRef } from 'react';
import { platform } from '@tauri-apps/plugin-os';
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

  if (platform() === 'linux') {
    return (
      <div className='pointer-events-auto absolute inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm'>
        <div
          ref={containerRef}
          style={{
            width: 'min(420px, calc(100% - 4rem))',
            height: 'min(620px, 72%)',
          }}
        />
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className='absolute inset-0 z-50 pointer-events-none'
    />
  );
}
