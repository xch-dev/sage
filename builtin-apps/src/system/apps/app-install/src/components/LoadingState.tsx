import { AppModalShell } from '@sage-app/ui';
import { INSTALL_APP_ICON } from '../constants';

export function LoadingState() {
  return (
    <AppModalShell
      appName='Sage'
      appIcon={INSTALL_APP_ICON}
      title='Install app'
    >
      <div className='text-sm text-muted-foreground'>Loading installer…</div>
    </AppModalShell>
  );
}
