import invalid from '@/invalid.png';
import missing from '@/missing.png';

export function nftUri(mimeType: string | null, data: string | null): string {
  if (data === null || mimeType === null) return missing;

  if (
    !['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(mimeType)
  )
    return invalid;

  return `data:${mimeType};base64,${data}`;
}
