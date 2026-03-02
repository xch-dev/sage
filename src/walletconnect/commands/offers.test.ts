import { vi, describe, expect, it, beforeEach } from 'vitest';

vi.mock('@/bindings', () => ({
  commands: {
    makeOffer: vi.fn(),
    takeOffer: vi.fn(),
    cancelOffer: vi.fn(),
  },
}));

import { commands } from '@/bindings';
import { HandlerContext } from '../handler';
import { handleCreateOffer, handleTakeOffer, handleCancelOffer } from './offers';

const mockCommands = vi.mocked(commands);

function makeContext(authResult = true): HandlerContext {
  return {
    promptIfEnabled: vi.fn(async () => authResult),
  };
}

describe('handleCreateOffer', () => {
  beforeEach(() => vi.clearAllMocks());

  it('maps empty assetId to null', async () => {
    mockCommands.makeOffer.mockResolvedValue({
      offer: 'offer-blob',
      offer_id: 'id-123',
    } as never);

    const result = await handleCreateOffer(
      {
        offerAssets: [{ assetId: '', amount: 1000 }],
        requestAssets: [{ assetId: 'cat1', amount: 500 }],
      },
      makeContext(),
    );

    expect(mockCommands.makeOffer).toHaveBeenCalledWith(
      expect.objectContaining({
        offered_assets: [
          expect.objectContaining({ asset_id: null }),
        ],
        requested_assets: [
          expect.objectContaining({ asset_id: 'cat1' }),
        ],
      }),
    );
    expect(result.offer).toBe('offer-blob');
    expect(result.id).toBe('id-123');
  });

  it('passes fee with default of 0', async () => {
    mockCommands.makeOffer.mockResolvedValue({
      offer: 'x',
      offer_id: 'y',
    } as never);

    await handleCreateOffer(
      {
        offerAssets: [],
        requestAssets: [],
      },
      makeContext(),
    );

    expect(mockCommands.makeOffer).toHaveBeenCalledWith(
      expect.objectContaining({ fee: 0 }),
    );
  });

  it('throws on biometric failure', async () => {
    await expect(
      handleCreateOffer(
        { offerAssets: [], requestAssets: [] },
        makeContext(false),
      ),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleTakeOffer', () => {
  beforeEach(() => vi.clearAllMocks());

  it('passes offer and fee to takeOffer', async () => {
    mockCommands.takeOffer.mockResolvedValue({
      transaction_id: 'tx-1',
    } as never);

    const result = await handleTakeOffer(
      { offer: 'offer-blob', fee: 50 },
      makeContext(),
    );

    expect(mockCommands.takeOffer).toHaveBeenCalledWith({
      offer: 'offer-blob',
      fee: 50,
      auto_submit: true,
    });
    expect(result.id).toBe('tx-1');
  });

  it('defaults fee to 0', async () => {
    mockCommands.takeOffer.mockResolvedValue({
      transaction_id: 'tx-2',
    } as never);

    await handleTakeOffer({ offer: 'x' }, makeContext());

    expect(mockCommands.takeOffer).toHaveBeenCalledWith(
      expect.objectContaining({ fee: 0 }),
    );
  });

  it('throws on auth failure', async () => {
    await expect(
      handleTakeOffer({ offer: 'x' }, makeContext(false)),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleCancelOffer', () => {
  beforeEach(() => vi.clearAllMocks());

  it('passes offer id and fee', async () => {
    mockCommands.cancelOffer.mockResolvedValue(undefined as never);

    const result = await handleCancelOffer(
      { id: 'offer-456', fee: 10 },
      makeContext(),
    );

    expect(mockCommands.cancelOffer).toHaveBeenCalledWith({
      offer_id: 'offer-456',
      fee: 10,
      auto_submit: true,
    });
    expect(result).toEqual({});
  });

  it('throws on auth failure', async () => {
    await expect(
      handleCancelOffer({ id: 'x' }, makeContext(false)),
    ).rejects.toThrow('Authentication failed');
  });
});
