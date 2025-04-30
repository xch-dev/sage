import React, { useEffect, useRef } from 'react';
import QRCodeStyling, { Options } from 'qr-code-styling';

// Make all properties optional and add React-specific props
type StyledQRCodeProps = Partial<Options> & {
  className?: string;
  download?: boolean;
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
  const ref = useRef<HTMLDivElement>(null);
  const qrCode = useRef<QRCodeStyling | null>(null);

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
    qrCode.current.append(ref.current);

    const currentRef = ref.current;
    return () => {
      if (currentRef) {
        currentRef.innerHTML = '';
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
  ]);

  useEffect(() => {
    if (download && qrCode.current) {
      qrCode.current.download({
        extension: type === 'svg' ? 'svg' : 'png',
        name: 'qr-code',
      });
    }
  }, [download, type]);

  return <div ref={ref} className={`w-full h-full ${className}`} />;
};

export default StyledQRCode;
