import { AppModalShell } from 'sage-app-ui';

export function LoadingState() {
  return (
    <AppModalShell appName='Sage' title='Install app'>
      <div className='text-sm text-muted-foreground'>Loading installer…</div>
    </AppModalShell>
  );
}
