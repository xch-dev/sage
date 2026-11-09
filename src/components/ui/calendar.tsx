import { ChevronLeft, ChevronRight } from 'lucide-react';
import * as React from 'react';
import { DayPicker, getDefaultClassNames } from 'react-day-picker';

import { buttonVariants } from '@/components/ui/button';
import { cn } from '@/lib/utils';

export type CalendarProps = React.ComponentProps<typeof DayPicker>;

function CalendarChevron({
  orientation,
  className,
}: {
  orientation?: 'up' | 'down' | 'left' | 'right';
  className?: string;
}) {
  return orientation === 'left' ? (
    <ChevronLeft className={cn('h-4 w-4', className)} />
  ) : (
    <ChevronRight className={cn('h-4 w-4', className)} />
  );
}

function Calendar({
  className,
  classNames,
  showOutsideDays = true,
  ...props
}: CalendarProps) {
  const defaultClassNames = getDefaultClassNames();

  return (
    <DayPicker
      showOutsideDays={showOutsideDays}
      className={cn('p-3', className)}
      classNames={{
        root: cn('w-fit', defaultClassNames.root),
        months: cn(
          'relative flex flex-col gap-4 sm:flex-row',
          defaultClassNames.months,
        ),
        month: cn('flex w-full flex-col gap-4', defaultClassNames.month),
        nav: cn(
          'absolute inset-x-0 top-0 flex w-full items-center justify-between gap-1',
          defaultClassNames.nav,
        ),
        button_previous: cn(
          buttonVariants({ variant: 'outline' }),
          'h-7 w-7 p-0 opacity-50 hover:opacity-100 aria-disabled:opacity-25',
          defaultClassNames.button_previous,
        ),
        button_next: cn(
          buttonVariants({ variant: 'outline' }),
          'h-7 w-7 p-0 opacity-50 hover:opacity-100 aria-disabled:opacity-25',
          defaultClassNames.button_next,
        ),
        month_caption: cn(
          'flex h-7 w-full items-center justify-center px-8',
          defaultClassNames.month_caption,
        ),
        caption_label: cn(
          'text-sm font-medium select-none',
          defaultClassNames.caption_label,
        ),
        month_grid: 'w-full border-collapse',
        weekdays: cn('flex', defaultClassNames.weekdays),
        weekday: cn(
          'w-8 rounded-md text-[0.8rem] font-normal text-muted-foreground select-none',
          defaultClassNames.weekday,
        ),
        week: cn('mt-2 flex w-full', defaultClassNames.week),
        day: cn(
          'relative h-8 w-8 p-0 text-center text-sm focus-within:relative focus-within:z-20',
          defaultClassNames.day,
        ),
        day_button: cn(
          buttonVariants({ variant: 'ghost' }),
          'h-8 w-8 p-0 font-normal aria-selected:opacity-100',
        ),
        selected: cn(
          '[&>button]:bg-primary [&>button]:text-primary-foreground [&>button:hover]:bg-primary [&>button:hover]:text-primary-foreground',
          defaultClassNames.selected,
        ),
        today: cn(
          '[&>button]:bg-accent [&>button]:text-accent-foreground',
          defaultClassNames.today,
        ),
        outside: cn(
          'text-muted-foreground opacity-50',
          defaultClassNames.outside,
        ),
        disabled: cn(
          'text-muted-foreground opacity-50',
          defaultClassNames.disabled,
        ),
        hidden: cn('invisible', defaultClassNames.hidden),
        ...classNames,
      }}
      components={{ Chevron: CalendarChevron }}
      {...props}
    />
  );
}
Calendar.displayName = 'Calendar';

export { Calendar };
