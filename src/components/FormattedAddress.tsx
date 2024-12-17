import { useEffect, useRef, useState } from 'react';

function getTextWidth(text: string, element: HTMLElement | null) {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context || !element) return 0;

  context.font = getComputedStyle(element).font;
  return context.measureText(text).width;
}

export function FormattedAddress({
  address,
  className = '',
  minChars = 4,
  maxChars = 62,
}: {
  address: string;
  className?: string;
  minChars?: number;
  maxChars?: number;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const [chars, setChars] = useState(8);

  useEffect(() => {
    if (!ref.current) return;

    const updateChars = () => {
      const element = ref.current;
      if (!element) return;

      const parentWidth = element.parentElement?.clientWidth ?? 0;
      if (parentWidth === 0) return;

      const dotsWidth = getTextWidth('...', element);
      const fullWidth = getTextWidth(address, element);

      if (parentWidth >= fullWidth) {
        setChars(address.length);
        return;
      }

      const charWidth = fullWidth / address.length;
      const availableChars = Math.floor((parentWidth - dotsWidth) / charWidth);
      const newChars = Math.max(
        minChars * 2,
        Math.min(maxChars, availableChars),
      );

      setChars(newChars);
    };

    const observer = new ResizeObserver(updateChars);
    observer.observe(ref.current);
    observer.observe(ref.current.parentElement as Element);

    return () => observer.disconnect();
  }, [address, minChars, maxChars]);

  const formatAddress = (address: string, chars: number) => {
    if (address.length <= chars * 2) return address;
    return `${address.slice(0, chars)}...${address.slice(-chars)}`;
  };

  return (
    <span ref={ref} className={`font-mono ${className}`}>
      {formatAddress(address, chars / 2)}
    </span>
  );
}
