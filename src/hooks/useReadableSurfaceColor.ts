import { getReadableSurfaceColor } from '@/lib/themeSurface';
import { useLayoutEffect, useState } from 'react';

/**
 * The readable surface color for the current theme, recomputed whenever the
 * theme is applied. theme-o-rama writes a theme as inline variables and a class
 * on the document root, so watching those attributes catches every theme
 * change after it lands, whenever and however the provider applies it.
 */
export function useReadableSurfaceColor(): string {
  const [color, setColor] = useState(getReadableSurfaceColor);

  useLayoutEffect(() => {
    const update = () => setColor(getReadableSurfaceColor());
    update();

    const observer = new MutationObserver(update);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['style', 'class'],
    });
    return () => observer.disconnect();
  }, []);

  return color;
}
