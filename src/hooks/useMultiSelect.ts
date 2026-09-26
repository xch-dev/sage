import { SelectionState } from '@/components/SelectableCard';
import { useCallback, useEffect, useState } from 'react';

export function useMultiSelect(resetDeps: readonly unknown[] = []) {
  // eslint-disable-next-line react/hook-use-state
  const [multiSelect, setMultiSelectState] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);

  const setMultiSelect = useCallback((value: boolean) => {
    setMultiSelectState(value);
    setSelected([]);
  }, []);

  const toggle = useCallback((id: string, value: boolean) => {
    setSelected((prev) => {
      if (value) return prev.includes(id) ? prev : [...prev, id];
      return prev.filter((existing) => existing !== id);
    });
  }, []);

  const selectAll = useCallback((ids: string[]) => setSelected(ids), []);
  const clear = useCallback(() => setSelected([]), []);

  const selectionStateFor = useCallback(
    (id: string): SelectionState =>
      multiSelect
        ? [selected.includes(id), (value: boolean) => toggle(id, value)]
        : null,
    [multiSelect, selected, toggle],
  );

  // Serialize so callers can pass a fresh array literal each render.
  const resetKey = JSON.stringify(resetDeps);

  useEffect(() => {
    setMultiSelect(false);
  }, [resetKey, setMultiSelect]);

  return {
    multiSelect,
    setMultiSelect,
    selected,
    setSelected,
    toggle,
    selectionStateFor,
    selectAll,
    clear,
  };
}
