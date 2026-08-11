import { t } from '@lingui/core/macro';
import QRCodeStyling, { Options } from 'qr-code-styling';
import React, { useCallback, useEffect, useRef } from 'react';

// Make all properties optional and add React-specific props
type StyledQRCodeProps = Partial<Options> & {
  className?: string;
};

const StyledQRCode: React.FC<StyledQRCodeProps> = ({
  className = '',
  type = 'svg',
  shape = 'square',
  width = 300,
  height = 300,
  margin = 10,
  data = 'https://example.com',
  qrOptions = {
    typeNumber: 0,
    errorCorrectionLevel: 'H',
    mode: 'Byte',
  },
  imageOptions = {
    saveAsBlob: true,
    hideBackgroundDots: true,
    imageSize: 0.4,
    margin: 10,
  },
  dotsOptions = {
    type: 'rounded',
    color: '#222222',
    roundSize: true,
  },
  backgroundOptions = {
    round: 0,
    color: '#ffffff',
  },
  ...rest
}) => {
  const ref = useRef<HTMLImageElement>(null);
  const qrCode = useRef<QRCodeStyling | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const updateImageSrc = useCallback(async () => {
    if (!qrCode.current || !ref.current) return;

    // Cancel any pending operations
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();
    const signal = abortControllerRef.current.signal;

    try {
      const rawData = await qrCode.current.getRawData('svg');
      if (signal.aborted || !ref.current) return;

      if (rawData instanceof Blob) {
        const dataUrl = await new Promise<string>((resolve, reject) => {
          if (signal.aborted) {
            reject(new Error('Aborted'));
            return;
          }

          const reader = new FileReader();
          reader.onloadend = () => {
            if (signal.aborted || !ref.current) {
              reject(new Error('Component unmounted or aborted'));
            } else {
              resolve(reader.result as string);
            }
          };
          reader.onerror = reject;
          reader.readAsDataURL(rawData);
        });

        if (!signal.aborted && ref.current) {
          ref.current.src = dataUrl;
        }
      }
    } catch (error) {
      if (
        error instanceof Error &&
        error.message !== 'Aborted' &&
        error.message !== 'Component unmounted or aborted'
      ) {
        console.error('Failed to get QR code data:', error);
      }
    }
  }, []);

  useEffect(() => {
    if (!ref.current) return;

    const options: Options = {
      type,
      shape,
      width,
      height,
      margin,
      data,
      qrOptions,
      imageOptions,
      dotsOptions,
      backgroundOptions,
      ...rest,
    };

    qrCode.current = new QRCodeStyling(options);
    updateImageSrc();

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [
    type,
    shape,
    width,
    height,
    margin,
    data,
    qrOptions,
    imageOptions,
    dotsOptions,
    backgroundOptions,
    rest,
    updateImageSrc,
    // Note: rest is intentionally omitted from dependencies to avoid infinite loops
    // The spread operator in the options object will still include rest properties
  ]);

  return (
    <img ref={ref} className={`w-full h-full ${className}`} alt={t`QR Code`} />
  );
};

export default StyledQRCode;
