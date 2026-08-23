import { useEffect, useState, type CSSProperties, type ReactNode } from 'react';
import { platform } from '@tauri-apps/plugin-os';
import { resolveBackgroundTintWithAlpha } from '../utils';

interface SystemModalShellProps {
  children: ReactNode;
  className?: string;
  contentClassName?: string;
  fixedContentHeight?: boolean;
}

interface VisualViewportBounds {
  left: number;
  top: number;
  width: number;
  height: number;
}

function useVisualViewportBounds(enabled: boolean) {
  const [bounds, setBounds] = useState<VisualViewportBounds | null>(null);

  useEffect(() => {
    const viewport = window.visualViewport;

    if (!enabled || !viewport) {
      setBounds(null);
      return;
    }

    const update = () => {
      setBounds({
        left: viewport.pageLeft,
        top: viewport.pageTop,
        width: viewport.width,
        height: viewport.height,
      });
    };

    update();
    viewport.addEventListener('resize', update);
    viewport.addEventListener('scroll', update);

    return () => {
      viewport.removeEventListener('resize', update);
      viewport.removeEventListener('scroll', update);
    };
  }, [enabled]);

  return bounds;
}

export function SystemModalShell({
  children,
  className = '',
  contentClassName = '',
  fixedContentHeight = false,
}: SystemModalShellProps) {
  // WebView2 cannot backdrop-filter content behind this separate webview, so
  // use an opaque theme surface on Windows rather than a see-through modal.
  const isWindows = platform() === 'windows';
  // In Sage's GTK3 multi-webview layout, overlapping transparent WebKitGTK
  // child surfaces do not compose reliably. Bound Linux modal webviews to an
  // opaque surface and render the frosted backdrop in the host webview.
  const isLinux = platform() === 'linux';
  const isIos = platform() === 'ios';
  const visualViewport = useVisualViewportBounds(isIos);

  const viewportStyle: CSSProperties | undefined = visualViewport
    ? {
        position: 'absolute',
        left: visualViewport.left,
        top: visualViewport.top,
        width: visualViewport.width,
        height: visualViewport.height,
      }
    : undefined;

  const contentViewportStyle: CSSProperties | undefined =
    visualViewport && !isLinux
      ? {
          width: Math.min(420, Math.max(1, visualViewport.width - 64)),
          ...(fixedContentHeight
            ? { height: Math.min(620, visualViewport.height * 0.72) }
            : { maxHeight: Math.min(620, visualViewport.height * 0.72) }),
        }
      : undefined;

  return (
    <div
      className={[
        visualViewport ? '' : 'h-screen w-screen',
        'overflow-hidden text-foreground',
        isLinux
          ? ''
          : 'flex items-center justify-center bg-black/20 backdrop-blur-sm',
        className,
      ].join(' ')}
      style={viewportStyle}
    >
      <div
        className={[
          isLinux
            ? 'h-full w-full overflow-hidden border border-border'
            : [
                fixedContentHeight
                  ? 'h-[min(620px,72vh)]'
                  : 'max-h-[min(620px,72vh)]',
                'w-[min(420px,calc(100vw-4rem))] rounded-2xl border border-border shadow-2xl',
                isIos && !fixedContentHeight
                  ? 'overflow-y-auto overscroll-contain'
                  : 'overflow-hidden',
              ].join(' '),
          contentClassName,
        ].join(' ')}
        style={{
          ...contentViewportStyle,
          WebkitOverflowScrolling:
            isIos && !fixedContentHeight ? 'touch' : undefined,
          backdropFilter: isLinux ? undefined : 'blur(80px) saturate(0.55)',
          WebkitBackdropFilter: isLinux
            ? undefined
            : 'blur(80px) saturate(0.55)',
          backgroundColor: resolveBackgroundTintWithAlpha(
            isWindows || isLinux ? 1 : 0.85,
          ),
        }}
      >
        {children}
      </div>
    </div>
  );
}
