import { i18n } from '@lingui/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ContactRecord } from '@/bindings';

const mocks = vi.hoisted(() => ({
  commands: {
    getContacts: vi.fn(),
    addContact: vi.fn(),
  },
  features: { xchandlesEnabled: true },
}));

vi.mock('@/bindings', () => ({ commands: mocks.commands }));
vi.mock('@/lib/features', () => mocks.features);

const { handleAddContact, handleGetContacts } = await import('./contacts');

i18n.loadAndActivate({ locale: 'en', messages: {} });

const contact: ContactRecord = {
  id: 7,
  name: 'Alice',
  note: null,
  kind: 'handle',
  recipient: '@alice',
  handle: 'alice',
  address: null,
  resolution: null,
  created_at: 1,
  updated_at: 2,
};

describe('address book WalletConnect methods', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.features.xchandlesEnabled = true;
  });

  it('lists contacts in camelCase', async () => {
    mocks.commands.getContacts.mockResolvedValue({
      contacts: [contact],
      total: 1,
    });

    const result = await handleGetContacts({ search: 'ali' });

    expect(mocks.commands.getContacts).toHaveBeenCalledWith({
      search: 'ali',
      kind: null,
      offset: 0,
      limit: 50,
    });
    expect(result).toEqual({
      contacts: [
        {
          id: 7,
          name: 'Alice',
          note: null,
          kind: 'handle',
          recipient: '@alice',
          handle: 'alice',
          address: null,
          resolution: null,
          createdAt: 1,
          updatedAt: 2,
        },
      ],
      total: 1,
    });
  });

  it('only shares address contacts when XCHandles is disabled', async () => {
    mocks.features.xchandlesEnabled = false;
    mocks.commands.getContacts.mockResolvedValue({ contacts: [], total: 0 });

    await handleGetContacts(undefined);

    expect(mocks.commands.getContacts).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'address' }),
    );
  });

  it('adds a suggested contact', async () => {
    mocks.commands.addContact.mockResolvedValue({ contact });

    const result = await handleAddContact({
      name: 'Alice',
      recipient: '@alice',
    });

    expect(mocks.commands.addContact).toHaveBeenCalledWith({
      name: 'Alice',
      recipient: '@alice',
      note: null,
    });
    expect(result.id).toBe(7);
  });

  it('rejects handle contacts when XCHandles is disabled', async () => {
    mocks.features.xchandlesEnabled = false;

    await expect(
      handleAddContact({ name: 'Alice', recipient: '@alice' }),
    ).rejects.toThrow();
    expect(mocks.commands.addContact).not.toHaveBeenCalled();
  });
});
