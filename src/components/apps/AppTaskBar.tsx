import { Button } from '@/components/ui/button.tsx';
import { Blocks, X } from 'lucide-react';
import clsx from 'clsx';
import { useEffect, useMemo, useRef, useState } from 'react';
import { AppIcon } from '@/components/apps/AppIcon.tsx';
import { ListedSageAppView } from '@/bindings.ts';

type InstalledAppView = Exclude<ListedSageAppView, { kind: 'corrupted' }>;

export interface AppTaskBarTab {
  app: InstalledAppView;
  isActive: boolean;
}

interface Props {
  tabs: AppTaskBarTab[];
  activeAppId: string | null;
  activeAppHasDonation: boolean;
  onOpenApps: () => void;
  onSelectApp: (tab: AppTaskBarTab) => void;
  onCloseApp: (tab: AppTaskBarTab) => void;
  onReorderTabs: (nextAppIds: string[]) => void;
  onOpenDonation: () => void;
}

interface DragState {
  draggedAppId: string;
  pointerOffsetWithinTab: number;
  currentPointerX: number;
  overlayLeftPx: number;
}

const MAX_TAB_WIDTH_PX = 200;
const MIN_TAB_WIDTH_PX = 80;
const TAB_GAP_PX = 4;

function reorderIds(
  ids: string[],
  fromIndex: number,
  toIndex: number,
): string[] {
  if (fromIndex === toIndex) {
    return ids;
  }

  const next = [...ids];
  const [moved] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, moved);
  return next;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function AppTaskBar({
  tabs,
  activeAppHasDonation,
  onOpenApps,
  onSelectApp,
  onCloseApp,
  onReorderTabs,
  onOpenDonation,
}: Props) {
  const tabsViewportRef = useRef<HTMLDivElement | null>(null);
  const tabsStripRef = useRef<HTMLDivElement | null>(null);

  const [dragState, setDragState] = useState<DragState | null>(null);
  const [previewOrder, setPreviewOrder] = useState<string[] | null>(null);
  const [tabsViewportWidthPx, setTabsViewportWidthPx] = useState(0);

  const baseOrder = useMemo(() => tabs.map((tab) => tab.app.common.identity.id), [tabs]);
  const activeOrder = previewOrder ?? baseOrder;

  const tabsById = useMemo(() => {
    return new Map(tabs.map((tab) => [tab.app.common.identity.id, tab] as const));
  }, [tabs]);

  const orderedTabs = useMemo<AppTaskBarTab[]>(() => {
    return activeOrder
      .map((appId) => tabsById.get(appId))
      .filter((tab): tab is AppTaskBarTab => tab != null);
  }, [activeOrder, tabsById]);

  const tabWidthPx = useMemo(() => {
    const count = Math.max(1, tabs.length);

    if (tabsViewportWidthPx <= 0) {
      return MAX_TAB_WIDTH_PX;
    }

    const totalGapPx = Math.max(0, count - 1) * TAB_GAP_PX;
    const availableForTabsPx = Math.max(0, tabsViewportWidthPx - totalGapPx);
    const fittedWidthPx = Math.floor(availableForTabsPx / count);

    return Math.max(
      MIN_TAB_WIDTH_PX,
      Math.min(MAX_TAB_WIDTH_PX, fittedWidthPx),
    );
  }, [tabs.length, tabsViewportWidthPx]);

  const totalStripWidthPx = useMemo(() => {
    if (tabs.length === 0) {
      return 0;
    }

    return tabs.length * tabWidthPx + (tabs.length - 1) * TAB_GAP_PX;
  }, [tabs.length, tabWidthPx]);

  const slotSpanPx = tabWidthPx + TAB_GAP_PX;

  useEffect(() => {
    const viewport = tabsViewportRef.current;
    if (!viewport) {
      return;
    }

    const updateWidth = () => {
      setTabsViewportWidthPx(viewport.getBoundingClientRect().width);
    };

    updateWidth();

    const resizeObserver = new ResizeObserver(() => {
      updateWidth();
    });

    resizeObserver.observe(viewport);
    window.addEventListener('resize', updateWidth);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener('resize', updateWidth);
    };
  }, []);

  useEffect(() => {
    if (!dragState) {
      setPreviewOrder(null);
    }
  }, [dragState]);

  useEffect(() => {
    if (!dragState) {
      return;
    }

    const handlePointerMove = (event: PointerEvent) => {
      setDragState((prev) =>
        prev
          ? {
              ...prev,
              currentPointerX: event.clientX,
            }
          : null,
      );
    };

    const handlePointerUp = () => {
      setDragState((prev) => {
        if (!prev) {
          return null;
        }

        if (previewOrder) {
          onReorderTabs(previewOrder);
        }

        return null;
      });
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [dragState, previewOrder, onReorderTabs]);

  useEffect(() => {
    if (!dragState) {
      return;
    }

    const viewportEl = tabsViewportRef.current;
    if (!viewportEl) {
      return;
    }

    const tabCount = activeOrder.length;
    if (tabCount === 0) {
      return;
    }

    const viewportRect = viewportEl.getBoundingClientRect();

    const minOverlayLeftPx = 0;
    const maxOverlayLeftPx = Math.max(0, totalStripWidthPx - tabWidthPx);

    const rawOverlayLeftPx =
      dragState.currentPointerX -
      viewportRect.left +
      viewportEl.scrollLeft -
      dragState.pointerOffsetWithinTab;

    const clampedOverlayLeftPx = clamp(
      rawOverlayLeftPx,
      minOverlayLeftPx,
      maxOverlayLeftPx,
    );

    const draggedCenterX = clampedOverlayLeftPx + tabWidthPx / 2;
    const nextIndex = clamp(
      Math.floor((draggedCenterX + TAB_GAP_PX / 2) / slotSpanPx),
      0,
      tabCount - 1,
    );

    setDragState((prev) =>
      prev
        ? {
            ...prev,
            overlayLeftPx: clampedOverlayLeftPx,
          }
        : null,
    );

    const currentIndex = activeOrder.indexOf(dragState.draggedAppId);
    if (currentIndex === -1 || nextIndex === currentIndex) {
      return;
    }

    setPreviewOrder(reorderIds(activeOrder, currentIndex, nextIndex));
  }, [dragState, activeOrder, tabWidthPx, slotSpanPx, totalStripWidthPx]);

  return (
    <div className='flex h-12 shrink-0 items-end gap-2 border-b bg-muted/30 px-3 pt-2'>
      <Button
        variant='ghost'
        className='h-9 shrink-0 px-3'
        onClick={onOpenApps}
      >
        <Blocks className='mr-2 h-4 w-4' />
        Apps
      </Button>

      <div
        ref={tabsViewportRef}
        className='min-w-0 flex-1 overflow-x-auto overflow-y-hidden'
      >
        <div
          ref={tabsStripRef}
          className='relative flex h-full items-end gap-1'
          style={{
            width: `${Math.max(totalStripWidthPx, tabsViewportWidthPx)}px`,
          }}
        >
          {orderedTabs.map((tab) => {
            const isDragged =
              dragState?.draggedAppId === tab.app.common.identity.id;

            return (
              <div
                key={tab.app.common.identity.id}
                className={clsx('shrink-0', isDragged && 'opacity-0')}
                style={{ width: `${tabWidthPx}px` }}
              >
                <button
                  type='button'
                  onClick={() => {
                    if (!dragState && !tab.isActive) {
                      onSelectApp(tab);
                    }
                  }}
                  onPointerDown={(event) => {
                    if (event.button !== 0) {
                      return;
                    }

                    onSelectApp(tab);

                    const viewportEl = tabsViewportRef.current;
                    if (!viewportEl) {
                      return;
                    }

                    const viewportRect = viewportEl.getBoundingClientRect();
                    const currentIndex = activeOrder.indexOf(
                      tab.app.common.identity.id,
                    );
                    const slotLeftPx = currentIndex * slotSpanPx;
                    const pointerXWithinStripPx =
                      event.clientX - viewportRect.left + viewportEl.scrollLeft;
                    const pointerOffsetWithinTabPx = clamp(
                      pointerXWithinStripPx - slotLeftPx,
                      0,
                      tabWidthPx,
                    );

                    setPreviewOrder((prev) => prev ?? baseOrder);
                    setDragState({
                      draggedAppId: tab.app.common.identity.id,
                      pointerOffsetWithinTab: pointerOffsetWithinTabPx,
                      currentPointerX: event.clientX,
                      overlayLeftPx: slotLeftPx,
                    });
                  }}
                  className={clsx(
                    'group flex h-9 w-full items-center gap-2 rounded-t-md border border-b-0 px-3 text-left transition-[background-color,color] duration-150 select-none',
                    tab.isActive
                      ? 'bg-background'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80',
                  )}
                >
                  <div className='flex h-4 w-4 shrink-0 items-center justify-center overflow-hidden rounded-sm'>
                    <AppIcon app={tab.app} />
                  </div>

                  <span className='min-w-0 flex-1 truncate text-sm font-medium'>
                    {tab.app.common.activeSnapshot.manifest.name}
                  </span>

                  <span
                    className={clsx(
                      'shrink-0',
                      tab.isActive
                        ? 'opacity-100'
                        : 'opacity-0 transition-opacity group-hover:opacity-100',
                    )}
                  >
                    <Button
                      type='button'
                      variant='ghost'
                      size='icon'
                      className='h-6 w-6'
                      onClick={(event) => {
                        event.stopPropagation();
                        onCloseApp(tab);
                      }}
                    >
                      <X className='h-3.5 w-3.5' />
                    </Button>
                  </span>
                </button>
              </div>
            );
          })}

          {dragState
            ? (() => {
                const draggedTab = tabsById.get(dragState.draggedAppId);
                if (!draggedTab) {
                  return null;
                }

                return (
                  <div
                    className='pointer-events-none absolute bottom-0 z-20'
                    style={{
                      left: `${dragState.overlayLeftPx}px`,
                      width: `${tabWidthPx}px`,
                    }}
                  >
                    <div
                      className={clsx(
                        'flex h-9 w-full items-center gap-2 rounded-t-md border border-b-0 px-3 text-left shadow-sm',
                        draggedTab.isActive
                          ? 'bg-background'
                          : 'bg-muted text-muted-foreground',
                      )}
                    >
                      <div className='flex h-4 w-4 shrink-0 items-center justify-center overflow-hidden rounded-sm'>
                        <AppIcon app={draggedTab.app} />
                      </div>

                      <span className='min-w-0 flex-1 truncate text-sm font-medium'>
                        {draggedTab.app.common.activeSnapshot.manifest.name}
                      </span>

                      <span className='shrink-0 opacity-100'>
                        <div className='h-6 w-6' />
                      </span>
                    </div>
                  </div>
                );
              })()
            : null}
        </div>
      </div>

      {activeAppHasDonation && (
        <Button
          variant='default'
          className='h-9 shrink-0 bg-amber-500 px-3 text-black hover:bg-amber-600'
          onClick={onOpenDonation}
        >
          Support developer
        </Button>
      )}
    </div>
  );
}
