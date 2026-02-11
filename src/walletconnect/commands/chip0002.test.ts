import { vi, describe, expect, it, beforeEach } from 'vitest';

vi.mock('@/bindings', () => ({
  commands: {
    getNetwork: vi.fn(),
    getDerivations: vi.fn(),
    filterUnlockedCoins: vi.fn(),
    getAssetCoins: vi.fn(),
    signCoinSpends: vi.fn(),
    signMessageWithPublicKey: vi.fn(),
    sendTransactionImmediately: vi.fn(),
  },
}));

import { commands } from '@/bindings';
import { HandlerContext } from '../handler';
import {
  handleChainId,
  handleConnect,
  handleGetPublicKeys,
  handleFilterUnlockedCoins,
  handleGetAssetBalance,
  handleSignCoinSpends,
  handleSignMessage,
  handleSendTransaction,
} from './chip0002';

const mockCommands = vi.mocked(commands);

function makeContext(authResult = true): HandlerContext {
  return {
    promptIfEnabled: vi.fn(async () => authResult),
  };
}

describe('handleChainId', () => {
  beforeEach(() => vi.clearAllMocks());

  it('returns network_id when present', async () => {
    mockCommands.getNetwork.mockResolvedValue({
      network: { network_id: 'mainnet', name: 'Chia Mainnet' },
    } as never);

    const result = await handleChainId();
    expect(result).toBe('mainnet');
  });

  it('falls back to name when network_id is empty', async () => {
    mockCommands.getNetwork.mockResolvedValue({
      network: { network_id: '', name: 'Chia Testnet' },
    } as never);

    const result = await handleChainId();
    expect(result).toBe('Chia Testnet');
  });
});

describe('handleConnect', () => {
  it('always returns true', async () => {
    expect(await handleConnect()).toBe(true);
  });
});

describe('handleGetPublicKeys', () => {
  beforeEach(() => vi.clearAllMocks());

  it('maps derivations to public keys', async () => {
    mockCommands.getDerivations.mockResolvedValue({
      derivations: [
        { public_key: 'pk1', index: 0 },
        { public_key: 'pk2', index: 1 },
      ],
    } as never);

    const result = await handleGetPublicKeys({ limit: 10, offset: 0 });
    expect(result).toEqual(['pk1', 'pk2']);
  });

  it('defaults limit=10 and offset=0', async () => {
    mockCommands.getDerivations.mockResolvedValue({
      derivations: [],
    } as never);

    await handleGetPublicKeys(undefined);

    expect(mockCommands.getDerivations).toHaveBeenCalledWith({
      limit: 10,
      offset: 0,
    });
  });

  it('uses provided limit and offset', async () => {
    mockCommands.getDerivations.mockResolvedValue({
      derivations: [],
    } as never);

    await handleGetPublicKeys({ limit: 5, offset: 20 });

    expect(mockCommands.getDerivations).toHaveBeenCalledWith({
      limit: 5,
      offset: 20,
    });
  });
});

describe('handleFilterUnlockedCoins', () => {
  beforeEach(() => vi.clearAllMocks());

  it('passes coinNames as coin_ids', async () => {
    mockCommands.filterUnlockedCoins.mockResolvedValue(['coin1'] as never);

    const result = await handleFilterUnlockedCoins({
      coinNames: ['coin1', 'coin2'],
    });

    expect(mockCommands.filterUnlockedCoins).toHaveBeenCalledWith({
      coin_ids: ['coin1', 'coin2'],
    });
    expect(result).toEqual(['coin1']);
  });
});

describe('handleGetAssetBalance', () => {
  beforeEach(() => vi.clearAllMocks());

  it('accumulates balance from coin records', async () => {
    mockCommands.getAssetCoins.mockResolvedValue([
      { coin: { amount: 100 }, locked: false },
      { coin: { amount: 200 }, locked: true },
      { coin: { amount: 50 }, locked: false },
    ] as never);

    const result = await handleGetAssetBalance({
      type: null,
      assetId: null,
    });

    expect(result.confirmed).toBe('350');
    expect(result.spendable).toBe('150');
    expect(result.spendableCoinCount).toBe(2);
  });

  it('handles empty coin list', async () => {
    mockCommands.getAssetCoins.mockResolvedValue([] as never);

    const result = await handleGetAssetBalance({
      type: 'cat',
      assetId: 'abc',
    });

    expect(result.confirmed).toBe('0');
    expect(result.spendable).toBe('0');
    expect(result.spendableCoinCount).toBe(0);
  });

  it('passes includedLocked: true', async () => {
    mockCommands.getAssetCoins.mockResolvedValue([] as never);

    await handleGetAssetBalance({ type: null, assetId: null });

    expect(mockCommands.getAssetCoins).toHaveBeenCalledWith(
      expect.objectContaining({ includedLocked: true }),
    );
  });
});

describe('handleSignCoinSpends', () => {
  beforeEach(() => vi.clearAllMocks());

  it('converts amount to string and returns aggregated signature', async () => {
    mockCommands.signCoinSpends.mockResolvedValue({
      spend_bundle: { aggregated_signature: 'sig123' },
    } as never);

    const result = await handleSignCoinSpends(
      {
        coinSpends: [
          {
            coin: { parent_coin_info: 'p', puzzle_hash: 'h', amount: 42 },
            puzzle_reveal: 'pr',
            solution: 'sol',
          },
        ],
      },
      makeContext(),
    );

    expect(result).toBe('sig123');
    expect(mockCommands.signCoinSpends).toHaveBeenCalledWith({
      coin_spends: [
        {
          coin: { parent_coin_info: 'p', puzzle_hash: 'h', amount: '42' },
          puzzle_reveal: 'pr',
          solution: 'sol',
        },
      ],
      partial: undefined,
      auto_submit: false,
    });
  });

  it('throws on biometric failure', async () => {
    await expect(
      handleSignCoinSpends(
        {
          coinSpends: [
            {
              coin: { parent_coin_info: 'a', puzzle_hash: 'b', amount: 1 },
              puzzle_reveal: 'x',
              solution: 'y',
            },
          ],
        },
        makeContext(false),
      ),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleSignMessage', () => {
  beforeEach(() => vi.clearAllMocks());

  it('returns signature', async () => {
    mockCommands.signMessageWithPublicKey.mockResolvedValue({
      signature: 'sig456',
    } as never);

    const result = await handleSignMessage(
      { message: 'hello', publicKey: 'pk' },
      makeContext(),
    );

    expect(result).toBe('sig456');
  });

  it('throws on auth failure', async () => {
    await expect(
      handleSignMessage(
        { message: 'test', publicKey: 'pk' },
        makeContext(false),
      ),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleSendTransaction', () => {
  beforeEach(() => vi.clearAllMocks());

  it('sends spend bundle', async () => {
    const bundle = {
      coin_spends: [],
      aggregated_signature: 'agg',
    };
    mockCommands.sendTransactionImmediately.mockResolvedValue({
      status: 1,
      error: null,
    } as never);

    const result = await handleSendTransaction({ spendBundle: bundle });

    expect(mockCommands.sendTransactionImmediately).toHaveBeenCalledWith({
      spend_bundle: bundle,
    });
    expect(result).toEqual({ status: 1, error: null });
  });
});
