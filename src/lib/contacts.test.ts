import { describe, expect, it } from 'vitest';
import { ContactRecord, HandleRecord } from '@/bindings';
import { contactState, handleState, matchContact } from './contacts';

const record: HandleRecord = {
  handle: 'alice',
  handle_hash: 'aa',
  address: 'txch1new',
  status: 'verified',
  previous_address: null,
  last_sent_address: null,
  changed_from: null,
  cached: true,
  verification: 'verified',
  verification_reason: null,
  owner_launcher_id: 'bb',
  resolved_launcher_id: 'cc',
  expiration: 2000000000,
  slot_coin_id: 'dd',
  slot_confirmation_height: 1,
  resolved_coin_id: 'ee',
  resolved_confirmation_height: 1,
  updated_height: 1,
  verified_height: 2,
  verified_timestamp: 3,
  last_verified_at: 4,
};

const contact: ContactRecord = {
  id: 1,
  name: 'Alice',
  note: null,
  kind: 'handle',
  recipient: '@alice',
  handle: 'alice',
  address: null,
  resolution: null,
  created_at: 1,
  updated_at: 1,
};

describe('contact states', () => {
  it('distinguishes raw addresses from handles', () => {
    expect(
      contactState({ ...contact, kind: 'address', recipient: 'txch1a' }),
    ).toBe('address');
    expect(contactState(contact)).toBe('unresolved');
    expect(contactState({ ...contact, resolution: record })).toBe('verified');
  });

  it('flags a changed handle', () => {
    expect(
      handleState({
        ...record,
        status: 'changed',
        previous_address: 'txch1old',
      }),
    ).toBe('changed');
    // A change from the address last sent to is flagged too.
    expect(handleState({ ...record, changed_from: 'txch1old' })).toBe(
      'changed',
    );
    // Even when the mapping is old.
    expect(
      handleState({
        ...record,
        verification: 'outdated',
        changed_from: 'txch1old',
      }),
    ).toBe('changed');
  });

  it('never shows an unusable mapping as verified', () => {
    for (const verification of [
      'outdated',
      'stale',
      'unverified',
      'expired',
    ] as const) {
      expect(handleState({ ...record, verification })).toBe(verification);
    }

    expect(
      handleState({
        ...record,
        verification: 'expired',
        changed_from: 'txch1old',
      }),
    ).toBe('expired');
  });

  it('matches a recipient field to a contact exactly', () => {
    expect(matchContact([contact], ' @alice ')).toBe(contact);
    expect(matchContact([contact], '@alic')).toBeUndefined();
  });
});
