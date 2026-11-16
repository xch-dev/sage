// @vitest-environment jsdom

import { act, cleanup, render, screen } from '@testing-library/react';
import { bech32m } from 'bech32';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const getNftMetadataUpdater = vi.fn();
const getRequestedNftThumbnail = vi.fn();

vi.mock('@/bindings', () => ({
  commands: {
    getNftMetadataUpdater: (...args: unknown[]) =>
      getNftMetadataUpdater(...args),
    getRequestedNftThumbnail: (...args: unknown[]) =>
      getRequestedNftThumbnail(...args),
  },
}));

import { useRequestedNft } from './useRequestedNft';

const encodeNftId = (byte: number) =>
  bech32m.encode('nft', bech32m.toWords(new Uint8Array(32).fill(byte)));
const nftId = encodeNftId(7);
const otherNftId = encodeNftId(8);

function Probe({ id }: { id: string }) {
  const lookup = useRequestedNft(id);
  return (
    <div>
      <span data-testid='status'>{lookup.status}</span>
      <span data-testid='name'>
        {lookup.status === 'found' ? (lookup.name ?? '') : ''}
      </span>
      <span data-testid='thumbnail'>
        {lookup.status === 'found' ? (lookup.thumbnail ?? '') : ''}
      </span>
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
  getRequestedNftThumbnail.mockResolvedValue({ thumbnail: null });
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.resetAllMocks();
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
  });

  it('treats a failed lookup as missing', async () => {
    getNftMetadataUpdater.mockRejectedValue(new Error('no peer'));

    render(<Probe id={nftId} />);
    await settle();

    expect(screen.getByTestId('status').textContent).toBe('missing');
  });

  it('waits for typing to stop before looking up', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: 'default',
      name: 'Cool NFT',
    });

    // An ID already there when the row appears is looked up straight away,
    // so start from an empty field and type.
    const { rerender } = render(<Probe id='' />);
    rerender(<Probe id={otherNftId} />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });
    rerender(<Probe id={nftId} />);
    await settle();

    expect(getNftMetadataUpdater).toHaveBeenCalledTimes(1);
    expect(getNftMetadataUpdater).toHaveBeenCalledWith({ nft_id: nftId });
  });

  it('ignores a lookup that finishes after the ID changed', async () => {
    let finishFirst: (value: unknown) => void = () => undefined;
    getNftMetadataUpdater
      .mockReturnValueOnce(
        new Promise((resolve) => {
          finishFirst = resolve;
        }),
      )
      .mockResolvedValueOnce({
        metadata_updater_kind: 'default',
        name: 'Second NFT',
      });

    const { rerender } = render(<Probe id={otherNftId} />);
    await settle();
    rerender(<Probe id={nftId} />);
    // The new ID hasn't been checked yet, so nothing from the old one shows.
    expect(screen.getByTestId('status').textContent).toBe('loading');
    expect(screen.getByTestId('name').textContent).toBe('');
    await settle();
    expect(screen.getByTestId('name').textContent).toBe('Second NFT');

    await act(async () => {
      finishFirst({ metadata_updater_kind: 'default', name: 'First NFT' });
    });
    expect(screen.getByTestId('name').textContent).toBe('Second NFT');
  });

  it('clears a found NFT as soon as the field is emptied or broken', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: 'default',
      name: 'Cool NFT',
    });

    const { rerender } = render(<Probe id={nftId} />);
    await settle();
    expect(screen.getByTestId('status').textContent).toBe('found');

    rerender(<Probe id='' />);
    expect(screen.getByTestId('status').textContent).toBe('idle');

    rerender(<Probe id='not an nft' />);
    expect(screen.getByTestId('status').textContent).toBe('invalid');
  });
  it('shows the thumbnail once it loads, without holding up the lookup', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: 'default',
      name: 'Cool NFT',
    });
    let finishThumbnail: (value: unknown) => void = () => undefined;
    getRequestedNftThumbnail.mockReturnValue(
      new Promise((resolve) => {
        finishThumbnail = resolve;
      }),
    );

    render(<Probe id={nftId} />);
    await settle();

    expect(screen.getByTestId('status').textContent).toBe('found');
    expect(screen.getByTestId('thumbnail').textContent).toBe('');
    expect(getRequestedNftThumbnail).toHaveBeenCalledWith({ nft_id: nftId });

    await act(async () => {
      finishThumbnail({ thumbnail: 'AQID' });
    });
    expect(screen.getByTestId('thumbnail').textContent).toBe(
      'data:image/png;base64,AQID',
    );
  });

  it("doesn't ask for a thumbnail of an NFT that wasn't found", async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: null,
      name: null,
    });

    render(<Probe id={nftId} />);
    await settle();

    expect(getRequestedNftThumbnail).not.toHaveBeenCalled();
  });

  it('keeps a found NFT when its thumbnail fails to load', async () => {
    getNftMetadataUpdater.mockResolvedValue({
      metadata_updater_kind: 'default',
      name: 'Cool NFT',
    });
    getRequestedNftThumbnail.mockRejectedValue(new Error('offline'));

    render(<Probe id={nftId} />);
    await settle();

    expect(screen.getByTestId('status').textContent).toBe('found');
    expect(screen.getByTestId('thumbnail').textContent).toBe('');
  });

  it('ignores a thumbnail that loads after the ID changed', async () => {
    getNftMetadataUpdater
      .mockResolvedValueOnce({
        metadata_updater_kind: 'default',
        name: 'First',
      })
      .mockResolvedValueOnce({
        metadata_updater_kind: 'default',
        name: 'Second',
      });
    let finishFirst: (value: unknown) => void = () => undefined;
    getRequestedNftThumbnail
      .mockReturnValueOnce(
        new Promise((resolve) => {
          finishFirst = resolve;
        }),
      )
      .mockResolvedValueOnce({ thumbnail: null });

    const { rerender } = render(<Probe id={otherNftId} />);
    await settle();
    rerender(<Probe id={nftId} />);
    await settle();
    expect(screen.getByTestId('name').textContent).toBe('Second');

    await act(async () => {
      finishFirst({ thumbnail: 'AQID' });
    });
    expect(screen.getByTestId('thumbnail').textContent).toBe('');
  });
});
