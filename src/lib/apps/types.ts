import type { SageAppView, SystemSageAppView, UserSageAppView } from '@/bindings';

export type UserAppView = Extract<SageAppView, { kind: 'user' }>;
export type SystemAppView = Extract<SageAppView, { kind: 'system' }>;
export type RouteableApp = UserAppView | SystemAppView;

export function asUserApp(app: UserSageAppView): UserAppView {
  return {
    kind: 'user',
    ...app,
  };
}

export function asSystemApp(app: SystemSageAppView): SystemAppView {
  return {
    kind: 'system',
    ...app,
  };
}

export function canRouteToApp(
  app: SageAppView | UserSageAppView | SystemSageAppView,
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
  app: SageAppView | UserSageAppView | SystemSageAppView,
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
