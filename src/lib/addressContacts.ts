import { AddressContactsRecord, commands, ContactRecord } from '@/bindings';
import { addressHandles } from '@/lib/addressHandles';
import { xchandlesEnabled } from '@/lib/features';
import { AddressLookupCache, AddressLookupCacheOptions } from './addressLookup';

/** Shared by every address without contacts, so it's a stable value. */
export const NO_CONTACTS: ContactRecord[] = [];

/**
 * Keeps only the contacts which may label an address. Builds without
 * XCHandles never show handle contacts, so only address contacts are kept.
 * The backend already sorts contacts by name.
 */
export function labelContacts(
  contacts: ContactRecord[],
  handlesEnabled = xchandlesEnabled,
): ContactRecord[] {
  return handlesEnabled
    ? contacts
    : contacts.filter((contact) => contact.kind === 'address');
}

export type LookupContacts = (
  addresses: string[],
) => Promise<AddressContactsRecord[]>;

/**
 * A client-side cache of the contacts each address is saved as, looked up with
 * `lookup_address_contacts`. That includes handle contacts whose verified
 * mapping resolves to the address.
 */
export class AddressContactCache extends AddressLookupCache<ContactRecord> {
  constructor(
    lookup: LookupContacts,
    options: AddressLookupCacheOptions & { handlesEnabled?: boolean } = {},
  ) {
    const { handlesEnabled = xchandlesEnabled, ...rest } = options;

    super(
      {
        lookup: async (addresses) =>
          new Map(
            (await lookup(addresses)).map((record) => [
              record.address,
              record.contacts,
            ]),
          ),
        normalize: (contacts) => labelContacts(contacts, handlesEnabled),
        empty: NO_CONTACTS,
        description: 'contacts',
      },
      rest,
    );
  }
}

export const addressContacts = new AddressContactCache(async (addresses) => {
  const response = await commands.lookupAddressContacts({ addresses });
  return response.results;
});

/**
 * Looks up every shown address's contacts and handles again, after contacts
 * were saved, renamed or deleted, or a handle was verified, so that labels on
 * screen don't show what was known before.
 */
export function refreshAddressLabels() {
  addressContacts.refresh();
  addressHandles.refresh();
}
