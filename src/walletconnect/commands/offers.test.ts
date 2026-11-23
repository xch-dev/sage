import { i18n } from '@lingui/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  commands: {
    cancelOffer: vi.fn(),
    deleteOffer: vi.fn(),
    getOffer: vi.fn(),
  },
}));

vi.mock('@/bindings', () => ({
  commands: mocks.commands,
  events: { syncEvent: { listen: vi.fn() } },
}));

const { handleCancelOffer } = await import('./offers');
const { parseCommand } = await import('../commands');
const { handleCommand } = await import('../handler');

i18n.loadAndActivate({ locale: 'en', messages: {} });

const context = {
  promptIfEnabled: () => Promise.resolve(true),
  isReadOnly: false,
};

describe('chia_cancelOffer', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('cancels on-chain by default', async () => {
    await handleCancelOffer(
      parseCommand('chia_cancelOffer', { id: 'abc', fee: 5 }),
      context,
    );

    expect(mocks.commands.cancelOffer).toHaveBeenCalledWith({
      offer_id: 'abc',
      fee: 5,
      auto_submit: true,
    });
    expect(mocks.commands.deleteOffer).not.toHaveBeenCalled();
  });

  it('only deletes the offer when offChain is set', async () => {
    await handleCancelOffer(
      parseCommand('chia_cancelOffer', { id: 'abc', fee: 5, offChain: true }),
      context,
    );

    expect(mocks.commands.deleteOffer).toHaveBeenCalledWith({
      offer_id: 'abc',
    });
    expect(mocks.commands.cancelOffer).not.toHaveBeenCalled();
  });

  it('cancels on-chain when offChain is false', async () => {
    await handleCancelOffer(
      parseCommand('chia_cancelOffer', { id: 'abc', offChain: false }),
      context,
    );

    expect(mocks.commands.cancelOffer).toHaveBeenCalled();
    expect(mocks.commands.deleteOffer).not.toHaveBeenCalled();
  });

  it('fails instead of deleting an offer the wallet does not have', async () => {
    mocks.commands.getOffer.mockRejectedValue({ kind: 'not_found' });

    await expect(
      handleCancelOffer(
        parseCommand('chia_cancelOffer', { id: 'missing', offChain: true }),
        context,
      ),
    ).rejects.toEqual({ kind: 'not_found' });
    expect(mocks.commands.deleteOffer).not.toHaveBeenCalled();
  });

  it('does not delete the offer if authentication fails', async () => {
    await expect(
      handleCancelOffer(
        parseCommand('chia_cancelOffer', { id: 'abc', offChain: true }),
        { ...context, promptIfEnabled: () => Promise.resolve(false) },
      ),
    ).rejects.toThrow('Authentication failed');
    expect(mocks.commands.deleteOffer).not.toHaveBeenCalled();
  });

  it.each(['true', 1])(
    'does not treat offChain %j as off-chain for a read-only wallet',
    async (offChain) => {
      await expect(
        handleCommand(
          'chia_cancelOffer',
          { id: 'abc', offChain },
          { ...context, isReadOnly: true },
        ),
      ).rejects.toThrow('read-only');
      expect(mocks.commands.deleteOffer).not.toHaveBeenCalled();
    },
  );

  it('allows off-chain cancellation from a read-only wallet', async () => {
    await expect(
      handleCommand(
        'chia_cancelOffer',
        { id: 'abc', offChain: true },
        { ...context, isReadOnly: true },
      ),
    ).resolves.toEqual({});

    await expect(
      handleCommand(
        'chia_cancelOffer',
        { id: 'abc' },
        { ...context, isReadOnly: true },
      ),
    ).rejects.toThrow('read-only');
  });
});
