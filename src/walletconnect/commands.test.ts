import { describe, expect, it } from 'vitest';
import { parseCommand, walletConnectCommands } from './commands';
import { ZodError } from 'zod';

describe('parseCommand', () => {
  // ─── chip0002_chainId ───
  describe('chip0002_chainId', () => {
    it('accepts empty object', () => {
      expect(() => parseCommand('chip0002_chainId', {})).not.toThrow();
    });

    it('accepts undefined', () => {
      expect(() => parseCommand('chip0002_chainId', undefined)).not.toThrow();
    });
  });

  // ─── chip0002_connect ───
  describe('chip0002_connect', () => {
    it('accepts empty object', () => {
      expect(() => parseCommand('chip0002_connect', {})).not.toThrow();
    });

    it('accepts eager flag', () => {
      const result = parseCommand('chip0002_connect', { eager: true });
      expect(result?.eager).toBe(true);
    });

    it('accepts undefined', () => {
      expect(() => parseCommand('chip0002_connect', undefined)).not.toThrow();
    });
  });

  // ─── chip0002_getPublicKeys ───
  describe('chip0002_getPublicKeys', () => {
    it('accepts limit and offset', () => {
      const result = parseCommand('chip0002_getPublicKeys', {
        limit: 5,
        offset: 10,
      });
      expect(result?.limit).toBe(5);
      expect(result?.offset).toBe(10);
    });

    it('accepts empty object', () => {
      expect(() =>
        parseCommand('chip0002_getPublicKeys', {}),
      ).not.toThrow();
    });

    it('accepts undefined', () => {
      expect(() =>
        parseCommand('chip0002_getPublicKeys', undefined),
      ).not.toThrow();
    });

    it('rejects string limit', () => {
      expect(() =>
        parseCommand('chip0002_getPublicKeys', { limit: 'five' }),
      ).toThrow(ZodError);
    });
  });

  // ─── chip0002_filterUnlockedCoins ───
  describe('chip0002_filterUnlockedCoins', () => {
    it('accepts valid coinNames', () => {
      const result = parseCommand('chip0002_filterUnlockedCoins', {
        coinNames: ['abc123'],
      });
      expect(result.coinNames).toEqual(['abc123']);
    });

    it('rejects empty coinNames array', () => {
      expect(() =>
        parseCommand('chip0002_filterUnlockedCoins', { coinNames: [] }),
      ).toThrow(ZodError);
    });

    it('rejects missing coinNames', () => {
      expect(() =>
        parseCommand('chip0002_filterUnlockedCoins', {}),
      ).toThrow(ZodError);
    });
  });

  // ─── chip0002_getAssetCoins ───
  describe('chip0002_getAssetCoins', () => {
    it('accepts cat type', () => {
      const result = parseCommand('chip0002_getAssetCoins', {
        type: 'cat',
        assetId: 'abc',
      });
      expect(result.type).toBe('cat');
    });

    it('accepts null type', () => {
      const result = parseCommand('chip0002_getAssetCoins', {
        type: null,
        assetId: null,
      });
      expect(result.type).toBeNull();
    });

    it('rejects invalid type', () => {
      expect(() =>
        parseCommand('chip0002_getAssetCoins', {
          type: 'invalid',
          assetId: null,
        }),
      ).toThrow(ZodError);
    });

    it('accepts optional includedLocked, offset, limit', () => {
      const result = parseCommand('chip0002_getAssetCoins', {
        type: 'nft',
        assetId: 'xyz',
        includedLocked: true,
        offset: 0,
        limit: 50,
      });
      expect(result.includedLocked).toBe(true);
      expect(result.limit).toBe(50);
    });
  });

  // ─── chip0002_getAssetBalance ───
  describe('chip0002_getAssetBalance', () => {
    it('accepts valid params', () => {
      const result = parseCommand('chip0002_getAssetBalance', {
        type: 'cat',
        assetId: 'abc',
      });
      expect(result.type).toBe('cat');
      expect(result.assetId).toBe('abc');
    });

    it('accepts null type and assetId', () => {
      const result = parseCommand('chip0002_getAssetBalance', {
        type: null,
        assetId: null,
      });
      expect(result.type).toBeNull();
    });

    it('rejects missing type', () => {
      expect(() =>
        parseCommand('chip0002_getAssetBalance', { assetId: 'abc' }),
      ).toThrow(ZodError);
    });
  });

  // ─── chip0002_signCoinSpends ───
  describe('chip0002_signCoinSpends', () => {
    const validCoinSpend = {
      coin: { parent_coin_info: 'abc', puzzle_hash: 'def', amount: 100 },
      puzzle_reveal: 'ff01',
      solution: 'ff02',
    };

    it('accepts valid coin spends', () => {
      const result = parseCommand('chip0002_signCoinSpends', {
        coinSpends: [validCoinSpend],
      });
      expect(result.coinSpends).toHaveLength(1);
    });

    it('accepts optional partialSign', () => {
      const result = parseCommand('chip0002_signCoinSpends', {
        coinSpends: [validCoinSpend],
        partialSign: true,
      });
      expect(result.partialSign).toBe(true);
    });

    it('rejects missing coinSpends', () => {
      expect(() => parseCommand('chip0002_signCoinSpends', {})).toThrow(
        ZodError,
      );
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chip0002_signCoinSpends.confirm).toBe(true);
    });
  });

  // ─── chip0002_signMessage ───
  describe('chip0002_signMessage', () => {
    it('accepts valid params', () => {
      const result = parseCommand('chip0002_signMessage', {
        message: 'hello',
        publicKey: 'abc123',
      });
      expect(result.message).toBe('hello');
    });

    it('rejects missing message', () => {
      expect(() =>
        parseCommand('chip0002_signMessage', { publicKey: 'abc' }),
      ).toThrow(ZodError);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chip0002_signMessage.confirm).toBe(true);
    });
  });

  // ─── chip0002_sendTransaction ───
  describe('chip0002_sendTransaction', () => {
    const validBundle = {
      coin_spends: [
        {
          coin: { parent_coin_info: 'a', puzzle_hash: 'b', amount: 1 },
          puzzle_reveal: 'ff',
          solution: 'ff',
        },
      ],
      aggregated_signature: 'sig',
    };

    it('accepts valid spend bundle', () => {
      const result = parseCommand('chip0002_sendTransaction', {
        spendBundle: validBundle,
      });
      expect(result.spendBundle.aggregated_signature).toBe('sig');
    });

    it('rejects missing spendBundle', () => {
      expect(() => parseCommand('chip0002_sendTransaction', {})).toThrow(
        ZodError,
      );
    });

    it('does not require confirm', () => {
      expect(walletConnectCommands.chip0002_sendTransaction.confirm).toBe(
        false,
      );
    });
  });

  // ─── chia_createOffer ───
  describe('chia_createOffer', () => {
    it('accepts valid offer params', () => {
      const result = parseCommand('chia_createOffer', {
        offerAssets: [{ assetId: '', amount: 1000 }],
        requestAssets: [{ assetId: 'cat123', amount: '500' }],
      });
      expect(result.offerAssets).toHaveLength(1);
      expect(result.requestAssets).toHaveLength(1);
    });

    it('accepts string amounts (safeAmount)', () => {
      const result = parseCommand('chia_createOffer', {
        offerAssets: [{ assetId: '', amount: '1000' }],
        requestAssets: [],
        fee: '100',
      });
      expect(result.fee).toBe('100');
    });

    it('rejects missing offerAssets', () => {
      expect(() =>
        parseCommand('chia_createOffer', { requestAssets: [] }),
      ).toThrow(ZodError);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_createOffer.confirm).toBe(true);
    });
  });

  // ─── chia_takeOffer ───
  describe('chia_takeOffer', () => {
    it('accepts valid params', () => {
      const result = parseCommand('chia_takeOffer', {
        offer: 'offer1...',
      });
      expect(result.offer).toBe('offer1...');
    });

    it('accepts optional fee', () => {
      const result = parseCommand('chia_takeOffer', {
        offer: 'offer1...',
        fee: 100,
      });
      expect(result.fee).toBe(100);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_takeOffer.confirm).toBe(true);
    });
  });

  // ─── chia_cancelOffer ───
  describe('chia_cancelOffer', () => {
    it('accepts valid params', () => {
      const result = parseCommand('chia_cancelOffer', {
        id: 'offer-id-123',
      });
      expect(result.id).toBe('offer-id-123');
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_cancelOffer.confirm).toBe(true);
    });
  });

  // ─── chia_getNfts ───
  describe('chia_getNfts', () => {
    it('accepts limit and offset', () => {
      const result = parseCommand('chia_getNfts', {
        limit: 20,
        offset: 5,
      });
      expect(result.limit).toBe(20);
    });

    it('accepts collectionId', () => {
      const result = parseCommand('chia_getNfts', {
        collectionId: 'col123',
      });
      expect(result.collectionId).toBe('col123');
    });

    it('accepts empty object', () => {
      expect(() => parseCommand('chia_getNfts', {})).not.toThrow();
    });

    it('does not require confirm', () => {
      expect(walletConnectCommands.chia_getNfts.confirm).toBe(false);
    });
  });

  // ─── chia_send ───
  describe('chia_send', () => {
    it('accepts XCH send (no assetId)', () => {
      const result = parseCommand('chia_send', {
        amount: 1000,
        address: 'xch1...',
      });
      expect(result.assetId).toBeUndefined();
    });

    it('accepts CAT send (with assetId)', () => {
      const result = parseCommand('chia_send', {
        assetId: 'abc',
        amount: '500',
        address: 'xch1...',
      });
      expect(result.assetId).toBe('abc');
    });

    it('accepts optional memos', () => {
      const result = parseCommand('chia_send', {
        amount: 1,
        address: 'xch1...',
        memos: ['hello', 'world'],
      });
      expect(result.memos).toEqual(['hello', 'world']);
    });

    it('rejects missing address', () => {
      expect(() =>
        parseCommand('chia_send', { amount: 1 }),
      ).toThrow(ZodError);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_send.confirm).toBe(true);
    });
  });

  // ─── chia_bulkMintNfts ───
  describe('chia_bulkMintNfts', () => {
    it('accepts valid mint params', () => {
      const result = parseCommand('chia_bulkMintNfts', {
        did: 'did:chia:...',
        nfts: [
          {
            dataUris: ['https://example.com/image.png'],
            dataHash: 'abc123',
          },
        ],
      });
      expect(result.nfts).toHaveLength(1);
    });

    it('rejects missing did', () => {
      expect(() =>
        parseCommand('chia_bulkMintNfts', { nfts: [] }),
      ).toThrow(ZodError);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_bulkMintNfts.confirm).toBe(true);
    });
  });

  // ─── chia_getAddress ───
  describe('chia_getAddress', () => {
    it('accepts empty object', () => {
      expect(() => parseCommand('chia_getAddress', {})).not.toThrow();
    });

    it('does not require confirm', () => {
      expect(walletConnectCommands.chia_getAddress.confirm).toBe(false);
    });
  });

  // ─── chia_signMessageByAddress ───
  describe('chia_signMessageByAddress', () => {
    it('accepts valid params', () => {
      const result = parseCommand('chia_signMessageByAddress', {
        message: 'test',
        address: 'xch1...',
      });
      expect(result.message).toBe('test');
      expect(result.address).toBe('xch1...');
    });

    it('rejects missing address', () => {
      expect(() =>
        parseCommand('chia_signMessageByAddress', { message: 'test' }),
      ).toThrow(ZodError);
    });

    it('requires confirm', () => {
      expect(walletConnectCommands.chia_signMessageByAddress.confirm).toBe(
        true,
      );
    });
  });
});

describe('confirm flags', () => {
  const readOnlyCommands: (keyof typeof walletConnectCommands)[] = [
    'chip0002_chainId',
    'chip0002_connect',
    'chip0002_getPublicKeys',
    'chip0002_filterUnlockedCoins',
    'chip0002_getAssetCoins',
    'chip0002_getAssetBalance',
    'chip0002_sendTransaction',
    'chia_getNfts',
    'chia_getAddress',
  ];

  const confirmCommands: (keyof typeof walletConnectCommands)[] = [
    'chip0002_signCoinSpends',
    'chip0002_signMessage',
    'chia_createOffer',
    'chia_takeOffer',
    'chia_cancelOffer',
    'chia_send',
    'chia_bulkMintNfts',
    'chia_signMessageByAddress',
  ];

  it.each(readOnlyCommands)(
    '%s does not require confirmation',
    (command) => {
      expect(walletConnectCommands[command].confirm).toBe(false);
    },
  );

  it.each(confirmCommands)('%s requires confirmation', (command) => {
    expect(walletConnectCommands[command].confirm).toBe(true);
  });
});
