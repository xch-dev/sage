import { AppModalShell } from '@sage-app/ui';
import { INSTALL_APP_ICON } from '../constants';

export function ErrorState({ error }: { error: string }) {
  return (
    <AppModalShell
      appName='Sage'
      appIcon={INSTALL_APP_ICON}
      title='Install app failed'
    >
      <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
        {error}
      </div>
    </AppModalShell>
  );
}
