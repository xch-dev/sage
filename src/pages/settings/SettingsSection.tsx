import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, IntegerInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { cloneElement, isValidElement, useId } from 'react';

interface SettingsSectionProps {
  title: string;
  description?: string;
  destructive?: boolean;
  children: React.ReactNode;
}

export function SettingsSection({
  title,
  description,
  destructive,
  children,
}: SettingsSectionProps) {
  return (
    <section
      className={`divide-y rounded-md border bg-card text-card-foreground overflow-hidden ${
        destructive ? 'border-destructive/50' : ''
      }`}
    >
      <div className='p-3 space-y-1'>
        <h3
          className={`text-sm font-medium ${destructive ? 'text-destructive' : ''}`}
        >
          {title}
        </h3>
        {description && (
          <p className='text-sm text-muted-foreground'>{description}</p>
        )}
      </div>
      <div className='divide-y'>{children}</div>
    </section>
  );
}

interface SettingItemProps {
  label: string;
  description?: string;
  control: React.ReactNode;
  children?: React.ReactNode;
}

// Form controls that take their accessible name from the setting's label. Buttons keep their own
// text, wrappers aren't controls, and a Select is labelled on its trigger.
const LABELLED_CONTROLS = new Set<unknown>([Switch, Input, FeeAmountInput]);

export function SettingItem({
  label,
  description,
  control,
  children,
}: SettingItemProps) {
  const labelId = useId();

  return (
    <div className='p-3'>
      <div className='flex flex-col sm:flex-row sm:items-center justify-between gap-x-4 gap-y-2'>
        <div className='space-y-1'>
          <Label id={labelId} className='text-sm font-medium'>
            {label}
          </Label>
          {description && (
            <div className='text-sm text-muted-foreground'>{description}</div>
          )}
        </div>
        <div className='flex sm:justify-end'>
          {isValidElement<{ 'aria-labelledby'?: string }>(control) &&
          LABELLED_CONTROLS.has(control.type)
            ? cloneElement(control, { 'aria-labelledby': labelId })
            : control}
        </div>
      </div>
      {children}
    </div>
  );
}

interface TimeInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export function TimeInput({ label, value, onChange }: TimeInputProps) {
  return (
    <div className='space-y-2'>
      <Label className='text-sm text-muted-foreground'>{label}</Label>
      <IntegerInput
        aria-label={label}
        value={value}
        placeholder='0'
        min={0}
        onValueChange={(values) => onChange(values.value)}
      />
    </div>
  );
}
