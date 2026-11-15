import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';
import { ArrowDownUp, ArrowLeftRight } from 'lucide-react';
import { ReactNode, useId } from 'react';

export interface OfferSettingToggle {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
}

export interface OfferSettingCardProps {
  title: ReactNode;
  description?: ReactNode;
  /** Adds a switch to the header; the card's content shows while it's on. */
  toggle?: OfferSettingToggle;
  children?: ReactNode;
}

export function OfferSettingCard({
  title,
  description,
  toggle,
  children,
}: OfferSettingCardProps) {
  const id = useId();
  const showContent = children && (!toggle || toggle.checked);

  return (
    <Card>
      <CardHeader
        className={cn(
          'flex-row items-start justify-between gap-4 space-y-0',
          showContent && 'pb-4',
        )}
      >
        <div className='flex flex-col gap-1.5'>
          <CardTitle id={`${id}-title`} className='text-base'>
            {title}
          </CardTitle>
          {description && (
            <CardDescription id={`${id}-description`}>
              {description}
            </CardDescription>
          )}
        </div>
        {toggle && (
          <Switch
            checked={toggle.checked}
            onCheckedChange={toggle.onCheckedChange}
            disabled={toggle.disabled}
            aria-labelledby={`${id}-title`}
            aria-describedby={description ? `${id}-description` : undefined}
          />
        )}
      </CardHeader>
      {showContent && <CardContent>{children}</CardContent>}
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
}

export interface OfferChoiceProps {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  options: [OfferChoiceOption, OfferChoiceOption];
}

/** A pair of radio options, side by side on wide screens. */
export function OfferChoice({
  label,
  value,
  onValueChange,
  options,
}: OfferChoiceProps) {
  const id = useId();

  return (
    <fieldset className='grid min-w-0 lg:grid-cols-2'>
      <legend className='sr-only'>{label}</legend>
      {options.map((option, index) => {
        const checked = option.value === value;
        const inputId = `${id}-${index}`;

        return (
          <div
            key={option.value}
            className={cn(
              'flex min-w-0 flex-col gap-3 transition-opacity',
              index === 0
                ? 'pb-4 lg:pb-0 lg:pr-6'
                : 'border-t border-border pt-4 lg:border-l lg:border-t-0 lg:pl-6 lg:pt-0',
              !checked && 'opacity-60 hover:opacity-90',
            )}
          >
            <label
              htmlFor={inputId}
              className='relative flex cursor-pointer items-start gap-3 text-sm'
            >
              <input
                id={inputId}
                type='radio'
                name={id}
                value={option.value}
                checked={checked}
                onChange={() => onValueChange(option.value)}
                aria-labelledby={`${inputId}-label`}
                aria-describedby={
                  option.description ? `${inputId}-description` : undefined
                }
                className='peer sr-only'
              />
              <span
                aria-hidden
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
                <span id={`${inputId}-label`} className='font-medium leading-5'>
                  {option.label}
                </span>
                {option.description && (
                  <span
                    id={`${inputId}-description`}
                    className='text-muted-foreground'
                  >
                    {option.description}
                  </span>
                )}
              </span>
            </label>
          </div>
        );
      })}
    </fieldset>
  );
}
