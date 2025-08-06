import { useCallback } from 'react';
import { mintGardenService } from '@/lib/mintGardenService';
import type { MintGardenServiceConfig } from '@/lib/mintGardenService';

export function useMintGardenConfig() {
  const getConfig = useCallback(() => {
    return mintGardenService.getConfig();
  }, []);

  const updateConfig = useCallback(
    (newConfig: Partial<MintGardenServiceConfig>) => {
      mintGardenService.updateConfig(newConfig);
    },
    [],
  );

  const clearCache = useCallback(async () => {
    await mintGardenService.clearCache();
  }, []);

  const clearExpiredCache = useCallback(async () => {
    await mintGardenService.clearExpiredCache();
  }, []);

  return {
    getConfig,
    updateConfig,
    clearCache,
    clearExpiredCache,
  };
}
