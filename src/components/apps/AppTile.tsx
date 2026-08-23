import type { SageAppView } from '@/bindings';
import type { SandboxLaunchDecision } from '@/lib/apps/sandboxPolicy';
import { AppIcon } from '@/components/apps/AppIcon.tsx';
import React, { useEffect, useRef } from 'react';

export interface AppTilePressPoint {
  clientX: number;
  clientY: number;
}

interface Props {
  app: SageAppView;
  launchDecision: SandboxLaunchDecision;
  onOpen: () => void;
  onContextMenu: (event: React.MouseEvent<HTMLDivElement>) => void;
  onLongPress: (point: AppTilePressPoint) => void;
}

const LONG_PRESS_DURATION_MS = 550;
const LONG_PRESS_MOVE_TOLERANCE_PX = 12;

export function AppTile({
  app,
  launchDecision,
  onOpen,
  onContextMenu,
  onLongPress,
}: Props) {
  const longPressTimerRef = useRef<number | null>(null);
  const pressStartRef = useRef<AppTilePressPoint | null>(null);
  const suppressNextClickRef = useRef(false);

  const isChecking =
    !launchDecision.allowed &&
    launchDecision.title === 'Sandbox tests are still running';

  const isBlocked = !launchDecision.allowed && !isChecking;

  function cancelLongPress() {
    if (longPressTimerRef.current !== null) {
      window.clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
    pressStartRef.current = null;
  }

  useEffect(
    () => () => {
      if (longPressTimerRef.current !== null) {
        window.clearTimeout(longPressTimerRef.current);
      }
      pressStartRef.current = null;
    },
    [],
  );

  function handleOpen(event?: React.MouseEvent<HTMLDivElement>) {
    if (suppressNextClickRef.current) {
      event?.preventDefault();
      suppressNextClickRef.current = false;
      return;
    }

    if (!launchDecision.allowed) return;
    onOpen();
  }

  return (
    <div
      role='button'
      tabIndex={launchDecision.allowed ? 0 : -1}
      onClick={handleOpen}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          handleOpen();
        }
      }}
      onContextMenu={onContextMenu}
      onPointerDown={(event) => {
        if (event.pointerType === 'mouse' || !event.isPrimary) {
          return;
        }

        cancelLongPress();

        const point = { clientX: event.clientX, clientY: event.clientY };
        pressStartRef.current = point;
        longPressTimerRef.current = window.setTimeout(() => {
          longPressTimerRef.current = null;
          pressStartRef.current = null;
          suppressNextClickRef.current = true;
          onLongPress(point);
        }, LONG_PRESS_DURATION_MS);
      }}
      onPointerMove={(event) => {
        const start = pressStartRef.current;
        if (!start) {
          return;
        }

        if (
          Math.hypot(
            event.clientX - start.clientX,
            event.clientY - start.clientY,
          ) > LONG_PRESS_MOVE_TOLERANCE_PX
        ) {
          cancelLongPress();
        }
      }}
      onPointerUp={cancelLongPress}
      onPointerCancel={cancelLongPress}
      onPointerLeave={cancelLongPress}
      aria-disabled={!launchDecision.allowed}
      className='relative group flex cursor-pointer flex-col items-center gap-3 rounded-2xl p-4 text-center transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring aria-disabled:cursor-default'
      style={{ WebkitTouchCallout: 'none' }}
    >
      {isChecking || isBlocked ? (
        <div className='absolute inset-0 z-10 flex items-center justify-center rounded-2xl bg-background/55 backdrop-blur-[1px]'>
          {isChecking ? (
            <div className='flex flex-col items-center gap-2'>
              <div className='h-5 w-5 animate-spin rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground' />
              <div className='text-xs text-muted-foreground'>Checking…</div>
            </div>
          ) : (
            <div className='px-3 text-center text-xs font-medium text-amber-600'>
              Blocked
            </div>
          )}
        </div>
      ) : null}

      <div className='flex h-20 w-20 items-center justify-center overflow-hidden rounded-2xl border bg-background shadow-sm'>
        <AppIcon app={app} />
      </div>

      <div className='min-w-0 w-full'>
        <div className='truncate text-sm font-medium'>
          {app.common.activeSnapshot.manifest.name}
        </div>

        {isBlocked ? (
          <div className='relative z-20 mt-1 text-xs text-amber-600'>
            {launchDecision.title}
          </div>
        ) : null}
      </div>
    </div>
  );
}
