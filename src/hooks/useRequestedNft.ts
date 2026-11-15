import { commands, events, NftUpdaterKind } from '@/bindings';
import { nftUri } from '@/lib/nftUri';
import { isValidAddress } from '@/lib/utils';
import { useEffect, useState } from 'react';
import { useDebounce } from './useDebounce';

export type NftLookup =
  | { status: 'idle' }
  | { status: 'invalid' }
  | { status: 'loading' }
  | { status: 'missing' }
  | {
      status: 'found';
      name: string | null;
      kind: NftUpdaterKind;
    };

/**
 * Looks up an NFT the user is requesting, so they can see they typed the
 * right ID before making the offer. The lookup goes to the network, the same
 * way the offer itself fetches a requested NFT.
 */
export function useRequestedNft(nftId: string): {
  lookup: NftLookup;
  thumbnail: string | null;
} {
  const trimmed = nftId.trim();
  const debounced = useDebounce(trimmed, 400);
  const [lookup, setLookup] = useState<NftLookup>({ status: 'idle' });
  const [thumbnail, setThumbnail] = useState<string | null>(null);

  useEffect(() => {
    setThumbnail(null);

    if (!debounced) {
      setLookup({ status: 'idle' });
      return;
    }

    if (!isValidAddress(debounced, 'nft')) {
      setLookup({ status: 'invalid' });
      return;
    }

    let cancelled = false;
    setLookup({ status: 'loading' });

    const updateThumbnail = () =>
      commands
        .getNftThumbnail({ nft_id: debounced })
        .then(({ thumbnail }) => {
          if (!cancelled && thumbnail !== null) {
            setThumbnail(nftUri('image/png', thumbnail));
          }
        })
        .catch(() => undefined);

    commands
      .getNftMetadataUpdater({ nft_id: debounced })
      .then(({ metadata_updater_kind: kind, name }) => {
        if (cancelled) return;
        setLookup(
          kind === null
            ? { status: 'missing' }
            : { status: 'found', name, kind },
        );
        if (kind !== null) updateThumbnail();
      })
      .catch(() => {
        if (!cancelled) setLookup({ status: 'missing' });
      });

    // The image is downloaded after the NFT is first seen, so check again
    // once it arrives.
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'nft_data') updateThumbnail();
    });

    return () => {
      cancelled = true;
      unlisten.then((u) => u());
    };
  }, [debounced]);

  // Until the debounce catches up, an edited ID hasn't been checked yet.
  if (trimmed !== debounced && trimmed) {
    return { lookup: { status: 'loading' }, thumbnail: null };
  }

  return { lookup, thumbnail };
}
