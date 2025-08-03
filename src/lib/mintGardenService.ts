import { MintGardenProfile } from '@/components/Profile';

interface CacheEntry {
  profile: MintGardenProfile;
  timestamp: number;
}

interface MintGardenServiceConfig {
  delayBetweenRequests: number; // milliseconds
  cacheDuration: number; // milliseconds
  maxConcurrentRequests: number;
}

class MintGardenService {
  private cache = new Map<string, CacheEntry>();
  private pendingRequests = new Map<string, Promise<MintGardenProfile>>();
  private lastRequestTime = 0;
  private config: MintGardenServiceConfig;

  constructor(config: Partial<MintGardenServiceConfig> = {}) {
    this.config = {
      delayBetweenRequests: 1000, // 1 second default
      cacheDuration: 15 * 60 * 1000, // 15 minutes default
      maxConcurrentRequests: 3, // Allow 3 concurrent requests
      ...config,
    };
  }

  private async delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private async ensureRateLimit(): Promise<void> {
    const now = Date.now();
    const timeSinceLastRequest = now - this.lastRequestTime;

    if (timeSinceLastRequest < this.config.delayBetweenRequests) {
      const delayNeeded =
        this.config.delayBetweenRequests - timeSinceLastRequest;
      await this.delay(delayNeeded);
    }

    this.lastRequestTime = Date.now();
  }

  private isCacheValid(entry: CacheEntry): boolean {
    const now = Date.now();
    return now - entry.timestamp < this.config.cacheDuration;
  }

  private async fetchProfileFromAPI(did: string): Promise<MintGardenProfile> {
    await this.ensureRateLimit();

    try {
      const response = await fetch(`https://api.mintgarden.io/profile/${did}`);
      const data = await response.json();

      if (data?.detail === 'Unknown profile.') {
        return {
          encoded_id: did,
          name: `${did.slice(9, 19)}...${did.slice(-4)}`,
          avatar_uri: null,
          is_unknown: true,
        };
      }

      // always supply a name
      data.name = data.name || `${did.slice(9, 19)}...${did.slice(-4)}`;
      data.is_unknown = false;
      return data;
    } catch {
      return {
        encoded_id: did,
        name: `${did.slice(9, 19)}...${did.slice(-4)}`,
        avatar_uri: null,
        is_unknown: true,
      };
    }
  }

  async getProfile(did: string): Promise<MintGardenProfile> {
    // Check cache first
    const cached = this.cache.get(did);
    if (cached && this.isCacheValid(cached)) {
      return cached.profile;
    }

    // Check if there's already a pending request for this DID
    const pendingRequest = this.pendingRequests.get(did);
    if (pendingRequest) {
      return pendingRequest;
    }

    // Check concurrent request limit
    if (this.pendingRequests.size >= this.config.maxConcurrentRequests) {
      // Wait for any request to complete
      await Promise.race(this.pendingRequests.values());
      // Recursive call to try again
      return this.getProfile(did);
    }

    // Create new request
    const requestPromise = this.fetchProfileFromAPI(did).then((profile) => {
      // Cache the result
      this.cache.set(did, {
        profile,
        timestamp: Date.now(),
      });

      // Remove from pending requests
      this.pendingRequests.delete(did);

      return profile;
    });

    // Add to pending requests
    this.pendingRequests.set(did, requestPromise);

    return requestPromise;
  }

  // Clear cache entries that have expired
  clearExpiredCache(): void {
    for (const [key, entry] of this.cache.entries()) {
      if (!this.isCacheValid(entry)) {
        this.cache.delete(key);
      }
    }
  }

  // Clear all cache
  clearCache(): void {
    this.cache.clear();
  }

  // Update configuration
  updateConfig(newConfig: Partial<MintGardenServiceConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  // Get current configuration
  getConfig(): MintGardenServiceConfig {
    return { ...this.config };
  }
}

// Create a singleton instance
export const mintGardenService = new MintGardenService();

// Export the function that maintains backward compatibility
export async function getMintGardenProfile(
  did: string,
): Promise<MintGardenProfile> {
  return mintGardenService.getProfile(did);
}

// Export the service class and configuration interface for advanced usage
export { MintGardenService };
export type { MintGardenServiceConfig };
