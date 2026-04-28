import type { SageApp, SystemSageApp, UserSageApp } from '@/bindings';

export type UserApp = Extract<SageApp, { kind: 'user' }>;
export type SystemApp = Extract<SageApp, { kind: 'system' }>;
export type RouteableApp = UserApp | SystemApp;

export function asUserApp(app: UserSageApp): UserApp {
  return {
    kind: 'user',
    ...app,
  };
}

export function asSystemApp(app: SystemSageApp): SystemApp {
  return {
    kind: 'system',
    ...app,
  };
}

export function canRouteToApp(
  app: SageApp | UserSageApp | SystemSageApp,
): boolean {
  if ('kind' in app) {
    if (app.kind === 'user') {
      return true;
    }

    return app.presentation === 'Taskbar';
  }

  if ('source' in app) {
    return true;
  }

  return app.presentation === 'Taskbar';
}

export function routeForApp(
  app: SageApp | UserSageApp | SystemSageApp,
): string | null {
  if ('kind' in app) {
    if (app.kind === 'user') {
      return `/apps/${app.common.identity.id}`;
    }

    return canRouteToApp(app) ? `/system-apps/${app.common.identity.id}` : null;
  }

  if ('source' in app) {
    return `/apps/${app.common.identity.id}`;
  }

  return canRouteToApp(app) ? `/system-apps/${app.common.identity.id}` : null;
}
