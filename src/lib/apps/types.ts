import type { SageAppView, SystemSageAppView, UserSageAppView } from '@/bindings';

export function routeForApp(
  app: SageAppView | UserSageAppView | SystemSageAppView,
): string | null {
  if ('kind' in app) {
    if (app.kind === 'user') {
      return `/apps/${app.common.identity.id}`;
    }

    return `/system-apps/${app.common.identity.id}`;
  }

  if ('source' in app) {
    return `/apps/${app.common.identity.id}`;
  }

  return `/system-apps/${app.common.identity.id}`;
}
