import { useMemo } from 'react';
import { ListedSageAppView } from '@/bindings.ts';

export function AppIconContent({
  name,
  iconUrl,
}: {
  name: string;
  iconUrl: string | null;
}) {
  if (iconUrl) {
    return (
      <img src={iconUrl} alt='' className='h-full w-full object-contain' />
    );
  }

  return <>{name.trim().charAt(0).toUpperCase() || 'A'}</>;
}

export function AppIcon({ app }: { app: ListedSageAppView }) {
  const name =
    app.kind === 'corrupted'
      ? (app.manifestHeader?.name ?? app.id)
      : app.common.activeSnapshot.manifest.name;

  const iconUrl = useMemo(() => {
    return iconUrlFromApp(app);
  }, [app]);

  return <AppIconContent name={name} iconUrl={iconUrl} />;
}

function iconUrlFromApp(app: ListedSageAppView): string | null {
  const icon = app.kind === 'corrupted' ? app.icon : app.common.icon;

  if (!icon) return null;

  return URL.createObjectURL(
    new Blob([new Uint8Array(icon.bytes)], { type: icon.mime }),
  );
}
