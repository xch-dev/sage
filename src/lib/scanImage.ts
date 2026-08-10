import jsQR from 'jsqr';

export type ScanImageErrorCode = 'no-qr-found' | 'file-too-big' | 'unreadable';

/**
 * Carries a reason code only. Never put file content, decoded content, or the
 * filename in the message — this error is logged and displayed.
 */
export class ScanImageError extends Error {
  constructor(public readonly code: ScanImageErrorCode) {
    super(code);
    this.name = 'ScanImageError';
  }
}

const MAX_FILE_BYTES = 20 * 1024 * 1024;
const MAX_PIXELS = 40_000_000;
const MAX_EDGE = 1600;

export async function scanQrFromImage(file: File): Promise<string> {
  if (file.size > MAX_FILE_BYTES) {
    throw new ScanImageError('file-too-big');
  }

  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    throw new ScanImageError('unreadable');
  }

  try {
    // Must run before any canvas is allocated: a small PNG can declare
    // enormous dimensions, and the backing ImageData is 4 bytes per pixel.
    if (bitmap.width * bitmap.height > MAX_PIXELS) {
      throw new ScanImageError('file-too-big');
    }

    const scale = Math.min(
      1,
      MAX_EDGE / Math.max(bitmap.width, bitmap.height),
    );
    const width = Math.max(1, Math.round(bitmap.width * scale));
    const height = Math.max(1, Math.round(bitmap.height * scale));

    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;

    const context = canvas.getContext('2d');
    if (!context) {
      throw new ScanImageError('unreadable');
    }

    context.drawImage(bitmap, 0, 0, width, height);
    const imageData = context.getImageData(0, 0, width, height);

    // attemptBoth reads light-on-dark codes. It is jsQR's current default,
    // passed explicitly so a future default change cannot silently drop it.
    const result = jsQR(imageData.data, width, height, {
      inversionAttempts: 'attemptBoth',
    });

    if (!result) {
      throw new ScanImageError('no-qr-found');
    }

    return result.data;
  } finally {
    bitmap.close();
  }
}
