import { OptionRecord, commands } from '@/bindings';
import { useEffect, useState } from 'react';

/** An option from this wallet. */
export function useOwnedOption(optionId: string) {
  const [option, setOption] = useState<OptionRecord | null>(null);

  useEffect(() => {
    if (!optionId) return;
    let cancelled = false;

    commands
      .getOption({ option_id: optionId })
      .then((response) => {
        if (!cancelled) setOption(response.option);
      })
      .catch(() => {
        if (!cancelled) setOption(null);
      });

    return () => {
      cancelled = true;
    };
  }, [optionId]);

  return option?.launcher_id === optionId ? option : null;
}
