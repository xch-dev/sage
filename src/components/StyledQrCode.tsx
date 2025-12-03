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

  const updateImageSrc = useCallback(async () => {
    if (!qrCode.current || !ref.current) return;

    try {
      const rawData = await qrCode.current.getRawData('svg');
      if (rawData instanceof Blob) {
        const dataUrl = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onloadend = () => {
            if (ref.current) {
              resolve(reader.result as string);
            } else {
              reject(new Error('Component unmounted'));
            }
          };
          reader.onerror = reject;
          reader.readAsDataURL(rawData);
        });
        if (ref.current) {
          ref.current.src = dataUrl;
        }
      }
    } catch (error) {
      console.error('Failed to get QR code data:', error);
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
  ]);

  useEffect(() => {
    if (!qrCode.current) return;

    const updateOptions: Partial<Options> = {
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

    qrCode.current.update(updateOptions);
    updateImageSrc();
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
  ]);

  return (
    <img ref={ref} className={`w-full h-full ${className}`} alt={t`QR Code`} />
  );
};

export default StyledQRCode;
