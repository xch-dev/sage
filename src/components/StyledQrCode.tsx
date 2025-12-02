import QRCodeStyling, { Options } from 'qr-code-styling';
import React, { useEffect, useMemo, useRef, useState } from 'react';

type StyledQRCodeProps = Partial<Options> & {
  className?: string;
  download?: boolean;
};

const blobToUrl = (blob: Blob | Buffer, mimeType: string): string => {
  if (blob instanceof Blob) {
    return URL.createObjectURL(blob);
  }

  const uint8Array = new Uint8Array(blob);
  const blobObj = new Blob([uint8Array], { type: mimeType });
  return URL.createObjectURL(blobObj);
};

const StyledQRCode: React.FC<StyledQRCodeProps> = ({
  className = '',
  download = false,
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
  const [imageSrc, setImageSrc] = useState<string>('');
  const qrCode = useRef<QRCodeStyling | null>(null);
  const imageSrcRef = useRef<string>('');
  const prevRestKeyRef = useRef<string>('');
  const prevQrOptionsKeyRef = useRef<string>('');
  const prevImageOptionsKeyRef = useRef<string>('');
  const prevDotsOptionsKeyRef = useRef<string>('');
  const prevBackgroundOptionsKeyRef = useRef<string>('');

  // Create stable keys for object props - only change when content actually changes
  const currentRestKey = JSON.stringify(rest);
  const restKey =
    prevRestKeyRef.current !== currentRestKey
      ? (prevRestKeyRef.current = currentRestKey)
      : prevRestKeyRef.current;

  const currentQrOptionsKey = JSON.stringify(qrOptions);
  const qrOptionsKey =
    prevQrOptionsKeyRef.current !== currentQrOptionsKey
      ? (prevQrOptionsKeyRef.current = currentQrOptionsKey)
      : prevQrOptionsKeyRef.current;

  const currentImageOptionsKey = JSON.stringify(imageOptions);
  const imageOptionsKey =
    prevImageOptionsKeyRef.current !== currentImageOptionsKey
      ? (prevImageOptionsKeyRef.current = currentImageOptionsKey)
      : prevImageOptionsKeyRef.current;

  const currentDotsOptionsKey = JSON.stringify(dotsOptions);
  const dotsOptionsKey =
    prevDotsOptionsKeyRef.current !== currentDotsOptionsKey
      ? (prevDotsOptionsKeyRef.current = currentDotsOptionsKey)
      : prevDotsOptionsKeyRef.current;

  const currentBackgroundOptionsKey = JSON.stringify(backgroundOptions);
  const backgroundOptionsKey =
    prevBackgroundOptionsKeyRef.current !== currentBackgroundOptionsKey
      ? (prevBackgroundOptionsKeyRef.current = currentBackgroundOptionsKey)
      : prevBackgroundOptionsKeyRef.current;

  const mimeType = useMemo(
    () => (type === 'svg' ? 'image/svg+xml' : 'image/png'),
    [type],
  );

  useEffect(() => {
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

    if (!qrCode.current) {
      qrCode.current = new QRCodeStyling(options);
    } else {
      qrCode.current.update(options);
    }

    let cancelled = false;
    qrCode.current.getRawData(type === 'svg' ? 'svg' : 'png').then((blob) => {
      if (blob && !cancelled) {
        if (imageSrcRef.current) {
          URL.revokeObjectURL(imageSrcRef.current);
        }
        const url = blobToUrl(blob, mimeType);
        imageSrcRef.current = url;
        setImageSrc(url);
      }
    });

    return () => {
      cancelled = true;
      if (imageSrcRef.current) {
        URL.revokeObjectURL(imageSrcRef.current);
        imageSrcRef.current = '';
      }
    };
    // Using stable keys (qrOptionsKey, imageOptionsKey, etc.) instead of the objects
    // themselves to prevent infinite loops when parent components pass new object references
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    type,
    shape,
    width,
    height,
    margin,
    data,
    qrOptionsKey,
    imageOptionsKey,
    dotsOptionsKey,
    backgroundOptionsKey,
    restKey,
    mimeType,
  ]);

  useEffect(() => {
    if (download && qrCode.current) {
      qrCode.current.download({
        extension: type === 'svg' ? 'svg' : 'png',
        name: 'qr-code',
      });
    }
  }, [download, type]);

  return (
    <img
      src={imageSrc}
      alt='QR Code'
      className={`w-full h-full ${className}`}
    />
  );
};

export default StyledQRCode;
