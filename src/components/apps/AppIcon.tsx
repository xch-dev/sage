import { ListedSageAppView } from '@/bindings.ts';

export function AppIconContent({
  name,
  iconUrl,
}: {
  name: string;
  iconUrl: string | null;
}) {
  if (iconUrl) {
    return <img src={iconUrl} alt='' />;
  }

  return <>{name.trim().charAt(0).toUpperCase() || 'A'}</>;
}

export function AppIcon({ app }: { app: ListedSageAppView }) {
  const name =
    app.kind === 'corrupted' ? app.id : app.common.activeSnapshot.manifest.name;

  const iconUrl =
    app.kind === 'corrupted' || !app.common.activeSnapshot.manifest.icon
      ? null
      : app.kind === 'system'
        ? `sage-system-app://${app.common.identity.originId}/${app.common.activeSnapshot.manifest.icon}`
        : `sage-app://${app.common.identity.originId}/${app.common.activeSnapshot.manifest.icon}`;

  return <AppIconContent name={name} iconUrl={iconUrl} />;
}

export function InstallAppIcon({
  name,
  iconUrl,
}: {
  name: string;
  iconUrl: string | null;
}) {
  return <AppIconContent name={name} iconUrl={iconUrl} />;
}
