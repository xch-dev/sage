import { useWalletState } from '@/state';
import { CopyBox } from './CopyBox';
import { formatAddress } from '@/lib/utils';
import { useEffect, useRef, useState } from 'react';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

function getTextWidth(text: string, element: HTMLElement | null) {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context || !element) return 0;

  context.font = getComputedStyle(element).font;
  return context.measureText(text).width;
}

export function ReceiveAddress({
  className,
  minChars = 4,
  maxChars = 62,
}: {
  className?: string;
  minChars?: number;
  maxChars?: number;
}) {
  const { receive_address } = useWalletState().sync;
  const ref = useRef<HTMLInputElement>(null);
  const [chars, setChars] = useState(8);

  useEffect(() => {
    if (!ref.current) return;

    const updateChars = () => {
      const containerWidth = ref.current?.clientWidth ?? 0;
      const dotsWidth = getTextWidth('...', ref.current) + 12;
      const fullWidth = getTextWidth(receive_address, ref.current);

      if (containerWidth >= fullWidth) {
        setChars(receive_address.length);
        return;
      }

      const charWidth = fullWidth / receive_address.length;
      const availableChars = Math.floor(
        (containerWidth - dotsWidth) / charWidth,
      );
      const newChars = Math.max(
        minChars * 2,
        Math.min(maxChars, availableChars),
      );

      setChars(newChars);
    };

    const observer = new ResizeObserver(updateChars);
    observer.observe(ref.current);

    return () => observer.disconnect();
  }, [receive_address, minChars, maxChars]);

  if (!receive_address || receive_address === 'Unknown') {
    return <div className={className}>Connecting to wallet...</div>;
  }

  return (
    <CopyBox
      inputRef={ref}
      truncate={false}
      title={t`Receive Address`}
      className={className}
      value={receive_address}
      displayValue={formatAddress(receive_address, chars / 2)}
      onCopy={() => {
        toast.success(t`Address copied to clipboard`);
      }}
    />
  );
}
