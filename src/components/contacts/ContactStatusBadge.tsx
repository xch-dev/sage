import { ContactState } from '@/lib/contacts';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import {
  AlertTriangleIcon,
  CheckCircleIcon,
  ClockIcon,
  HelpCircleIcon,
  XCircleIcon,
} from 'lucide-react';

const STYLES: Record<ContactState, string> = {
  address: 'border-border text-muted-foreground',
  unresolved: 'border-border text-muted-foreground',
  verified: 'border-green-600/40 text-green-700 dark:text-green-500',
  outdated: 'border-yellow-600/40 text-yellow-700 dark:text-yellow-500',
  changed: 'border-destructive/50 text-destructive',
  stale: 'border-yellow-600/40 text-yellow-700 dark:text-yellow-500',
  unverified: 'border-destructive/50 text-destructive',
  expired: 'border-destructive/50 text-destructive',
};

export function contactStateLabel(state: ContactState): string {
  switch (state) {
    case 'address':
      return t`Address`;
    case 'unresolved':
      return t`Not verified yet`;
    case 'verified':
      return t`Verified recently`;
    case 'outdated':
      return t`Outdated`;
    case 'changed':
      return t`Address changed`;
    case 'stale':
      return t`Needs verification`;
    case 'unverified':
      return t`Verification failed`;
    case 'expired':
      return t`Expired`;
  }
}

export function contactStateDescription(state: ContactState): string {
  switch (state) {
    case 'address':
      return t`A raw address, which is used exactly as saved.`;
    case 'unresolved':
      return t`This handle hasn't been resolved yet. It's verified when you send to it.`;
    case 'verified':
      return t`Verified against the blockchain within the last day. It's verified again whenever you send to it.`;
    case 'outdated':
      return t`Last verified more than a day ago, so it may have changed. It's verified again when you send to it.`;
    case 'changed':
      return t`This handle now resolves to a different address than before. You'll have to confirm the new address before sending to it.`;
    case 'stale':
      return t`A chain reorganization may have undone the coins this was verified against. It must be verified again before it can be used.`;
    case 'unverified':
      return t`The last lookup was rejected or the handle is no longer registered. Only the last verified address is shown.`;
    case 'expired':
      return t`This handle has expired since it was last verified.`;
  }
}

function StateIcon({ state }: { state: ContactState }) {
  const className = 'h-3 w-3 shrink-0';

  switch (state) {
    case 'verified':
      return <CheckCircleIcon className={className} aria-hidden='true' />;
    case 'outdated':
    case 'stale':
      return <ClockIcon className={className} aria-hidden='true' />;
    case 'changed':
      return <AlertTriangleIcon className={className} aria-hidden='true' />;
    case 'unverified':
    case 'expired':
      return <XCircleIcon className={className} aria-hidden='true' />;
    case 'unresolved':
      return <HelpCircleIcon className={className} aria-hidden='true' />;
    case 'address':
      return null;
  }
}

export function ContactStatusBadge({
  state,
  className,
}: {
  state: ContactState;
  className?: string;
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px] font-medium whitespace-nowrap',
        STYLES[state],
        className,
      )}
      title={contactStateDescription(state)}
    >
      <StateIcon state={state} />
      {contactStateLabel(state)}
    </span>
  );
}
