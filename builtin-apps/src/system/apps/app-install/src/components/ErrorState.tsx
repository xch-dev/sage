import { AppModalShell } from 'sage-app-ui';

export function ErrorState({ error }: { error: string }) {
  return (
    <AppModalShell appName='Sage' title='Install app failed'>
      <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
        {error}
      </div>
    </AppModalShell>
  );
}
