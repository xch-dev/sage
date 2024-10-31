import invalid from '@/images/invalid.png';
import missing from '@/images/missing.png';

export function nftUri(mimeType: string | null, data: string | null): string {
  if (data === null || mimeType === null) return missing;

  if (
    ![
      'image/png',
      'image/jpeg',
      'image/gif',
      'image/webp',
      'image/svg+xml',
    ].includes(mimeType)
  )
    return invalid;

  return `data:${mimeType};base64,${data}`;
}
