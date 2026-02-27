import { IntegerInput } from '@/components/ui/masked-input';
import { Trans } from '@lingui/react/macro';

export interface Duration {
  days: string;
  hours: string;
  minutes: string;
}

interface DurationInputProps {
  value: Duration;
  onChange: (value: Duration) => void;
}

export function DurationInput({ value, onChange }: DurationInputProps) {
  return (
    <div className='flex gap-2'>
      <div className='relative'>
        <IntegerInput
          className='pr-12'
          value={value.days}
          placeholder='0'
          min={0}
          onValueChange={(values: { value: string }) => {
            onChange({ ...value, days: values.value });
          }}
        />
        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
          <span className='text-muted-foreground text-sm'>
            <Trans>Days</Trans>
          </span>
        </div>
      </div>

      <div className='relative'>
        <IntegerInput
          className='pr-14'
          value={value.hours}
          placeholder='0'
          min={0}
          onValueChange={(values: { value: string }) => {
            onChange({ ...value, hours: values.value });
          }}
        />
        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
          <span className='text-muted-foreground text-sm'>
            <Trans>Hours</Trans>
          </span>
        </div>
      </div>

      <div className='relative'>
        <IntegerInput
          className='pr-[4.5rem]'
          value={value.minutes}
          placeholder='0'
          min={0}
          onValueChange={(values: { value: string }) => {
            onChange({ ...value, minutes: values.value });
          }}
        />
        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
          <span className='text-muted-foreground text-sm'>
            <Trans>Minutes</Trans>
          </span>
        </div>
      </div>
    </div>
  );
}
