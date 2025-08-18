import { mintGardenService } from './mintGardenService';

// MintGarden API rate limiting configuration
export const MINTGARDEN_CONFIG = {
  // Delay between API requests in milliseconds
  // Increase this value if you're getting 429 errors
  DELAY_BETWEEN_REQUESTS: 1000, // 1 second

  // How long to cache profile data in milliseconds
  // 15 minutes = 15 * 60 * 1000
  CACHE_DURATION: 15 * 60 * 1000,

  // Maximum number of concurrent requests
  // Lower this value if you're getting 429 errors
  MAX_CONCURRENT_REQUESTS: 3,
} as const;

// Initialize the service with the configuration
export function initializeMintGardenService(): void {
  mintGardenService.updateConfig({
    delayBetweenRequests: MINTGARDEN_CONFIG.DELAY_BETWEEN_REQUESTS,
    cacheDuration: MINTGARDEN_CONFIG.CACHE_DURATION,
    maxConcurrentRequests: MINTGARDEN_CONFIG.MAX_CONCURRENT_REQUESTS,
  });
}

// Export the service for direct access if needed
export { mintGardenService };
