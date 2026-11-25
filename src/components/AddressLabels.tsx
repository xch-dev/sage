import { AddressContacts } from '@/components/AddressContacts';
import { AddressHandles } from '@/components/AddressHandles';
import { useAddressContacts } from '@/hooks/useAddressContacts';
import { useAddressHandles } from '@/hooks/useAddressHandles';
import { cn } from '@/lib/utils';

export interface AddressLabelsProps {
  address: string | null | undefined;
  /** Smaller labels, for tables and dense lists. */
  compact?: boolean;
  className?: string;
}

/**
 * The labels Sage shows next to an address it displays: the names it's saved
 * as in the address book, then the handles known to resolve to it. Every
 * surface which shows an address renders this beside it.
 *
 * Renders nothing for values which aren't addresses, or have no labels.
 */
export function AddressLabels({
  address,
  compact,
  className,
}: AddressLabelsProps) {
  const contacts = useAddressContacts(address);
  const handles = useAddressHandles(address);

  if (!address || (contacts.length === 0 && handles.length === 0)) {
    return null;
  }

  return (
    <span
      className={cn(
        'inline-flex max-w-full flex-wrap items-center gap-1',
        className,
      )}
    >
      <AddressContacts address={address} compact={compact} />
      <AddressHandles address={address} compact={compact} />
    </span>
  );
}
