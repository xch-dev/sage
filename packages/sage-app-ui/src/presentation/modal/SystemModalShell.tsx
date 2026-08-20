import type { ReactNode } from 'react';
import { platform } from '@tauri-apps/plugin-os';
import { resolveBackgroundTintWithAlpha } from '../utils';

interface SystemModalShellProps {
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}

export function SystemModalShell({
  children,
  className = '',
  contentClassName = '',
}: SystemModalShellProps) {
  // WebView2 cannot backdrop-filter content behind this separate webview, so
  // use an opaque theme surface on Windows rather than a see-through modal.
  const isWindows = platform() === 'windows';
  // In Sage's GTK3 multi-webview layout, overlapping transparent WebKitGTK
  // child surfaces do not compose reliably. Bound Linux modal webviews to an
  // opaque surface and render the frosted backdrop in the host webview.
  const isLinux = platform() === 'linux';

  return (
    <div
      className={[
        'h-screen w-screen overflow-hidden text-foreground',
        isLinux
          ? ''
          : 'flex items-center justify-center bg-black/20 backdrop-blur-sm',
        className,
      ].join(' ')}
    >
      <div
        className={[
          isLinux
            ? 'h-full w-full overflow-hidden border border-border'
            : 'max-h-[min(620px,72vh)] w-[min(420px,calc(100vw-4rem))] overflow-hidden rounded-2xl border border-border shadow-2xl',
          contentClassName,
        ].join(' ')}
        style={{
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
