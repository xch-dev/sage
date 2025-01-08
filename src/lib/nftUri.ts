import invalid from '@/images/invalid.png';
import missing from '@/images/missing.png';

const imageTypes = [
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
  'image/svg+xml',
];

const videoTypes = ['video/webm', 'video/mp4'];

export function nftUri(mimeType: string | null, data: string | null): string {
  if (data === null || mimeType === null) return missing;

  if (!imageTypes.concat(videoTypes).includes(mimeType)) return invalid;

  return `data:${mimeType};base64,${data}`;
}

export function isImage(mimeType: string | null): boolean {
  return mimeType !== null && imageTypes.includes(mimeType);
}

export function isVideo(mimeType: string | null): boolean {
  return mimeType !== null && videoTypes.includes(mimeType);
}
