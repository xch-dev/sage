import { useEffect, useMemo, useState } from 'react';
import {
  focusRuntime,
  hideRuntime,
  killRuntime,
  listRuntimes,
  onRuntimesChanged,
  type RuntimeRecord,
} from './taskManagerApi';
import type { SageAppRuntimeRecordView } from '@sage-system-app/sdk';

function formatDuration(ms: number) {
  const s = Math.floor(Math.max(0, ms) / 1000);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;

  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m`;
  if (m > 0) return `${m}m ${String(sec).padStart(2, '0')}s`;
  return `${sec}s`;
}

function formatTime(value: number) {
  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(value));
}

function runtimeAppId(runtime: SageAppRuntimeRecordView) {
  return runtime.app.common.identity.id;
}

function runtimeAppName(runtime: SageAppRuntimeRecordView) {
  return runtime.app.common.activeSnapshot.manifest.name;
}

function runtimeKind(runtime: SageAppRuntimeRecordView) {
  return runtime.app.kind === 'user' ? 'User' : 'System';
}

function runtimeVisible(runtime: SageAppRuntimeRecordView) {
  return runtime.visibility === 'Visible';
}

function ActionButton({
  children,
  danger,
  disabled,
  onClick,
}: {
  children: string;
  danger?: boolean;
  disabled?: boolean;
  onClick: () => void | Promise<void>;
}) {
  return (
    <button
      disabled={disabled}
      onClick={() => void onClick()}
      className={[
        'h-8 rounded-md border px-3 text-xs font-semibold transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        danger
          ? 'border-destructive bg-destructive text-destructive-foreground hover:bg-destructive/90'
          : 'border-border bg-secondary text-secondary-foreground hover:bg-secondary/80',
      ].join(' ')}
    >
      {children}
    </button>
  );
}

function Metric({ label, value }: { label: string; value: string | number }) {
  return (
    <div className='rounded-lg border border-border bg-muted px-3 py-2'>
      <div className='text-[11px] text-muted-foreground'>{label}</div>
      <div className='mt-1 text-sm font-bold text-foreground'>{value}</div>
    </div>
  );
}

export function App() {
  const [runtimes, setRuntimes] = useState<RuntimeRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [busyAppId, setBusyAppId] = useState<string | null>(null);
  const [now, setNow] = useState(() => Date.now());

  async function refresh() {
    setLoading(true);
    try {
      setRuntimes(await listRuntimes());
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void (async () => {
      const tauri = (window as any).__TAURI__;

      console.log('[probe] __TAURI__', tauri);
      console.log('[probe] modules', Object.keys(tauri ?? {}));

      const webviewApi = tauri?.webview;
      const windowApi = tauri?.window;
      const eventApi = tauri?.event;

      console.log('[probe] webview api keys', Object.keys(webviewApi ?? {}));
      console.log('[probe] window api keys', Object.keys(windowApi ?? {}));
      console.log('[probe] event api keys', Object.keys(eventApi ?? {}));

      try {
        const currentWebview = webviewApi?.getCurrentWebview?.();
        console.log('[probe] current webview', currentWebview);
        console.log(
          '[probe] current webview keys',
          Object.keys(currentWebview ?? {}),
        );

        await currentWebview?.listen?.(
          'probe-current-webview-listen',
          (event: unknown) => {
            console.log('[probe] current webview received event', event);
          },
        );

        console.log('[probe] current webview listen OK');
      } catch (err) {
        console.log('[probe] current webview listen DENIED/FAILED', err);
      }

      try {
        const allWebviews = await webviewApi?.getAllWebviews?.();
        console.log('[probe] getAllWebviews OK', allWebviews);

        for (const webview of allWebviews ?? []) {
          console.log(
            '[probe] webview item',
            webview,
            Object.keys(webview ?? {}),
          );

          try {
            await webview.listen?.(
              'probe-other-webview-listen',
              (event: unknown) => {
                console.log(
                  '[probe] received on listed webview',
                  webview.label,
                  event,
                );
              },
            );

            console.log('[probe] listen on listed webview OK', webview.label);
          } catch (err) {
            console.log(
              '[probe] listen on listed webview DENIED/FAILED',
              webview.label,
              err,
            );
          }
        }
      } catch (err) {
        console.log('[probe] getAllWebviews DENIED/FAILED', err);
      }

      try {
        const allWindows = await windowApi?.getAllWindows?.();
        console.log('[probe] getAllWindows OK', allWindows);

        for (const win of allWindows ?? []) {
          console.log('[probe] window item', win, Object.keys(win ?? {}));

          try {
            await win.listen?.('probe-window-listen', (event: unknown) => {
              console.log(
                '[probe] received on listed window',
                win.label,
                event,
              );
            });

            console.log('[probe] listen on listed window OK', win.label);
          } catch (err) {
            console.log(
              '[probe] listen on listed window DENIED/FAILED',
              win.label,
              err,
            );
          }
        }
      } catch (err) {
        console.log('[probe] getAllWindows DENIED/FAILED', err);
      }

      try {
        await eventApi?.listen?.('probe-global-listen', (event: unknown) => {
          console.log('[probe] received global event', event);
        });

        console.log('[probe] global event listen OK');
      } catch (err) {
        console.log('[probe] global event listen DENIED/FAILED', err);
      }

      try {
        await eventApi?.emit?.('probe-global-emit', { hello: 'world' });
        console.log('[probe] global emit OK');
      } catch (err) {
        console.log('[probe] global emit DENIED/FAILED', err);
      }
    })();
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    let disposed = false;

    void refresh();

    const unsubscribe = onRuntimesChanged((event) => {
      if (disposed) return;
      setRuntimes(event.runtimes);
      setLoading(false);
    });

    return () => {
      disposed = true;
      unsubscribe();
    };
  }, []);

  const sorted = useMemo(
    () =>
      [...runtimes].sort((a, b) =>
        runtimeAppName(a).localeCompare(runtimeAppName(b)),
      ),
    [runtimes],
  );

  async function runAction(appId: string, action: () => Promise<unknown>) {
    setBusyAppId(appId);
    try {
      await action();
    } finally {
      setBusyAppId(null);
    }
  }

  return (
    <div className='min-h-screen bg-background text-foreground font-sans'>
      <main className='mx-auto max-w-6xl p-6'>
        <header className='mb-5 flex items-start justify-between gap-4'>
          <div>
            <div className='text-sm text-muted-foreground'>
              Built-in system app
            </div>
            <h1 className='mt-1 text-3xl font-bold tracking-tight'>
              Task Manager
            </h1>
            <div className='mt-1 text-sm text-muted-foreground'>
              {runtimes.length} runtimes · updated {formatTime(now)}
            </div>
          </div>

          <ActionButton disabled={loading} onClick={refresh}>
            {loading ? 'Refreshing…' : 'Refresh'}
          </ActionButton>
        </header>

        <section className='mb-4 grid grid-cols-1 gap-3 sm:grid-cols-3'>
          <Metric label='Runtimes' value={runtimes.length} />
          <Metric
            label='Visible'
            value={runtimes.filter(runtimeVisible).length}
          />
          <Metric
            label='Hidden'
            value={
              runtimes.filter((runtime) => !runtimeVisible(runtime)).length
            }
          />
        </section>

        <section className='overflow-hidden rounded-xl border border-border bg-card text-card-foreground'>
          {loading && sorted.length === 0 ? (
            <div className='p-5 text-muted-foreground'>Loading runtimes…</div>
          ) : sorted.length === 0 ? (
            <div className='p-5 text-muted-foreground'>No running apps.</div>
          ) : (
            sorted.map((runtime, index) => {
              const appId = runtimeAppId(runtime);
              const visible = runtimeVisible(runtime);
              const busy = busyAppId === appId;

              return (
                <div
                  key={runtime.runtimeId}
                  className={[
                    'grid gap-4 p-4 md:grid-cols-[minmax(0,1fr)_auto]',
                    index === 0 ? '' : 'border-t border-border',
                  ].join(' ')}
                >
                  <div className='min-w-0'>
                    <div className='flex min-w-0 items-center gap-2'>
                      <span
                        className={[
                          'h-2.5 w-2.5 shrink-0 rounded-full',
                          visible ? 'bg-primary' : 'bg-muted-foreground',
                        ].join(' ')}
                      />

                      <strong className='min-w-0 truncate text-sm font-semibold'>
                        {runtimeAppName(runtime)}
                      </strong>

                      <span
                        className={[
                          'rounded-full px-2 py-0.5 text-[11px] font-bold',
                          visible
                            ? 'bg-primary text-primary-foreground'
                            : 'bg-muted text-muted-foreground',
                        ].join(' ')}
                      >
                        {visible ? 'Visible' : 'Hidden'}
                      </span>
                    </div>

                    <div className='mt-1 truncate text-xs text-muted-foreground'>
                      {appId}
                    </div>

                    <div className='mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4'>
                      <Metric
                        label='Uptime'
                        value={formatDuration(now - runtime.startedAt)}
                      />
                      <Metric
                        label='Started'
                        value={formatTime(runtime.startedAt)}
                      />
                      <Metric label='Kind' value={runtimeKind(runtime)} />
                      <Metric label='Mode' value={runtime.mode} />
                    </div>
                  </div>

                  <div className='flex items-start gap-2'>
                    <ActionButton
                      disabled={busy || visible}
                      onClick={() =>
                        runAction(appId, () => focusRuntime(appId))
                      }
                    >
                      Focus
                    </ActionButton>

                    <ActionButton
                      disabled={busy || !visible}
                      onClick={() => runAction(appId, () => hideRuntime(appId))}
                    >
                      Hide
                    </ActionButton>

                    <ActionButton
                      danger
                      disabled={busy}
                      onClick={() =>
                        runAction(appId, async () => {
                          await killRuntime(appId);
                          await refresh();
                        })
                      }
                    >
                      Kill
                    </ActionButton>
                  </div>
                </div>
              );
            })
          )}
        </section>
      </main>
    </div>
  );
}
