import { useCallback, useEffect, useMemo, useRef } from 'react';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';
import { useAppRuntimes } from '@/hooks/useAppRuntimes';

interface CachedRuntimeRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function SystemAppModalLayer() {
  const modalBoundsRef = useRef<HTMLDivElement | null>(null);
  const lastRectByWebviewLabelRef = useRef<Map<string, CachedRuntimeRect>>(
    new Map(),
  );
  const retryTimerRef = useRef<number | null>(null);
  const resizeRafRef = useRef<number | null>(null);
  const resizePumpUntilRef = useRef(0);

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

  const modalRuntimeKey = useMemo(
    () =>
      modalRuntimes
        .map((runtime) => `${runtime.webviewLabel}:${runtime.runtimeId}`)
        .sort()
        .join('\n'),
    [modalRuntimes],
  );

  const scheduleRetry = useCallback(() => {
    if (retryTimerRef.current != null) return;

    retryTimerRef.current = window.setTimeout(() => {
      retryTimerRef.current = null;
      void sync();
    }, 50);
  }, []);

  const sync = useCallback(async () => {
    const bounds = modalBoundsRef.current;
    if (!bounds) return;

    const activeLabels = new Set(
      modalRuntimes.map((runtime) => runtime.webviewLabel),
    );

    for (const cachedLabel of lastRectByWebviewLabelRef.current.keys()) {
      if (!activeLabels.has(cachedLabel)) {
        lastRectByWebviewLabelRef.current.delete(cachedLabel);
      }
    }

    const rect = bounds.getBoundingClientRect();

    const nextRect = {
      left: Math.round(rect.left),
      top: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    };

    let missingWebview = false;

    for (const runtime of modalRuntimes) {
      const webview = await Webview.getByLabel(runtime.webviewLabel).catch(
        () => null,
      );

      if (!webview) {
        missingWebview = true;
        continue;
      }

      const previous = lastRectByWebviewLabelRef.current.get(
        runtime.webviewLabel,
      );

      const unchanged =
        previous &&
        previous.left === nextRect.left &&
        previous.top === nextRect.top &&
        previous.width === nextRect.width &&
        previous.height === nextRect.height;

      if (!unchanged) {
        await webview.setPosition(
          new LogicalPosition(nextRect.left, nextRect.top),
        );
        await webview.setSize(new LogicalSize(nextRect.width, nextRect.height));

        lastRectByWebviewLabelRef.current.set(runtime.webviewLabel, nextRect);
      }

      await webview.show();
    }

    if (missingWebview && modalRuntimes.length > 0) {
      scheduleRetry();
    }
  }, [modalRuntimes, scheduleRetry]);

  const pumpSyncDuringResize = useCallback(() => {
    resizePumpUntilRef.current = performance.now() + 250;

    if (resizeRafRef.current != null) {
      return;
    }

    const tick = () => {
      lastRectByWebviewLabelRef.current.clear();
      void sync();

      if (performance.now() < resizePumpUntilRef.current) {
        resizeRafRef.current = window.requestAnimationFrame(tick);
      } else {
        resizeRafRef.current = null;
      }
    };

    resizeRafRef.current = window.requestAnimationFrame(tick);
  }, [sync]);

  useEffect(() => {
    lastRectByWebviewLabelRef.current.clear();

    requestAnimationFrame(() => {
      void sync();
    });

    const observer = new ResizeObserver(() => {
      pumpSyncDuringResize();
    });

    if (modalBoundsRef.current) {
      observer.observe(modalBoundsRef.current);
    }

    window.addEventListener('resize', pumpSyncDuringResize);

    return () => {
      observer.disconnect();
      window.removeEventListener('resize', pumpSyncDuringResize);

      if (retryTimerRef.current != null) {
        window.clearTimeout(retryTimerRef.current);
        retryTimerRef.current = null;
      }

      if (resizeRafRef.current != null) {
        window.cancelAnimationFrame(resizeRafRef.current);
        resizeRafRef.current = null;
      }
    };
  }, [sync, pumpSyncDuringResize, modalRuntimeKey]);

  if (modalRuntimes.length === 0) {
    return null;
  }

  return (
    <div
      className='pointer-events-none absolute inset-0 z-50 flex items-center justify-center px-6 py-8'
      aria-hidden
    >
      <div className='absolute inset-0 bg-black/20 backdrop-blur-sm' />

      <div
        ref={modalBoundsRef}
        className='relative h-[min(620px,72vh)] w-[min(620px,calc(100vw-4rem))] rounded-2xl border border-border shadow-2xl'
        style={{
          backdropFilter: 'blur(60px) saturate(0.75)',
          WebkitBackdropFilter: 'blur(60px) saturate(0.75)',
          backgroundColor: 'rgb(255 255 255 / 0.12)',
        }}
      />
    </div>
  );
}
