import { MintGardenProfile } from '@/hooks/useDidData';
import { Store, load } from '@tauri-apps/plugin-store';

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
  private store: Store | null = null;
  private pendingRequests = new Map<string, Promise<MintGardenProfile>>();
  private lastRequestTime = 0;
  private config: MintGardenServiceConfig;
  private isInitialized = false;

  constructor(config: Partial<MintGardenServiceConfig> = {}) {
    this.config = {
      delayBetweenRequests: 750, // 3/4 second default
      cacheDuration: 15 * 60 * 1000, // 15 minutes default
      maxConcurrentRequests: 3, // Allow 3 concurrent requests
      ...config,
    };

    try {
      this.initializeStore().then(() => {
        this.clearExpiredCache();
      });
    } catch (error) {
      console.warn('Failed to initialize MintGardenService:', error);
    }
  }

  private async initializeStore(): Promise<void> {
    if (this.isInitialized) return;

    try {
      this.store = await load('.mintgarden-profile-cache.dat');
    } catch (error) {
      console.warn('Failed to load MintGarden cache store:', error);
    } finally {
      this.isInitialized = true;
    }
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

  private async getCachedProfile(
    did: string,
  ): Promise<MintGardenProfile | null> {
    await this.initializeStore();

    if (!this.store) {
      return null;
    }

    try {
      const cached = await this.store.get<CacheEntry>(`profile:${did}`);

      if (cached && this.isCacheValid(cached)) {
        return cached.profile;
      }
    } catch (error) {
      console.warn('Failed to read from cache:', error);
    }

    return null;
  }

  private async setCachedProfile(
    did: string,
    profile: MintGardenProfile,
  ): Promise<void> {
    await this.initializeStore();

    if (!this.store) {
      return;
    }

    try {
      const entry: CacheEntry = {
        profile,
        timestamp: Date.now(),
      };
      await this.store.set(`profile:${did}`, entry);
      await this.store.save();
    } catch (error) {
      console.warn('Failed to write to cache:', error);
    }
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

  async loadProfiles(dids: string[]): Promise<void> {
    if (dids.length === 0) {
      return;
    }

    await this.clearExpiredCache();
    await this.ensureRateLimit();

    try {
      const response = await fetch('https://api.mintgarden.io/profiles_by_id', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(dids),
      });
      const data = await response.json();
      const profiles = data.map((profile: MintGardenProfile) => ({
        encoded_id: profile.encoded_id,
        name: profile.name,
        avatar_uri: profile.avatar_uri,
        is_unknown: false,
      }));

      // this bit goes through and fills in the profiles for the dids that are not in the response
      for (const did of dids) {
        const profile = profiles.find(
          (p: MintGardenProfile) => p.encoded_id === did,
        ) ?? {
          encoded_id: did,
          name: `${did.slice(9, 19)}...${did.slice(-4)}`,
          avatar_uri: null,
          is_unknown: true,
        };

        await this.setCachedProfile(did, profile);
      }
    } catch (error) {
      console.warn('Failed to load profiles:', error);
    }
  }

  async getProfile(did: string, cacheOnly = false): Promise<MintGardenProfile> {
    // Check cache first
    const cached = await this.getCachedProfile(did);
    if (cached) {
      return cached;
    }

    // the cache only flag is used to avoid making a request if the profile is not in the cache
    if (cacheOnly) {
      return {
        encoded_id: did,
        name: `${did.slice(9, 19)}...${did.slice(-4)}`,
        avatar_uri: null,
        is_unknown: true,
      };
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
    const requestPromise = this.fetchProfileFromAPI(did).then(
      async (profile) => {
        // Cache the result
        await this.setCachedProfile(did, profile);

        // Remove from pending requests
        this.pendingRequests.delete(did);

        return profile;
      },
    );

    // Add to pending requests
    this.pendingRequests.set(did, requestPromise);

    return requestPromise;
  }

  // Clear cache entries that have expired
  async clearExpiredCache(): Promise<void> {
    await this.initializeStore();

    if (!this.store) {
      return;
    }

    try {
      const keys = await this.store.keys();
      const profileKeys = keys.filter((key) => key.startsWith('profile:'));

      for (const key of profileKeys) {
        const entry = await this.store.get<CacheEntry>(key);
        if (entry && !this.isCacheValid(entry)) {
          await this.store.delete(key);
        }
      }

      await this.store.save();
    } catch (error) {
      console.warn('Failed to clear expired cache:', error);
    }
  }

  // Clear all cache
  async clearCache(): Promise<void> {
    await this.initializeStore();

    if (!this.store) {
      return;
    }

    try {
      const keys = await this.store.keys();
      const profileKeys = keys.filter((key) => key.startsWith('profile:'));

      for (const key of profileKeys) {
        await this.store.delete(key);
      }

      await this.store.save();
    } catch (error) {
      console.warn('Failed to clear cache:', error);
    }
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
  cacheOnly = false,
): Promise<MintGardenProfile> {
  return mintGardenService.getProfile(did, cacheOnly);
}

// Export the service class and configuration interface for advanced usage
export { MintGardenService };
export type { MintGardenServiceConfig };
