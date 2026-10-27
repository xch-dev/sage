// @vitest-environment jsdom

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const { lookupAddressHandles, resolveHandle } = vi.hoisted(() => ({
  lookupAddressHandles: vi.fn(),
  resolveHandle: vi.fn(),
}));

vi.mock('@/bindings', () => ({
  commands: { lookupAddressHandles, resolveHandle },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));

import type { HandleRecord, LookupAddressHandles } from '@/bindings';
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
  resolveHandle.mockReset();
});

function render(ui: ReactElement) {
  return renderBase(ui, {
    wrapper: ({ children }) => (
      <I18nProvider i18n={i18n}>{children}</I18nProvider>
    ),
  });
}

/** Opens the popover listing every known handle for an address. */
async function openHandles(name: string) {
  const trigger = (await screen.findByText(name)).closest('button');
  if (!trigger) throw new Error(`No label for ${name}`);
  fireEvent.click(trigger);
  return screen.findByRole('dialog');
}

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

  it('lists every handle for an address, in a fixed order', async () => {
    render(<AddressItem label='Owner' address={ALICE} />);

    const trigger = (await screen.findByText('@alice')).closest('button');
    expect(trigger?.textContent).toContain('+1');

    const popover = await openHandles('@alice');
    const names = within(popover)
      .getAllByText(/^@/)
      .map((element) => element.textContent);
    expect(names).toEqual(['@alice', '@ally']);
    // The full address can be copied from the popover too.
    expect(within(popover).getByText(ALICE)).toBeTruthy();
  });

  it("doesn't call an outdated mapping verified", async () => {
    render(<AddressItem label='Owner' address={CAROL} />);

    const trigger = (await screen.findByText('@carol')).closest('button');
    const name = trigger?.getAttribute('aria-label') ?? '';
    expect(name).toContain('Not verified recently');
    expect(name).not.toContain('(Verified)');

    const popover = await openHandles('@carol');
    expect(within(popover).queryByText('Verified')).toBeNull();
    expect(within(popover).getByText('Not verified recently')).toBeTruthy();
  });

  it('verifies an outdated handle again on request, then relabels', async () => {
    resolveHandle.mockResolvedValue({ handle: handle('carol', CAROL) });
    render(<AddressItem label='Owner' address={CAROL} />);

    const popover = await openHandles('@carol');
    fireEvent.click(
      within(popover).getByRole('button', { name: /Verify again/ }),
    );

    await vi.waitFor(() =>
      expect(lookupAddressHandles).toHaveBeenCalledTimes(2),
    );
    expect(resolveHandle).toHaveBeenCalledWith({ handle: 'carol' });
  });

  it('only offers to verify again when a handle is outdated', async () => {
    render(<AddressItem label='Owner' address={BOB} />);

    const popover = await openHandles('@bob');
    expect(
      within(popover).queryByRole('button', { name: /Verify again/ }),
    ).toBeNull();
  });

  it('shows nothing for addresses without known handles', async () => {
    const unknown = address(4);
    render(
      <div>
        <div data-testid='unknown'>
          <AddressItem label='Owner' address={unknown} />
        </div>
        <AddressItem label='Royalties' address={BOB} />
      </div>,
    );

    // Both are looked up together, so once one label arrives the other's
    // result has too.
    await screen.findByText('@bob');
    expect(lookupAddressHandles).toHaveBeenCalledWith({
      addresses: [unknown, BOB],
    });
    expect(within(screen.getByTestId('unknown')).queryByText(/^@/)).toBeNull();
  });
});
