import { useEffect, useMemo } from 'react';
import { SageAppCommonView } from '@sage-system-app/sdk';

export type AppIcon =
  | { kind: 'url'; iconUrl: string }
  | { kind: 'bytes'; icon: AppIconBytes };

export type AppIconBytes = {
  bytes: number[];
  mime: string;
};

export function AppIcon({ appName, appIcon }: { appName: string, appIcon: AppIcon | null }) {
  if (!appIcon) {
    return <AppIconFallback name={appName} className='h-full w-full' />
  }
  if (appIcon.kind === 'url') {
    return <AppIconFromUrl
      name={appName}
      iconUrl={appIcon.iconUrl}
      className='h-full w-full'
    />
  }
  if (appIcon.kind === 'bytes') {
    return <AppIconFromBytes
      name={appName}
      icon={appIcon.icon}
      className='h-full w-full'
    />
  }

  return <AppIconFallback name={appName} className='h-full w-full' />;
}

export function AppIconFallback({ name, className }: { name: string; className?: string }) {
  return (
    <div className={['flex items-center justify-center', className].join(' ')}>
      {name.trim().charAt(0).toUpperCase() || 'A'}
    </div>
  );
}

export function AppIconFromUrl({
  name,
  iconUrl,
  className,
}: {
  name: string;
  iconUrl: string | null;
  className?: string;
}) {
  if (iconUrl) {
    return (
      <img
        src={iconUrl}
        alt=''
        className={['h-full w-full object-contain', className].join(' ')}
      />
    );
  }

  return <AppIconFallback name={name} className={className} />;
}

export function AppIconFromBytes({
  name,
  icon,
  className,
}: {
  name: string;
  icon: AppIconBytes | null;
  className?: string;
}) {
  const iconUrl = useMemo(() => {
    if (!icon) return null;

    return URL.createObjectURL(
      new Blob([new Uint8Array(icon.bytes)], { type: icon.mime }),
    );
  }, [icon]);

  useEffect(() => {
    return () => {
      if (iconUrl) URL.revokeObjectURL(iconUrl);
    };
  }, [iconUrl]);

  return <AppIconFromUrl name={name} iconUrl={iconUrl} className={className} />;
}

export function AppIconFromCommonView({ common }: { common: SageAppCommonView }) {
  const icon = common.icon;

  if (!icon) {
    return (
      <AppIconFromBytes
        name={common.activeSnapshot.manifest.name}
        icon={null}
      />
    );
  }

  return (
    <AppIconFromBytes
      name={common.activeSnapshot.manifest.name}
      icon={{
        bytes: icon.bytes,
        mime: icon.mime,
      }}
    />
  );
}

export function appIconFromCommonView(
  common: SageAppCommonView,
): AppIcon | null {
  const icon = common.icon;

  if (!icon) {
    return null;
  }

  return {
    kind: 'bytes',
    icon: {
      bytes: icon.bytes,
      mime: icon.mime,
    },
  };
}
