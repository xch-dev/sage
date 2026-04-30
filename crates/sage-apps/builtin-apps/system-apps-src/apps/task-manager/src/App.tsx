import { useEffect, useMemo, useState } from 'react';
import {
  focusRuntime,
  hideRuntime,
  killRuntime,
  listRuntimes,
  onRuntimesChanged,
  type RuntimeRecord,
} from './taskManagerApi';
import type {
  SageAppRuntimeRecordView,
} from '@sage-system-app/sdk';

function formatDuration(ms: number) {
  const safeMs = Math.max(0, ms);
  const s = Math.floor(safeMs / 1000);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;

  if (h > 0) {
    return `${h}h ${String(m).padStart(2, '0')}m ${String(sec).padStart(2, '0')}s`;
  }

  if (m > 0) {
    return `${m}m ${String(sec).padStart(2, '0')}s`;
  }

  return `${sec}s`;
}

function formatTime(value: number) {
  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(value));
}

function statusColor() {
  /*if (normalized.includes('running') || normalized.includes('active')) {
    return '#34d399';
  }

  if (normalized.includes('stopping') || normalized.includes('starting')) {
    return '#fbbf24';
  }

  if (normalized.includes('failed') || normalized.includes('error')) {
    return '#fb7185';
  }*/

  return '#94a3b8';
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
      style={{
        height: 34,
        padding: '0 12px',
        borderRadius: 999,
        border: danger
          ? '1px solid rgba(251,113,133,0.35)'
          : '1px solid rgba(255,255,255,0.12)',
        background: danger
          ? 'rgba(251,113,133,0.10)'
          : 'rgba(255,255,255,0.055)',
        color: danger ? '#fecdd3' : '#f8fafc',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.45 : 1,
        fontSize: 13,
        fontWeight: 600,
        backdropFilter: 'blur(16px)',
      }}
    >
      {children}
    </button>
  );
}

function Metric({ label, value }: { label: string; value: string | number }) {
  return (
    <div
      style={{
        minWidth: 88,
        padding: '10px 12px',
        borderRadius: 14,
        background: 'rgba(255,255,255,0.045)',
        border: '1px solid rgba(255,255,255,0.08)',
      }}
    >
      <div style={{ fontSize: 11, color: 'rgba(248,250,252,0.48)' }}>
        {label}
      </div>
      <div style={{ marginTop: 4, fontSize: 14, fontWeight: 700 }}>{value}</div>
    </div>
  );
}

function runtimeAppId(runtime: SageAppRuntimeRecordView): string {
  return runtime.app.common.identity.id;
}

function runtimeAppName(runtime: SageAppRuntimeRecordView): string {
  return runtime.app.common.activeSnapshot.manifest.name;
}

function runtimeKind(runtime: SageAppRuntimeRecordView): 'User' | 'System' {
  return runtime.app.kind === 'user' ? 'User' : 'System';
}

function runtimeVisible(runtime: SageAppRuntimeRecordView): boolean {
  return runtime.visibility === 'Visible';
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
    const intervalId = window.setInterval(() => {
      setNow(Date.now());
    }, 1000);

    return () => {
      window.clearInterval(intervalId);
    };
  }, []);

  useEffect(() => {
    let disposed = false;

    void refresh();

    const unsubscribe = onRuntimesChanged((event) => {
      if (disposed) {
        return;
      }

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

  const totals = useMemo(
    () => ({
      runtimes: runtimes.length,
    }),
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
    <div
      style={{
        fontFamily:
          'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
        minHeight: '100vh',
        color: '#f8fafc',
        background:
          'radial-gradient(circle at top left, rgba(59,130,246,0.22), transparent 34%), radial-gradient(circle at top right, rgba(168,85,247,0.16), transparent 32%), linear-gradient(180deg, #09090b 0%, #0f1117 100%)',
      }}
    >
      <div style={{ maxWidth: 1180, margin: '0 auto', padding: 24 }}>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            gap: 20,
            marginBottom: 22,
          }}
        >
          <div>
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 8,
                padding: '5px 10px',
                borderRadius: 999,
                border: '1px solid rgba(255,255,255,0.10)',
                background: 'rgba(255,255,255,0.045)',
                color: 'rgba(248,250,252,0.68)',
                fontSize: 12,
                marginBottom: 10,
              }}
            >
              <span
                style={{
                  width: 7,
                  height: 7,
                  borderRadius: 999,
                  background: '#34d399',
                  boxShadow: '0 0 18px rgba(52,211,153,0.75)',
                }}
              />
              Built-in system app
            </div>

            <h1
              style={{
                margin: 0,
                fontSize: 34,
                letterSpacing: '-0.04em',
                lineHeight: 1,
              }}
            >
              Task Manager
            </h1>

            <div
              style={{
                marginTop: 8,
                color: 'rgba(248,250,252,0.56)',
                fontSize: 14,
              }}
            >
              Live runtime control · updated {formatTime(now)}
            </div>
          </div>

          <button
            onClick={() => void refresh()}
            disabled={loading}
            style={{
              height: 40,
              padding: '0 16px',
              borderRadius: 999,
              border: '1px solid rgba(255,255,255,0.14)',
              background:
                'linear-gradient(180deg, rgba(255,255,255,0.11), rgba(255,255,255,0.055))',
              color: '#f8fafc',
              cursor: loading ? 'not-allowed' : 'pointer',
              opacity: loading ? 0.6 : 1,
              fontWeight: 700,
              boxShadow: '0 14px 40px rgba(0,0,0,0.25)',
            }}
          >
            {loading ? 'Refreshing…' : 'Refresh'}
          </button>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(4, minmax(0, 1fr))',
            gap: 12,
            marginBottom: 18,
          }}
        >
          <Metric label='Runtimes' value={totals.runtimes} />
        </div>

        <div
          style={{
            overflow: 'hidden',
            borderRadius: 24,
            border: '1px solid rgba(255,255,255,0.10)',
            background: 'rgba(15,23,42,0.48)',
            boxShadow:
              '0 24px 80px rgba(0,0,0,0.38), inset 0 1px 0 rgba(255,255,255,0.06)',
            backdropFilter: 'blur(22px)',
          }}
        >
          {loading && sorted.length === 0 ? (
            <div style={{ padding: 22, color: 'rgba(248,250,252,0.62)' }}>
              Loading runtimes…
            </div>
          ) : sorted.length === 0 ? (
            <div style={{ padding: 28, color: 'rgba(248,250,252,0.62)' }}>
              No running apps.
            </div>
          ) : (
            sorted.map((runtime, index) => {
              const busy = busyAppId === runtimeAppId(runtime);
              const color = statusColor();

              return (
                <div
                  key={runtime.runtimeId}
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'minmax(0, 1fr) auto',
                    gap: 18,
                    padding: 18,
                    borderTop:
                      index === 0
                        ? 'none'
                        : '1px solid rgba(255,255,255,0.075)',
                    background:
                      index % 2 === 0
                        ? 'rgba(255,255,255,0.018)'
                        : 'transparent',
                  }}
                >
                  <div style={{ minWidth: 0 }}>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 10,
                        minWidth: 0,
                      }}
                    >
                      <span
                        style={{
                          width: 9,
                          height: 9,
                          borderRadius: 999,
                          background: color,
                          boxShadow: `0 0 18px ${color}`,
                          flexShrink: 0,
                        }}
                      />

                      <div
                        style={{
                          minWidth: 0,
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                          fontSize: 16,
                          fontWeight: 750,
                          letterSpacing: '-0.01em',
                        }}
                      >
                        {runtimeAppName(runtime)}
                      </div>

                      {runtimeVisible(runtime) ? (
                        <span
                          style={{
                            padding: '3px 8px',
                            borderRadius: 999,
                            background: 'rgba(52,211,153,0.10)',
                            border: '1px solid rgba(52,211,153,0.20)',
                            color: '#bbf7d0',
                            fontSize: 11,
                            fontWeight: 700,
                          }}
                        >
                          Visible
                        </span>
                      ) : (
                        <span
                          style={{
                            padding: '3px 8px',
                            borderRadius: 999,
                            background: 'rgba(148,163,184,0.10)',
                            border: '1px solid rgba(148,163,184,0.16)',
                            color: '#cbd5e1',
                            fontSize: 11,
                            fontWeight: 700,
                          }}
                        >
                          Hidden
                        </span>
                      )}
                    </div>

                    <div
                      style={{
                        marginTop: 7,
                        fontSize: 12,
                        color: 'rgba(248,250,252,0.45)',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {runtimeAppId(runtime)}
                    </div>

                    <div
                      style={{
                        display: 'flex',
                        flexWrap: 'wrap',
                        gap: 8,
                        marginTop: 12,
                      }}
                    >
                      <Metric
                        label='Uptime'
                        value={formatDuration(now - runtime.startedAt)}
                      />
                      <Metric
                        label='Started'
                        value={formatTime(runtime.startedAt)}
                      />
                      <Metric
                        label='Kind'
                        value={runtimeKind(runtime)}
                      />
                      <Metric label='Mode' value={runtime.mode} />
                    </div>
                  </div>

                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: 8,
                      flexShrink: 0,
                    }}
                  >
                    <ActionButton
                      disabled={busy}
                      onClick={() =>
                        runAction(runtimeAppId(runtime), () =>
                          focusRuntime(runtimeAppId(runtime)),
                        )
                      }
                    >
                      Focus
                    </ActionButton>

                    <ActionButton
                      disabled={busy}
                      onClick={() =>
                        runAction(runtimeAppId(runtime), () =>
                          hideRuntime(runtimeAppId(runtime)),
                        )
                      }
                    >
                      Hide
                    </ActionButton>

                    <ActionButton
                      danger
                      disabled={busy}
                      onClick={() =>
                        runAction(runtimeAppId(runtime), async () => {
                          await killRuntime(runtimeAppId(runtime));
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
        </div>
      </div>
    </div>
  );
}
