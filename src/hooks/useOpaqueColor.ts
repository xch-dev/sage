import { makeColorOpaque } from '@/lib/color-utils';
import { useEffect, useState } from 'react';

/**
 * Hook that provides an opaque version of a CSS variable color.
 * Useful for ensuring elements have solid backgrounds regardless of theme transparency settings.
 */
export function useOpaqueColor(cssVariable: string): string {
  const [opaqueColor, setOpaqueColor] = useState<string>('');

  useEffect(() => {
    const updateOpaqueColor = () => {
      if (typeof document !== 'undefined') {
        const computedStyle = getComputedStyle(document.documentElement);
        const colorValue = computedStyle.getPropertyValue(cssVariable).trim();

        if (colorValue) {
          const opaque = makeColorOpaque(colorValue);
          setOpaqueColor(opaque);
        }
      }
    };

    // Update immediately
    updateOpaqueColor();

    // Listen for theme changes (this could be improved with a more specific event)
    const observer = new MutationObserver(() => {
      updateOpaqueColor();
    });

    if (typeof document !== 'undefined') {
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['style', 'class'],
      });
    }

    return () => observer.disconnect();
  }, [cssVariable]);

  return opaqueColor;
}

/**
 * Hook that provides opaque versions of common theme colors.
 * Returns an object with guaranteed opaque colors for backgrounds.
 */
export function useOpaqueThemeColors() {
  const opaqueBackground = useOpaqueColor('--background');
  const opaqueCard = useOpaqueColor('--card');
  const opaqueSecondary = useOpaqueColor('--secondary');
  const opaqueMuted = useOpaqueColor('--muted');

  return {
    background: opaqueBackground,
    card: opaqueCard,
    secondary: opaqueSecondary,
    muted: opaqueMuted,
  };
}
