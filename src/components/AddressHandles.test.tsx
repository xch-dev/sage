// @vitest-environment jsdom

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const { lookupAddressHandles } = vi.hoisted(() => ({
  lookupAddressHandles: vi.fn(),
}));

vi.mock('@/bindings', () => ({
  commands: { lookupAddressHandles },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));

import type { HandleRecord, LookupAddressHandles } from '@/bindings';
import { addressHandles } from '@/lib/addressHandles';
import { defaultState, useWalletState } from '@/state';
import { i18n } from '@lingui/core';
import { cleanup, render, screen } from '@testing-library/react';
import { bech32m } from 'bech32';
import { AddressItem } from './AddressItem';

function address(byte: number, prefix = 'txch') {
  return bech32m.encode(prefix, bech32m.toWords(new Uint8Array(32).fill(byte)));
}

const ALICE = address(1);
const BOB = address(2);
const CAROL = address(3);

function handle(
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

const STORE = [
  handle('bob', BOB),
  handle('alice', ALICE),
  handle('ally', ALICE),
  handle('carol', CAROL, { verification: 'outdated' }),
];

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });

  const sync = defaultState().sync;
  useWalletState.setState({ sync: { ...sync, receive_address: address(9) } });

  lookupAddressHandles.mockImplementation(
    async ({ addresses }: LookupAddressHandles) => ({
      results: addresses.map((requested) => ({
        address: requested,
        handles: STORE.filter((record) => record.address === requested),
      })),
    }),
  );
});

afterEach(() => {
  cleanup();
  addressHandles.clear();
  lookupAddressHandles.mockClear();
});

describe('AddressHandles', () => {
  it('labels a page of addresses with one lookup', async () => {
    render(
      <div>
        <AddressItem label='Royalties' address={ALICE} />
        <AddressItem label='Owner' address={BOB} />
        <AddressItem label='Again' address={ALICE} />
        <AddressItem label='Coin ID' address={'ab'.repeat(32)} />
        <AddressItem label='Mainnet' address={address(1, 'xch')} />
      </div>,
    );

    expect(await screen.findByText('@bob')).toBeTruthy();
    expect(lookupAddressHandles).toHaveBeenCalledTimes(1);
    // Values which aren't addresses on this network aren't looked up.
    expect(lookupAddressHandles).toHaveBeenCalledWith({
      addresses: [ALICE, BOB],
    });
  });

  it('keeps the full address next to the handle', async () => {
    render(<AddressItem label='Owner' address={BOB} />);

    expect(await screen.findByText('@bob')).toBeTruthy();
    expect(screen.getByDisplayValue(BOB)).toBeTruthy();
  });

  it('shows the first of several handles and makes the rest discoverable', async () => {
    render(<AddressItem label='Owner' address={ALICE} />);

    const label = await screen.findByText('@alice');
    const trigger = label.closest('button');
    expect(trigger?.textContent).toContain('+1');
    expect(trigger?.getAttribute('aria-label')).toContain('1 more handles');
  });

  it("doesn't call an outdated mapping verified", async () => {
    render(<AddressItem label='Owner' address={CAROL} />);

    const trigger = (await screen.findByText('@carol')).closest('button');
    expect(trigger?.getAttribute('title')).toBe(
      '@carol (Not verified recently)',
    );
    expect(trigger?.className).toContain('border-dashed');
  });

  it('shows nothing for addresses without known handles', async () => {
    const { container } = render(
      <AddressItem label='Owner' address={address(4)} />,
    );

    await vi.waitFor(() => expect(lookupAddressHandles).toHaveBeenCalled());
    expect(container.querySelector('button[title^="@"]')).toBeNull();
  });
});
