import type { SageClient } from '../types';

let started = false;

export function bootstrapTheme(client: SageClient) {
  if (started) return;
  started = true;

  void client.environment.theme.mountCssVars().catch((err) => {
    console.debug('[Sage SDK] Theme CSS vars not mounted:', err);
  });

  try {
    client.environment.theme.onChanged?.(() => {
      void client.environment.theme.mountCssVars().catch((err) => {
        console.debug('[Sage SDK] Theme CSS vars refresh failed:', err);
      });
    });
  } catch (err) {
    console.debug('[Sage SDK] Theme change listener not available:', err);
  }
}
