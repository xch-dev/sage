import { z } from 'zod';

const NAMESDAO_API_BASE = 'https://namesdaolookup.xchstorage.com';
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes in milliseconds

const nameInfoSchema = z.object({
  address: z.string(),
  uris: z.array(z.string()).optional(),
  meta_uris: z.array(z.string()).optional(),
  nft_coin_id: z.string().optional(),
});

type NameInfo = z.infer<typeof nameInfoSchema>;

interface CacheEntry {
  address: string;
  timestamp: number;
}

const nameCache = new Map<string, CacheEntry>();

/**
 * Validates if a string is a potentially valid .xch name
 * Only checks if it ends with .xch (case-insensitive)
 */
export function isValidXchName(name: string): boolean {
  if (!name) return false;
  return /\.xch$/i.test(name.trim());
}

/**
 * Formats an input string to ensure proper .xch suffix
 * Converts to lowercase and adds .xch if missing
 */
export function formatXchName(name: string): string {
  const normalized = name.trim().toLowerCase();
  if (normalized.endsWith('.xch')) return normalized;
  return `${normalized}.xch`;
}

/**
 * Resolves a .xch name to its corresponding address
 * Returns null if name is invalid or resolution fails
 * Implements caching to reduce API calls
 */
export async function resolveXchName(name: string): Promise<string | null> {
  try {
    const trimmedName = name.trim();
    if (!isValidXchName(trimmedName)) return null;
    
    const normalized = formatXchName(trimmedName);
    const cached = nameCache.get(normalized);
    
    // Return cached value if still valid
    if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
      return cached.address;
    }
    
    // Remove .xch for API call
    const baseName = normalized.replace('.xch', '');
    const response = await fetch(`${NAMESDAO_API_BASE}/${baseName}.json`);
    
    if (!response.ok) {
      if (response.status === 404) return null;
      throw new Error(`Failed to resolve name: ${response.statusText}`);
    }
    
    const data = await response.json();
    const parsed = nameInfoSchema.parse(data);
    
    // Cache the result
    nameCache.set(normalized, {
      address: parsed.address,
      timestamp: Date.now(),
    });
    
    return parsed.address;
  } catch (error) {
    console.error('Error resolving .xch name:', error);
    return null;
  }
}
