import { Card, CardContent } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { ArrowDownUp, ArrowLeftRight } from 'lucide-react';
import { ReactNode, useId } from 'react';

export interface OfferSettingCardProps {
  title: ReactNode;
  description?: ReactNode;
  children?: ReactNode;
}

export function OfferSettingCard({
  title,
  description,
  children,
}: OfferSettingCardProps) {
  return (
    <Card>
      <CardContent className='flex flex-col gap-4 pt-6'>
        <div className='flex flex-col gap-1'>
          <h3 className='font-heading text-base font-semibold leading-none'>
            {title}
          </h3>
          {description && (
            <p className='text-sm text-muted-foreground'>{description}</p>
          )}
        </div>
        {children}
      </CardContent>
    </Card>
  );
}

/** Sits between the offered and requested cards, which stack below `lg`. */
export function OfferSidesDivider() {
  return (
    <div
      aria-hidden
      className='relative z-10 -my-2.5 flex items-center justify-center lg:my-0 lg:-mx-2.5'
    >
      <span className='flex h-9 w-9 items-center justify-center rounded-full border border-border bg-background text-muted-foreground shadow-sm'>
        <ArrowDownUp className='h-4 w-4 lg:hidden' />
        <ArrowLeftRight className='hidden h-4 w-4 lg:block' />
      </span>
    </div>
  );
}

export interface OfferChoiceOption {
  value: string;
  label: ReactNode;
  description?: ReactNode;
  /** Shown below the option while it is selected. */
  children?: ReactNode;
}

export interface OfferChoiceProps {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  options: OfferChoiceOption[];
  disabled?: boolean;
}

/** A pair of radio options laid out side by side, each with its own details. */
export function OfferChoice({
  label,
  value,
  onValueChange,
  options,
  disabled,
}: OfferChoiceProps) {
  const name = useId();

  return (
    <div
      role='radiogroup'
      aria-label={label}
      aria-disabled={disabled}
      className='grid divide-y divide-border sm:grid-cols-2 sm:divide-x sm:divide-y-0'
    >
      {options.map((option) => {
        const checked = option.value === value;

        return (
          <div
            key={option.value}
            className={cn(
              'flex min-w-0 flex-col gap-3 py-4 transition-opacity first:pt-0 last:pb-0 sm:py-0 sm:first:pr-6 sm:last:pl-6',
              !checked && !disabled && 'opacity-60 hover:opacity-90',
              disabled && 'opacity-50',
            )}
          >
            <label
              className={cn(
                'relative flex items-start gap-3 text-sm',
                disabled ? 'cursor-not-allowed' : 'cursor-pointer',
              )}
            >
              <input
                type='radio'
                name={name}
                value={option.value}
                checked={checked}
                disabled={disabled}
                onChange={() => onValueChange(option.value)}
                className='peer sr-only'
              />
              <span
                className={cn(
                  'mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full border peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background',
                  checked ? 'border-primary' : 'border-muted-foreground',
                )}
              >
                {checked && (
                  <span className='h-2 w-2 rounded-full bg-primary' />
                )}
              </span>
              <span className='flex flex-col gap-1'>
                <span className='font-medium leading-5'>{option.label}</span>
                {option.description && (
                  <span className='text-muted-foreground'>
                    {option.description}
                  </span>
                )}
              </span>
            </label>
            {checked && option.children && (
              <div className='sm:pl-7'>{option.children}</div>
            )}
          </div>
        );
      })}
    </div>
  );
}
