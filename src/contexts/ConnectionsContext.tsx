import { commands, type ConnectionsView } from '@/bindings';
import { OperationDialog } from '@/components/connections/OperationDialog';
import { PairingDialog } from '@/components/connections/PairingDialog';
import { platform } from '@tauri-apps/plugin-os';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';

interface ConnectionsContextValue {
  view: ConnectionsView | null;
  error: string | null;
  refresh: () => Promise<void>;
  review: (id: string) => void;
  setManaging: (open: boolean) => void;
}
const ConnectionsContext = createContext<ConnectionsContextValue | null>(null);
export const useConnections = () => useContext(ConnectionsContext);

export function ConnectionsProvider({ children }: { children: ReactNode }) {
  const desktop = !['ios', 'android'].includes(platform());
  const [view, setView] = useState<ConnectionsView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [managing, setManaging] = useState(false);
  const generation = useRef(0);
  const refresh = useCallback(async () => {
    if (!desktop) return;
    const id = ++generation.current;
    try {
      const result = await commands.connectionsReview();
      if (id !== generation.current) return;
      setView(result);
      setError(result.error);
      setDismissed((ids) =>
        ids.filter((id) =>
          [...result.pending, ...result.operations].some((p) => p.id === id),
        ),
      );
    } catch {
      if (id === generation.current)
        setError(
          'Connections could not be refreshed. Try again before changing access.',
        );
    }
  }, [desktop]);
  const invalidate = useCallback(() => {
    ++generation.current;
  }, []);
  useEffect(() => {
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      await refresh();
      if (!stopped) timer = setTimeout(() => void poll(), 2500);
    };
    void poll();
    return () => {
      stopped = true;
      invalidate();
      clearTimeout(timer);
    };
  }, [refresh, invalidate]);
  const selectedPairing = view?.pending.find((p) => p.id === selected);
  const selectedOperation = view?.operations.find((p) => p.id === selected);
  const pending = selectedOperation
    ? undefined
    : (selectedPairing ?? view?.pending.find((p) => !dismissed.includes(p.id)));
  const operation = pending
    ? undefined
    : (selectedOperation ??
      view?.operations.find((p) => !dismissed.includes(p.id)));
  return (
    <ConnectionsContext.Provider
      value={{ view, error, refresh, review: setSelected, setManaging }}
    >
      {children}
      {desktop && !managing && operation && (
        <OperationDialog
          key={operation.id}
          request={operation}
          unavailable={!!error}
          onClose={() => {
            setDismissed((ids) => [...ids, operation.id]);
            setSelected(null);
          }}
          onChanged={refresh}
        />
      )}
      {desktop && !managing && pending && view && (
        <PairingDialog
          key={pending.id}
          request={pending}
          capabilities={view.capabilities}
          unavailable={!!error}
          onClose={() => {
            setDismissed((ids) => [...ids, pending.id]);
            setSelected(null);
          }}
          onChanged={refresh}
        />
      )}
    </ConnectionsContext.Provider>
  );
}
