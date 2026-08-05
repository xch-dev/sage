import { AppIcon, AppModalShell } from '@sage-app/ui';
import { formatSageError } from '@sage-system-app/sdk';
import { useEffect, useMemo, useState } from 'react';
import {
  closeSelf,
  getSandboxState,
  onSandboxStateChanged,
  rerunSandboxTests,
  type SandboxStateView,
} from './sandboxApi';
import { SandboxResultList } from './components/SandboxResultList';
import { SandboxTabs } from './components/SandboxTabs';
import {
  isCurrentSandboxRunActive,
  selectedSandboxState,
  type SandboxTab,
} from './sandboxState';

const appIcon: AppIcon = {
  kind: 'url',
  iconUrl: '/icon.svg',
};

function emptyTextForTab(tab: SandboxTab) {
  switch (tab) {
    case 'effective':
      return 'No effective sandbox gate state is available yet.';
    case 'previous':
      return 'No completed sandbox test run is available yet.';
    case 'current':
      return 'No sandbox test run is currently active.';
  }
}

export function App() {
  const [state, setState] = useState<SandboxStateView | null>(null);
  const [activeTab, setActiveTab] = useState<SandboxTab>('effective');
  const [loaded, setLoaded] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const currentRunning = isCurrentSandboxRunActive(state);

  const selectedState = useMemo(
    () => selectedSandboxState(state, activeTab),
    [state, activeTab],
  );

  useEffect(() => {
    let disposed = false;

    async function load() {
      try {
        const next = await getSandboxState();

        if (!disposed) {
          setState(next);
        }
      } catch (err) {
        if (!disposed) {
          setError(formatSageError(err));
        }
      } finally {
        if (!disposed) {
          setLoaded(true);
        }
      }
    }

    void load();

    const off = onSandboxStateChanged((next) => {
      setState(next);

      const stillRunning = isCurrentSandboxRunActive(next);

      if (!stillRunning) {
        setRunning(false);

        setActiveTab((prev) => (prev === 'current' ? 'previous' : prev));
      }
    });

    return () => {
      disposed = true;
      off();
    };
  }, []);

  async function rerun() {
    setRunning(true);
    setError(null);
    setActiveTab('current');

    try {
      const next = await rerunSandboxTests();
      setState(next);

      if (!isCurrentSandboxRunActive(next)) {
        setRunning(false);
        setActiveTab('previous');
      }
    } catch (err) {
      setRunning(false);
      setError(formatSageError(err));
    }
  }

  if (!loaded) {
    return (
      <AppModalShell
        title='Sandbox tests'
        appName='Sandbox tests'
        appIcon={null}
      >
        <div className='text-sm text-muted-foreground'>
          Loading sandbox tests…
        </div>
      </AppModalShell>
    );
  }

  return (
    <AppModalShell
      title='Sandbox tests'
      appName='Sandbox tests'
      appIcon={appIcon}
      footer={
        <div className='flex justify-end gap-2'>
          <button
            type='button'
            disabled={running}
            onClick={() => void closeSelf()}
            className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50'
          >
            Close
          </button>

          <button
            type='button'
            disabled={running}
            onClick={() => void rerun()}
            className='rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50'
          >
            {running ? 'Running…' : 'Re-run tests'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <SandboxTabs
          activeTab={activeTab}
          currentEnabled={currentRunning}
          onChange={setActiveTab}
        />

        {error ? (
          <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-sm text-destructive'>
            {error}
          </div>
        ) : null}

        <SandboxResultList
          state={selectedState}
          emptyText={emptyTextForTab(activeTab)}
        />
      </div>
    </AppModalShell>
  );
}
