import { AddressHandles } from '@/components/AddressHandles';

export interface AddressLabelsProps {
  address: string | null | undefined;
  /** Smaller labels, for tables and dense lists. */
  compact?: boolean;
  className?: string;
}

/**
 * The labels Sage shows next to an address it displays. Every surface which
 * shows an address renders this beside it, so other labels, like contact names
 * from the address book, only need to be added here.
 *
 * Renders nothing for values which aren't addresses, or have no labels.
 */
export function AddressLabels({
  address,
  compact,
  className,
}: AddressLabelsProps) {
  return (
    <AddressHandles address={address} compact={compact} className={className} />
  );
}
