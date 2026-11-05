import { i18n } from '@lingui/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  commands: {
    cancelOffer: vi.fn(),
    deleteOffer: vi.fn(),
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
    vi.clearAllMocks();
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
