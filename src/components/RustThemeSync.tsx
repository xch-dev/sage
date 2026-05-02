import { useEffect } from 'react';
import { useTheme } from 'theme-o-rama';
import { publishEnvironmentThemeToRust } from '@/lib/apps/environmentTheme';

export function RustThemeSync() {
  const { currentTheme } = useTheme();

  useEffect(() => {
    if (!currentTheme) {
      return;
    }

    const frame = requestAnimationFrame(() => {
      void publishEnvironmentThemeToRust(currentTheme).catch((err) => {
        console.error('Failed to publish current theme to Rust:', err);
      });
    });

    return () => {
      cancelAnimationFrame(frame);
    };
  }, [currentTheme]);

  return null;
}
