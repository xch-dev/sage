// @vitest-environment jsdom

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const { lookupAddressContacts, lookupAddressHandles } = vi.hoisted(() => ({
  lookupAddressContacts: vi.fn(),
  lookupAddressHandles: vi.fn(),
}));

vi.mock('@/bindings', () => ({
  commands: { lookupAddressContacts, lookupAddressHandles },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));

import type {
  ContactRecord,
  HandleRecord,
  LookupAddressContacts,
} from '@/bindings';
import { addressContacts } from '@/lib/addressContacts';
import { addressHandles } from '@/lib/addressHandles';
import { defaultState, useWalletState } from '@/state';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import {
  cleanup,
  fireEvent,
  render as renderBase,
  screen,
  within,
} from '@testing-library/react';
import { bech32m } from 'bech32';
import type { ReactElement } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { AddressItem } from './AddressItem';

function address(byte: number) {
  return bech32m.encode('txch', bech32m.toWords(new Uint8Array(32).fill(byte)));
}

const ALICE = address(1);
const BOB = address(2);
const CAROL = address(3);

function resolution(
  name: string,
  resolvesTo: string,
  overrides: Partial<HandleRecord> = {},
): HandleRecord {
  return {
    handle: name,
    handle_hash: '00'.repeat(32),
    address: resolvesTo,
    status: 'verified',
    previous_address: null,
    last_sent_address: null,
    changed_from: null,
    cached: true,
    verification: 'verified',
    verification_reason: null,
    owner_launcher_id: '11'.repeat(32),
    resolved_launcher_id: '22'.repeat(32),
    expiration: 2_000_000_000,
    slot_coin_id: '33'.repeat(32),
    slot_confirmation_height: 10,
    resolved_coin_id: '44'.repeat(32),
    resolved_confirmation_height: 12,
    updated_height: 12,
    verified_height: 20,
    verified_timestamp: 1_800_000_000,
    last_verified_at: 1_800_000_000,
    ...overrides,
  };
}

function contact(
  id: number,
  name: string,
  overrides: Partial<ContactRecord>,
): ContactRecord {
  return {
    id,
    name,
    note: null,
    kind: 'address',
    recipient: '',
    handle: null,
    address: null,
    resolution: null,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

const STORE: ContactRecord[] = [
  contact(1, 'Alice', { recipient: ALICE, address: ALICE, note: 'Rent' }),
  contact(2, 'Alice (handle)', {
    kind: 'handle',
    recipient: '@alice',
    handle: 'alice',
    address: ALICE,
    resolution: resolution('alice', ALICE),
  }),
  contact(3, 'Carol', {
    kind: 'handle',
    recipient: '@carol',
    handle: 'carol',
    address: CAROL,
    resolution: resolution('carol', CAROL, { verification: 'outdated' }),
  }),
];

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });

  const sync = defaultState().sync;
  useWalletState.setState({ sync: { ...sync, receive_address: address(9) } });

  lookupAddressContacts.mockImplementation(
    async ({ addresses }: LookupAddressContacts) => ({
      results: addresses.map((requested) => ({
        address: requested,
        contacts: STORE.filter((record) => record.address === requested),
      })),
    }),
  );

  lookupAddressHandles.mockImplementation(
    async ({ addresses }: { addresses: string[] }) => ({
      results: addresses.map((requested) => ({
        address: requested,
        handles: [],
      })),
    }),
  );
});

afterEach(() => {
  cleanup();
  addressContacts.clear();
  addressHandles.clear();
  lookupAddressContacts.mockClear();
});

function render(ui: ReactElement) {
  return renderBase(ui, {
    wrapper: ({ children }) => (
      <MemoryRouter>
        <I18nProvider i18n={i18n}>{children}</I18nProvider>
      </MemoryRouter>
    ),
  });
}

async function openContacts(name: string) {
  const trigger = (await screen.findByText(name)).closest('button');
  if (!trigger) throw new Error(`No label for ${name}`);
  fireEvent.click(trigger);
  return screen.findByRole('dialog');
}

describe('AddressContacts', () => {
  it('labels a page of addresses with one contact lookup', async () => {
    render(
      <div>
        <AddressItem label='Royalties' address={ALICE} />
        <AddressItem label='Owner' address={BOB} />
        <AddressItem label='Coin ID' address={'ab'.repeat(32)} />
      </div>,
    );

    expect(await screen.findByText('Alice')).toBeTruthy();
    expect(lookupAddressContacts).toHaveBeenCalledTimes(1);
    expect(lookupAddressContacts).toHaveBeenCalledWith({
      addresses: [ALICE, BOB],
    });
    // The address is still shown, since a name never replaces it.
    expect(screen.getByDisplayValue(ALICE)).toBeTruthy();
  });

  it('lists every contact saved for an address', async () => {
    render(<AddressItem label='Owner' address={ALICE} />);

    expect(await screen.findByText('+1')).toBeTruthy();
    const popover = await openContacts('Alice');
    expect(within(popover).getByText('Alice (handle)')).toBeTruthy();
    expect(within(popover).getByText('Rent')).toBeTruthy();
    expect(within(popover).getByText(/Saved as @alice/)).toBeTruthy();
  });

  it('warns when a handle contact was not verified recently', async () => {
    render(<AddressItem label='Owner' address={CAROL} />);

    const popover = await openContacts('Carol');
    expect(
      within(popover).getByText(/It may not resolve here anymore/),
    ).toBeTruthy();
    expect(within(popover).queryByText('Verified recently')).toBeNull();
  });

  it('shows nothing for addresses which are not contacts', async () => {
    render(
      <div>
        <div data-testid='unknown'>
          <AddressItem label='Owner' address={BOB} />
        </div>
        <AddressItem label='Royalties' address={ALICE} />
      </div>,
    );

    await screen.findByText('Alice');
    expect(
      within(screen.getByTestId('unknown')).queryByRole('button', {
        name: /Contact:/,
      }),
    ).toBeNull();
  });
});
