import type { AppIcon } from '@sage-app/ui';
import type { InstallSource } from '../types';

export function resolveInstallIcon(source: InstallSource): AppIcon | null {
  if (source.kind === 'url') {
    if (source.preview.icon === null) {
      return null;
    }

    return {
      kind: 'bytes',
      icon: source.preview.icon,
    };
  }

  return null;
}
