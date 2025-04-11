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
const audioTypes = [
  'audio/webm',
  'audio/mp3',
  'audio/mpeg',
  'audio/wav',
  'audio/ogg',
  'audio/mp4',
];
const textTypes = [
  'text/plain',
  'text/html',
  'text/css',
  'text/javascript',
  'text/markdown',
];
const jsonTypes = ['application/json', 'application/ld+json'];

/**
 * Extracts the base MIME type from a potentially parameterized MIME type string
 * For example: "text/plain; charset=utf-8" -> "text/plain"
 */
export function getBaseMimeType(mimeType: string | null): string | null {
  if (!mimeType) return null;
  return mimeType.split(';')[0].trim();
}

export function nftUri(mimeType: string | null, data: string | null): string {
  if (data === null || mimeType === null) return missing;

  const baseMimeType = getBaseMimeType(mimeType);
  if (
    !baseMimeType ||
    !imageTypes
      .concat(videoTypes, audioTypes, textTypes, jsonTypes)
      .includes(baseMimeType)
  )
    return invalid;

  return `data:${mimeType};base64,${data}`;
}

export function isImage(mimeType: string | null): boolean {
  const baseMimeType = getBaseMimeType(mimeType);
  return baseMimeType !== null && imageTypes.includes(baseMimeType);
}

export function isVideo(mimeType: string | null): boolean {
  const baseMimeType = getBaseMimeType(mimeType);
  return baseMimeType !== null && videoTypes.includes(baseMimeType);
}

export function isAudio(mimeType: string | null): boolean {
  const baseMimeType = getBaseMimeType(mimeType);
  return baseMimeType !== null && audioTypes.includes(baseMimeType);
}

export function isText(mimeType: string | null): boolean {
  const baseMimeType = getBaseMimeType(mimeType);
  return baseMimeType !== null && textTypes.includes(baseMimeType);
}

export function isJson(mimeType: string | null): boolean {
  const baseMimeType = getBaseMimeType(mimeType);
  return baseMimeType !== null && jsonTypes.includes(baseMimeType);
}
