import { ContactRecord, HandleRecord } from '@/bindings';
import { xchandlesEnabled } from '@/lib/features';

/**
 * How far a contact can be relied on, from the wallet's records. Handle
 * contacts are read from the verified mapping store, so even `verified` only
 * means it was verified recently: the handle is resolved and checked again
 * whenever it's used as a recipient.
 */
export type ContactState =
  | 'address'
  | 'unresolved'
  | 'verified'
  | 'outdated'
  | 'changed'
  | 'stale'
  | 'unverified'
  | 'expired';

export function handleState(record: HandleRecord): ContactState {
  // A change has to be acknowledged before the handle can be used, so it's
  // shown above everything but a mapping which can't be used at all.
  switch (record.verification) {
    case 'expired':
    case 'unverified':
    case 'stale':
      return record.verification;
  }

  if (record.status === 'changed' || record.changed_from) return 'changed';

  return record.verification;
}

export function contactState(contact: ContactRecord): ContactState {
  if (contact.kind === 'address') return 'address';
  if (!contact.resolution) return 'unresolved';
  return handleState(contact.resolution);
}

/**
 * Builds without XCHandles still have an address book, but only for raw
 * addresses, since handles aren't offered anywhere else in the interface.
 */
export const contactKindFilter = xchandlesEnabled ? null : ('address' as const);

/** The contact a recipient field's value refers to, if any. */
export function matchContact(
  contacts: ContactRecord[],
  value: string,
): ContactRecord | undefined {
  const recipient = value.trim();
  return contacts.find((contact) => contact.recipient === recipient);
}
