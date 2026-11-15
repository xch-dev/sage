import { Label } from '@/components/ui/label';

interface SettingsSectionProps {
  title: string;
  children: React.ReactNode;
}

export function SettingsSection({ title, children }: SettingsSectionProps) {
  return (
    <div className='divide-y rounded-md border bg-card text-card-foreground overflow-hidden'>
      <div className='p-3'>
        <h3 className='text-sm font-medium'>{title}</h3>
      </div>
      <div className='divide-y'>{children}</div>
    </div>
  );
}

interface SettingItemProps {
  label: string;
  description?: string;
  control: React.ReactNode;
  children?: React.ReactNode;
}

export function SettingItem({
  label,
  description,
  control,
  children,
}: SettingItemProps) {
  return (
    <div className='p-3'>
      <div className='flex flex-col sm:flex-row sm:items-center justify-between gap-x-4 gap-y-2'>
        <div className='space-y-1'>
          <Label className='text-sm font-medium'>{label}</Label>
          {description && (
            <div className='text-sm text-muted-foreground'>{description}</div>
          )}
        </div>
        <div className='flex sm:justify-end'>{control}</div>
      </div>
      {children}
    </div>
  );
}
