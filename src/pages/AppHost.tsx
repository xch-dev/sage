import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useApps } from '@/contexts/AppsContext';
import { useAppEmbeddedRuntime } from '@/hooks/useAppEmbeddedRuntime.ts';
import { formatSandboxLaunchDecision } from '@/lib/apps/sandboxPolicy';

function AppNotFound() {
  return (
    <div className='mx-auto w-full max-w-4xl p-4 md:p-6'>
      <Alert>
        <AlertTitle>App not found</AlertTitle>
        <AlertDescription>This app does not exist.</AlertDescription>
      </Alert>
    </div>
  );
}

function AppBlocked({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <div className='mx-auto w-full max-w-4xl p-4 md:p-6'>
      <Alert>
        <AlertTitle>{title}</AlertTitle>
        <AlertDescription>{description}</AlertDescription>
      </Alert>
    </div>
  );
}

export function AppHost() {
  const { appId = '' } = useParams();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const { getListedApp, getLaunchGate, loading } = useApps();

  const app = getListedApp(appId);

  const userLaunchDecision =
    app?.kind === 'user'
      ? formatSandboxLaunchDecision(getLaunchGate(app.common.identity.id))
      : null;

  const routeableSystemApp =
    app?.kind === 'system' && app.presentation === 'Taskbar';

  const shouldMountRuntime =
    !!app &&
    (app.kind === 'system'
      ? routeableSystemApp
      : !!userLaunchDecision?.allowed);

  const { attaching, attachError } = useAppEmbeddedRuntime({
    app: shouldMountRuntime ? app : null,
    containerRef,
  });

  if (loading) {
    return (
      <div className='mx-auto w-full max-w-4xl p-4 md:p-6'>
        <Alert>
          <AlertTitle>Loading app...</AlertTitle>
          <AlertDescription>Please wait.</AlertDescription>
        </Alert>
      </div>
    );
  }

  if (!app) {
    return <AppNotFound />;
  }

  if (app.kind === 'system' && app.presentation !== 'Taskbar') {
    return (
      <AppBlocked
        title='System app is not routeable'
        description='This system app is opened contextually by Sage and is not available through direct navigation.'
      />
    );
  }

  if (app.kind === 'user' && !userLaunchDecision?.allowed) {
    return (
      <AppBlocked
        title={userLaunchDecision?.title ?? 'App launch blocked'}
        description={
          userLaunchDecision?.description ??
          'This app cannot be launched until required sandbox checks pass.'
        }
      />
    );
  }

  return (
    <div className='flex h-full min-h-0 w-full flex-col overflow-hidden'>
      {attachError ? (
        <AppBlocked title='Failed to launch app' description={attachError} />
      ) : attaching ? (
        <div className='mx-auto w-full max-w-4xl p-4 md:p-6'>
          <Alert>
            <AlertTitle>Starting app...</AlertTitle>
            <AlertDescription>Please wait.</AlertDescription>
          </Alert>
        </div>
      ) : null}

      <div className='flex-1 min-h-0'>
        <div
          ref={containerRef}
          className='h-full w-full overflow-hidden bg-background'
        />
      </div>
    </div>
  );
}
