import {
  addCustomSource,
  migrateLegacyTokenIndexUrl,
  moveSource,
  normalizePriceSources,
  PriceSourceSetting,
} from '@/lib/priceSources';
import { useCallback, useEffect, useMemo } from 'react';
import { useLocalStorage } from 'usehooks-ts';

const LEGACY_TOKEN_INDEX_URL_KEY = 'tokenIndexUrl';

// The price and token sources, in priority order. Nothing is stored until the
// user changes something, so the defaults apply until then.
export function usePriceSources() {
  const [stored, setStored] = useLocalStorage<unknown>('priceSources', null);

  const sources = useMemo(() => normalizePriceSources(stored), [stored]);

  const update = useCallback(
    (change: (sources: PriceSourceSetting[]) => PriceSourceSetting[]) =>
      setStored((previous: unknown) => change(normalizePriceSources(previous))),
    [setStored],
  );

  // Read once the single URL an earlier build of this setting stored.
  useEffect(() => {
    try {
      const legacy = localStorage.getItem(LEGACY_TOKEN_INDEX_URL_KEY);
      if (legacy === null) return;
      localStorage.removeItem(LEGACY_TOKEN_INDEX_URL_KEY);
      update((current) => migrateLegacyTokenIndexUrl(current, legacy));
    } catch {
      // Storage unavailable: nothing to migrate.
    }
  }, [update]);

  return {
    sources,
    setEnabled: (id: string, enabled: boolean) =>
      update((current) =>
        current.map((source) =>
          source.id === id ? { ...source, enabled } : source,
        ),
      ),
    move: (id: string, direction: -1 | 1) =>
      update((current) => moveSource(current, id, direction)),
    remove: (id: string) =>
      update((current) =>
        current.filter((source) => source.id !== id || !source.url),
      ),
    /** Adds a custom token index URL; false if it is not a valid https URL. */
    addCustom: (url: string) => {
      if (!addCustomSource(sources, url)) return false;
      update((current) => addCustomSource(current, url) ?? current);
      return true;
    },
  };
}
