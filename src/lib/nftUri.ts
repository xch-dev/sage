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

const textTypes = ['text/plain'];

export function nftUri(mimeType: string | null, data: string | null): string {
  if (data === null || mimeType === null) return missing;

  if (textTypes.includes(mimeType) || isJson(mimeType)) {
    try {
      // Try to decode as base64 first
      const binaryStr = atob(data);
      // Convert binary string to Uint8Array for proper UTF-8 handling
      const bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }
      // Decode as UTF-8
      return new TextDecoder().decode(bytes);
    } catch {
      // If decoding fails, assume it's already plain text
      return data;
    }
  }

  if (!imageTypes.concat(videoTypes).includes(mimeType)) return invalid;

  return `data:${mimeType};base64,${data}`;
}

export function isImage(mimeType: string | null): boolean {
  return mimeType !== null && imageTypes.includes(mimeType);
}

export function isVideo(mimeType: string | null): boolean {
  return mimeType !== null && videoTypes.includes(mimeType);
}

export function isText(mimeType: string | null): boolean {
  return mimeType === 'text/plain';
}

export function isJson(mimeType: string | null): boolean {
  return mimeType === 'application/json' || mimeType?.endsWith('+json');
}
