import { OfferRecordStatus } from '@/bindings';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import {
  CircleCheck,
  CircleDashed,
  CircleDot,
  CircleOff,
  Hourglass,
  LucideIcon,
} from 'lucide-react';

export function offerStatusLabel(status: OfferRecordStatus): string {
  switch (status) {
    case 'active':
      return t`Active`;
    case 'pending':
      return t`Pending`;
    case 'completed':
      return t`Completed`;
    case 'cancelled':
      return t`Cancelled`;
    case 'expired':
      return t`Expired`;
  }
}

const statusIcons: Record<OfferRecordStatus, [LucideIcon, string]> = {
  active: [CircleDot, 'text-green-600 dark:text-green-400'],
  pending: [CircleDashed, 'text-muted-foreground'],
  completed: [CircleCheck, 'text-primary'],
  cancelled: [CircleOff, 'text-muted-foreground'],
  expired: [Hourglass, 'text-amber-600 dark:text-amber-400'],
};

export interface OfferStatusBadgeProps {
  status: OfferRecordStatus;
  className?: string;
}

/** Offer status as a colored icon and label. */
export function OfferStatusBadge({ status, className }: OfferStatusBadgeProps) {
  const [Icon, color] = statusIcons[status];

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 text-sm font-medium',
        color,
        className,
      )}
    >
      <Icon className='h-4 w-4 flex-shrink-0' aria-hidden='true' />
      {offerStatusLabel(status)}
    </span>
  );
}
