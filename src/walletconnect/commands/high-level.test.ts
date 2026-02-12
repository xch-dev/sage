import { vi, describe, expect, it, beforeEach } from 'vitest';

vi.mock('@/bindings', () => ({
  commands: {
    getNfts: vi.fn(),
    sendCat: vi.fn(),
    sendXch: vi.fn(),
    signMessageByAddress: vi.fn(),
    bulkMintNfts: vi.fn(),
  },
}));

vi.mock('@/state', () => ({
  useWalletState: {
    getState: vi.fn(() => ({
      sync: { receive_address: 'xch1testaddr' },
    })),
  },
}));

import { commands } from '@/bindings';
import { HandlerContext } from '../handler';
import {
  handleGetNfts,
  handleSend,
  handleGetAddress,
  handleSignMessageByAddress,
  handleBulkMintNfts,
} from './high-level';

const mockCommands = vi.mocked(commands);

function makeContext(authResult = true): HandlerContext {
  return {
    promptIfEnabled: vi.fn(async () => authResult),
  };
}

describe('handleGetNfts', () => {
  beforeEach(() => vi.clearAllMocks());

  it('maps NFT fields from snake_case to camelCase', async () => {
    mockCommands.getNfts.mockResolvedValue({
      nfts: [
        {
          name: 'Test NFT',
          launcher_id: 'nft1',
          minter_did: 'did1',
          owner_did: 'did2',
          collection_id: 'col1',
          collection_name: 'Collection',
          created_height: 100,
          coin_id: 'coin1',
          address: 'addr1',
          royalty_address: 'raddr',
          royalty_ten_thousandths: 300,
          data_uris: ['https://example.com/data'],
          data_hash: 'hash1',
          metadata_uris: ['https://example.com/meta'],
          metadata_hash: 'hash2',
          license_uris: [],
          license_hash: null,
          edition_number: 1,
          edition_total: 10,
        },
      ],
    } as never);

    const result = await handleGetNfts({});

    expect(result.nfts).toHaveLength(1);
    const nft = result.nfts[0];
    expect(nft.launcherId).toBe('nft1');
    expect(nft.minterDid).toBe('did1');
    expect(nft.ownerDid).toBe('did2');
    expect(nft.collectionId).toBe('col1');
    expect(nft.collectionName).toBe('Collection');
    expect(nft.createdHeight).toBe(100);
    expect(nft.coinId).toBe('coin1');
    expect(nft.royaltyTenThousandths).toBe(300);
    expect(nft.dataUris).toEqual(['https://example.com/data']);
  });

  it('defaults limit=10, offset=0', async () => {
    mockCommands.getNfts.mockResolvedValue({ nfts: [] } as never);

    await handleGetNfts({});

    expect(mockCommands.getNfts).toHaveBeenCalledWith(
      expect.objectContaining({ limit: 10, offset: 0 }),
    );
  });

  it('passes collectionId', async () => {
    mockCommands.getNfts.mockResolvedValue({ nfts: [] } as never);

    await handleGetNfts({ collectionId: 'col-abc' });

    expect(mockCommands.getNfts).toHaveBeenCalledWith(
      expect.objectContaining({ collection_id: 'col-abc' }),
    );
  });
});

describe('handleSend', () => {
  beforeEach(() => vi.clearAllMocks());

  it('sends XCH when no assetId', async () => {
    mockCommands.sendXch.mockResolvedValue(undefined as never);

    const result = await handleSend(
      { amount: 1000, address: 'xch1dest', memos: ['memo1'] },
      makeContext(),
    );

    expect(mockCommands.sendXch).toHaveBeenCalledWith({
      address: 'xch1dest',
      amount: 1000,
      fee: 0,
      memos: ['memo1'],
      auto_submit: true,
    });
    expect(mockCommands.sendCat).not.toHaveBeenCalled();
    expect(result).toEqual({});
  });

  it('sends CAT when assetId is present', async () => {
    mockCommands.sendCat.mockResolvedValue(undefined as never);

    await handleSend(
      { assetId: 'cat-id', amount: 500, address: 'xch1dest', fee: 10 },
      makeContext(),
    );

    expect(mockCommands.sendCat).toHaveBeenCalledWith({
      asset_id: 'cat-id',
      address: 'xch1dest',
      amount: 500,
      fee: 10,
      memos: [],
      auto_submit: true,
    });
    expect(mockCommands.sendXch).not.toHaveBeenCalled();
  });

  it('defaults fee to 0 and memos to empty', async () => {
    mockCommands.sendXch.mockResolvedValue(undefined as never);

    await handleSend(
      { amount: 1, address: 'xch1x' },
      makeContext(),
    );

    expect(mockCommands.sendXch).toHaveBeenCalledWith(
      expect.objectContaining({ fee: 0, memos: [] }),
    );
  });

  it('throws on auth failure', async () => {
    await expect(
      handleSend({ amount: 1, address: 'xch1x' }, makeContext(false)),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleGetAddress', () => {
  it('returns receive_address from wallet state', async () => {
    const result = await handleGetAddress();
    expect(result.address).toBe('xch1testaddr');
  });
});

describe('handleSignMessageByAddress', () => {
  beforeEach(() => vi.clearAllMocks());

  it('passes params and returns result', async () => {
    mockCommands.signMessageByAddress.mockResolvedValue({
      publicKey: 'pk1',
      signature: 'sig1',
    } as never);

    await handleSignMessageByAddress(
      { message: 'hello', address: 'xch1addr' },
      makeContext(),
    );

    expect(mockCommands.signMessageByAddress).toHaveBeenCalledWith({
      message: 'hello',
      address: 'xch1addr',
    });
  });

  it('throws on auth failure', async () => {
    await expect(
      handleSignMessageByAddress(
        { message: 'x', address: 'y' },
        makeContext(false),
      ),
    ).rejects.toThrow('Authentication failed');
  });
});

describe('handleBulkMintNfts', () => {
  beforeEach(() => vi.clearAllMocks());

  it('maps mint params and returns nft IDs', async () => {
    mockCommands.bulkMintNfts.mockResolvedValue({
      nft_ids: ['nft1', 'nft2'],
    } as never);

    const result = await handleBulkMintNfts(
      {
        did: 'did:chia:abc',
        nfts: [
          {
            dataUris: ['https://example.com/img.png'],
            dataHash: 'hash123',
            royaltyTenThousandths: 300,
          },
        ],
        fee: 100,
      },
      makeContext(),
    );

    expect(result.nftIds).toEqual(['nft1', 'nft2']);
    expect(mockCommands.bulkMintNfts).toHaveBeenCalledWith({
      did_id: 'did:chia:abc',
      fee: 100,
      auto_submit: true,
      mints: [
        expect.objectContaining({
          data_uris: ['https://example.com/img.png'],
          data_hash: 'hash123',
          royalty_ten_thousandths: 300,
        }),
      ],
    });
  });

  it('throws when dataUris present without dataHash', async () => {
    await expect(
      handleBulkMintNfts(
        {
          did: 'did:chia:abc',
          nfts: [{ dataUris: ['https://example.com/x'] }],
        },
        makeContext(),
      ),
    ).rejects.toThrow('Data hash is required');
  });

  it('throws when metadataUris present without metadataHash', async () => {
    await expect(
      handleBulkMintNfts(
        {
          did: 'did:chia:abc',
          nfts: [{ metadataUris: ['https://example.com/meta'] }],
        },
        makeContext(),
      ),
    ).rejects.toThrow('Metadata hash is required');
  });

  it('throws when licenseUris present without licenseHash', async () => {
    await expect(
      handleBulkMintNfts(
        {
          did: 'did:chia:abc',
          nfts: [{ licenseUris: ['https://example.com/license'] }],
        },
        makeContext(),
      ),
    ).rejects.toThrow('License hash is required');
  });

  it('throws on auth failure', async () => {
    await expect(
      handleBulkMintNfts(
        { did: 'did:chia:abc', nfts: [] },
        makeContext(false),
      ),
    ).rejects.toThrow('Authentication failed');
  });
});
