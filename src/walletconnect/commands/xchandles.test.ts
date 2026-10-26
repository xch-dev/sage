import { beforeEach, describe, expect, it, vi } from 'vitest';
import { HandleRecord } from '@/bindings';
import { WalletConnectError } from '../errors';

const mocks = vi.hoisted(() => ({
  commands: {
    resolveHandle: vi.fn(),
    getHandles: vi.fn(),
  },
  features: { xchandlesEnabled: true },
}));

vi.mock('@/bindings', () => ({ commands: mocks.commands }));
vi.mock('@/lib/features', () => mocks.features);

const { handleGetOwnedHandles, handleResolveHandle } =
  await import('./xchandles');

const record: HandleRecord = {
  handle: 'alice',
  handle_hash: 'aa',
  address: 'txch1new',
  status: 'changed',
  previous_address: 'txch1old',
  last_sent_address: null,
  cached: false,
  verification: 'verified',
  verification_reason: null,
  owner_launcher_id: 'bb',
  resolved_launcher_id: 'cc',
  expiration: 2000000000,
  slot_coin_id: 'dd',
  slot_confirmation_height: 1,
  resolved_coin_id: 'ee',
  verified_height: 2,
  verified_timestamp: 3,
  last_verified_at: 4,
};

describe('XCHandles WalletConnect methods', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.features.xchandlesEnabled = true;
  });

  it('returns the change warning along with the address', async () => {
    mocks.commands.resolveHandle.mockResolvedValue({ handle: record });

    const result = await handleResolveHandle({ handle: '@alice' });

    expect(mocks.commands.resolveHandle).toHaveBeenCalledWith({
      handle: '@alice',
    });
    expect(result).toMatchObject({
      address: 'txch1new',
      status: 'changed',
      previousAddress: 'txch1old',
      verification: 'verified',
      cached: false,
    });
  });

  it('passes on the handle error code', async () => {
    mocks.commands.resolveHandle.mockRejectedValue({
      kind: 'not_found',
      reason: 'Handle has expired',
      handle_error: 'expired',
    });

    const error = await handleResolveHandle({ handle: 'alice' }).catch(
      (error) => error,
    );

    expect(error).toBeInstanceOf(WalletConnectError);
    expect(error.data).toBe('expired');
    expect(error.message).toBe('Handle has expired');
  });

  it('only lists verified owned handles, without refreshing', async () => {
    mocks.commands.getHandles.mockResolvedValue({
      handles: [record],
      total: 1,
    });

    const result = await handleGetOwnedHandles(undefined);

    expect(mocks.commands.getHandles).toHaveBeenCalledWith({
      filter: 'owned',
      include_unverified: false,
      refresh: false,
      offset: 0,
      limit: 50,
    });
    expect(result.total).toBe(1);
    expect(result.handles[0].handle).toBe('alice');
  });

  it('is unavailable when XCHandles is disabled', async () => {
    mocks.features.xchandlesEnabled = false;

    await expect(handleResolveHandle({ handle: 'alice' })).rejects.toThrow();
    expect(mocks.commands.resolveHandle).not.toHaveBeenCalled();
  });
});
