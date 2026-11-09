import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';
import { Input } from '@/components/ui/input';
import { IntegerInput } from '@/components/ui/masked-input';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useNowSeconds } from '@/hooks/useNowSeconds';
import {
  durationUntil,
  Expiration,
  expirationError,
  expirationTimestamp,
} from '@/lib/expiration';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CalendarIcon } from 'lucide-react';
import { ReactNode, useId } from 'react';
import { useTheme } from 'theme-o-rama';

export interface ExpirationInputProps {
  value: Expiration;
  onChange: (value: Expiration) => void;
  // Text for the date picker button, given the formatted expiration time.
  formatLabel?: (date: string) => string;
  // Validation message from the parent form, shown below the input.
  error?: string;
}

const pad = (n: number) => n.toString().padStart(2, '0');

export function ExpirationInput({
  value,
  onChange,
  formatLabel = (date) => t`Expires ${date}`,
  error,
}: ExpirationInputProps) {
  const timeInputId = useId();
  const { currentTheme } = useTheme();

  // Many themes use a transparent popover, which leaves the calendar hard to
  // read over the form. Give it a solid base and layer the theme's popover
  // color on top so themed tints still apply.
  const popoverStyle = {
    backgroundColor:
      (currentTheme?.inherits ?? currentTheme?.mostLike) === 'dark'
        ? '#0a0a0a'
        : '#ffffff',
    backgroundImage: 'linear-gradient(var(--popover), var(--popover))',
  };
  const now = useNowSeconds();
  const expiresAtSeconds = expirationTimestamp(value, now);
  const problem = expirationError(value, now);
  // A huge duration can't be represented as a date, so only format it once it
  // is known to be in range.
  const tooFar = problem !== undefined && expiresAtSeconds > now;
  const expiresAt = new Date(tooFar ? now * 1000 : expiresAtSeconds * 1000);
  const expiresAtLabel = tooFar
    ? ''
    : formatTimestamp(expiresAtSeconds, 'medium', 'short');

  // A picked timestamp means the user chose an absolute date and time;
  // otherwise the expiration is a duration counted from now.
  const mode = value.timestamp === undefined ? 'duration' : 'date';
  const remaining = durationUntil(expiresAtSeconds, now);
  const { days, hours, minutes } = remaining;

  // An empty duration is obviously incomplete, but a picked time that has
  // already passed, or anything too far out, needs explaining.
  const message = error ?? (mode === 'date' || tooFar ? problem : undefined);

  const setMode = (newMode: string) => {
    if (newMode === mode) return;

    if (newMode === 'date') {
      // Start from the current expiration, or a day from now if the duration
      // is empty or unusable. Rounding down to the minute keeps the remaining
      // time just under the duration, so durationUntil (which rounds up) gives
      // back the same duration when switching back.
      const start =
        problem === undefined ? expiresAtSeconds : now + 24 * 60 * 60;
      onChange({ ...value, timestamp: Math.floor(start / 60) * 60 });
    } else {
      onChange({ ...value, ...remaining, timestamp: undefined });
    }
  };

  const setDuration = (
    field: 'days' | 'hours' | 'minutes',
    fieldValue: string,
  ) => {
    onChange({ ...value, [field]: fieldValue });
  };

  const setDate = (date: Date) => {
    onChange({ ...value, timestamp: Math.floor(date.getTime() / 1000) });
  };

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const durationField = (
    field: 'days' | 'hours' | 'minutes',
    label: ReactNode,
  ) => (
    <div className='relative'>
      <IntegerInput
        className='pr-12'
        value={value[field]}
        placeholder='0'
        min={0}
        onValueChange={(values, { source }) => {
          // Ignore updates echoed back when the value is set programmatically.
          if (source === 'event') setDuration(field, values.value);
        }}
      />
      <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
        <span className='text-muted-foreground text-sm'>{label}</span>
      </div>
    </div>
  );

  return (
    <div className='flex flex-col gap-2'>
      <Tabs value={mode} onValueChange={setMode}>
        <TabsList>
          <TabsTrigger value='duration'>
            <Trans>Duration</Trans>
          </TabsTrigger>
          <TabsTrigger value='date'>
            <Trans>Date & time</Trans>
          </TabsTrigger>
        </TabsList>
      </Tabs>

      {mode === 'duration' ? (
        <>
          <div className='flex gap-2'>
            {durationField('days', <Trans>Days</Trans>)}
            {durationField('hours', <Trans>Hours</Trans>)}
            {durationField('minutes', <Trans>Minutes</Trans>)}
          </div>
          {!tooFar && (
            <span className='text-muted-foreground text-sm'>
              {formatLabel(expiresAtLabel)}
            </span>
          )}
        </>
      ) : (
        <>
          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant='outline'
                className='w-fit justify-start font-normal'
                aria-label={t`Pick expiration date and time`}
              >
                <CalendarIcon className='mr-2 h-4 w-4' />
                {formatLabel(expiresAtLabel)}
              </Button>
            </PopoverTrigger>
            <PopoverContent
              className='w-auto p-0'
              align='start'
              style={popoverStyle}
            >
              <Calendar
                mode='single'
                selected={expiresAt}
                defaultMonth={expiresAt}
                disabled={{ before: today }}
                onSelect={(day) => {
                  if (!day) return;
                  const date = new Date(expiresAt);
                  date.setFullYear(
                    day.getFullYear(),
                    day.getMonth(),
                    day.getDate(),
                  );
                  setDate(date);
                }}
              />
              <div className='flex items-center gap-2 border-t p-3'>
                <label htmlFor={timeInputId} className='text-sm'>
                  <Trans>Time</Trans>
                </label>
                <Input
                  id={timeInputId}
                  type='time'
                  className='w-fit'
                  value={`${pad(expiresAt.getHours())}:${pad(expiresAt.getMinutes())}`}
                  onChange={(e) => {
                    const [hours, minutes] = e.target.value
                      .split(':')
                      .map(Number);
                    if (isNaN(hours) || isNaN(minutes)) return;
                    const date = new Date(expiresAt);
                    date.setHours(hours, minutes, 0, 0);
                    setDate(date);
                  }}
                />
              </div>
            </PopoverContent>
          </Popover>
          <span className='text-muted-foreground text-sm'>
            <Trans>
              {days}d {hours}h {minutes}m from now
            </Trans>
          </span>
        </>
      )}

      {message && (
        <p className='text-[0.8rem] font-medium text-destructive'>{message}</p>
      )}
    </div>
  );
}
