import { ListedSageApp } from '@/bindings.ts';

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

export function AppIcon({ app }: { app: ListedSageApp }) {
  const name = app.kind === 'corrupted' ? app.id : app.common.name;

  const iconUrl =
    app.kind === 'corrupted' || !app.common.iconFile
      ? null
      : app.kind === 'system'
        ? `sage-system-app://${app.common.originId}/${app.common.iconFile}`
        : `sage-app://${app.common.originId}/${app.common.iconFile}`;

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
