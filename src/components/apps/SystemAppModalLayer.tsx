import { useCallback, useEffect, useMemo, useRef } from 'react';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';
import { useAppRuntimes } from '@/hooks/useAppRuntimes';

export function SystemAppModalLayer() {
  const modalBoundsRef = useRef<HTMLDivElement | null>(null);
  const lastRectByWebviewLabelRef = useRef<
    Map<string, { left: number; top: number; width: number; height: number }>
  >(new Map());

  const runtimes = useAppRuntimes({ includeInternal: true });

  const modalRuntimes = useMemo(
    () =>
      runtimes.filter(
        (runtime) =>
          runtime.app.kind === 'system' &&
          runtime.app.presentation === 'AppModal' &&
          runtime.visibility === 'Visible',
      ),
    [runtimes],
  );

  const modalRuntimeLabels = useMemo(
    () => modalRuntimes.map((runtime) => runtime.webviewLabel).sort(),
    [modalRuntimes],
  );

  const modalRuntimeLabelsKey = modalRuntimeLabels.join('\n');

  const sync = useCallback(async () => {
    const bounds = modalBoundsRef.current;
    if (!bounds) return;

    const rect = bounds.getBoundingClientRect();

    const nextRect = {
      left: Math.round(rect.left),
      top: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    };

    for (const webviewLabel of modalRuntimeLabels) {
      const previous = lastRectByWebviewLabelRef.current.get(webviewLabel);

      const unchanged =
        previous &&
        previous.left === nextRect.left &&
        previous.top === nextRect.top &&
        previous.width === nextRect.width &&
        previous.height === nextRect.height;

      if (unchanged) {
        continue;
      }

      const webview = await Webview.getByLabel(webviewLabel).catch(() => null);
      if (!webview) continue;

      await webview.setPosition(
        new LogicalPosition(nextRect.left, nextRect.top),
      );
      await webview.setSize(new LogicalSize(nextRect.width, nextRect.height));
      await webview.show();

      lastRectByWebviewLabelRef.current.set(webviewLabel, nextRect);
    }
  }, [modalRuntimeLabels]);

  useEffect(() => {
    void sync();

    const observer = new ResizeObserver(() => {
      void sync();
    });

    if (modalBoundsRef.current) {
      observer.observe(modalBoundsRef.current);
    }

    window.addEventListener('resize', sync);

    return () => {
      observer.disconnect();
      window.removeEventListener('resize', sync);
    };
  }, [sync, modalRuntimeLabelsKey]);

  if (modalRuntimes.length === 0) {
    return null;
  }

  return (
    <div
      className='pointer-events-none absolute inset-0 z-50 flex items-start justify-center bg-black/30 px-6 pb-6 pt-[12vh]'
      aria-hidden
    >
      <div
        ref={modalBoundsRef}
        className='h-[min(620px,72vh)] w-[min(620px,calc(100vw-4rem))]'
      />
    </div>
  );
}
