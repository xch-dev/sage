import { CopyBox } from './CopyBox';
import { formatAddress } from '@/lib/utils';
import { useEffect, useRef, useState } from 'react';
import { t } from '@lingui/core/macro';

function getTextWidth(text: string, element: HTMLElement | null) {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context || !element) return 0;

  context.font = getComputedStyle(element).font;
  return context.measureText(text).width;
}

export function FormattedAddressBox({
  className,
  minChars = 4,
  maxChars = 62,
  address,
  title,
}: {
  className?: string;
  minChars?: number;
  maxChars?: number;
  address: string;
  title?: string;
}) {
  const ref = useRef<HTMLInputElement>(null);
  const [chars, setChars] = useState(8);

  useEffect(() => {
    if (!ref.current) return;

    const updateChars = () => {
      const containerWidth = ref.current?.clientWidth ?? 0;
      const dotsWidth = getTextWidth('...', ref.current) + 12;
      const fullWidth = getTextWidth(address, ref.current);

      if (containerWidth >= fullWidth) {
        setChars(address.length);
        return;
      }

      const charWidth = fullWidth / address.length;
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
  }, [address, minChars, maxChars]);

  return (
    <CopyBox
      inputRef={ref}
      truncate={false}
      title={title ?? t`Address`}
      className={className}
      value={address}
      displayValue={formatAddress(address, chars / 2)}
    />
  );
} 