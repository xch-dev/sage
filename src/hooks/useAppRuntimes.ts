import { useEffect, useState } from 'react';
import {
  listAppRuntimes,
  subscribeAppRuntimes,
  type SageAppRuntimeRecordView,
} from '@/lib/apps/runtimeRegistry';

export function useAppRuntimes(options?: { includeInternal?: boolean }) {
  const includeInternal = options?.includeInternal ?? false;

  const [runtimes, setRuntimes] = useState<SageAppRuntimeRecordView[]>(() => {
    const all = listAppRuntimes();
    return includeInternal ? all : all.filter((runtime) => !runtime.internal);
  });

  useEffect(() => {
    return subscribeAppRuntimes((next: SageAppRuntimeRecordView[]) => {
      setRuntimes(
        includeInternal ? next : next.filter((runtime) => !runtime.internal),
      );
    });
  }, [includeInternal]);

  return runtimes;
}
