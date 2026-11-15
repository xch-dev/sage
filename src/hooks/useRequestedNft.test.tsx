// @vitest-environment jsdom

import { act, cleanup, render, screen } from '@testing-library/react';
import { bech32m } from 'bech32';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const getNftMetadataUpdater = vi.fn();
const getNftThumbnail = vi.fn();

vi.mock('@/bindings', () => ({
  commands: {
    getNftMetadataUpdater: (...args: unknown[]) =>
      getNftMetadataUpdater(...args),
    getNftThumbnail: (...args: unknown[]) => getNftThumbnail(...args),
  },
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));

import { useRequestedNft } from './useRequestedNft';

const nftId = bech32m.encode(
  'nft',
  bech32m.toWords(new Uint8Array(32).fill(7)),
);

function Probe({ id }: { id: string }) {
  const { lookup, thumbnail } = useRequestedNft(id);
  return (
    <div>
      <span data-testid='status'>{lookup.status}</span>
      <span data-testid='name'>
        {lookup.status === 'found' ? (lookup.name ?? '') : ''}
      </span>
      <span data-testid='thumbnail'>{thumbnail ?? ''}</span>
    </div>
  );
}

async function settle() {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(500);
  });
}

beforeEach(() => {
  vi.useFakeTimers();
  getNftThumbnail.mockResolvedValue({ thumbnail: null });
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe('useRequestedNft', () => {
  it('stays idle while the field is empty', async () => {
    render(<Probe id='  ' />);
    await settle();
    expect(screen.getByTestId('status').textContent).toBe('idle');
    expect(getNftMetadataUpdater).not.toHaveBeenCalled();
  });

  it('rejects text that is not an NFT ID without a lookup', async () => {
    render(<Probe id='nft1notreal' />);
    await settle();
    expect(screen.getByTestId('status').textContent).toBe('invalid');
    expect(getNftMetadataUpdater).not.toHaveBeenCalled();
  });

  it('shows the name of an NFT found on the network', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: 'default',
      name: 'Cool NFT',
    });

    render(<Probe id={nftId} />);
    expect(screen.getByTestId('status').textContent).toBe('loading');
    await settle();

    expect(getNftMetadataUpdater).toHaveBeenCalledWith({ nft_id: nftId });
    expect(screen.getByTestId('status').textContent).toBe('found');
    expect(screen.getByTestId('name').textContent).toBe('Cool NFT');
  });

  it('reports an NFT the network does not know', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: null,
      name: null,
    });

    render(<Probe id={nftId} />);
    await settle();

    expect(screen.getByTestId('status').textContent).toBe('missing');
    expect(getNftThumbnail).not.toHaveBeenCalled();
  });

  it('treats a failed lookup as missing', async () => {
    getNftMetadataUpdater.mockRejectedValue(new Error('no peer'));

    render(<Probe id={nftId} />);
    await settle();

    expect(screen.getByTestId('status').textContent).toBe('missing');
  });
});
