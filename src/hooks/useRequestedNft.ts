import { commands, NftUpdaterKind } from '@/bindings';
import { isValidAddress } from '@/lib/utils';
import { useEffect, useState } from 'react';
import { useDebounce } from './useDebounce';
import { useNftThumbnails } from './useOwnedNft';

export type NftLookup =
  | { status: 'idle' }
  | { status: 'invalid' }
  | { status: 'loading' }
  | { status: 'missing' }
  | {
      status: 'found';
      /** Null until the NFT's metadata has been downloaded. */
      name: string | null;
      kind: NftUpdaterKind;
      /**
       * Null until the wallet has downloaded the NFT's data, which it does
       * for a looked-up NFT as it does for its own.
       */
      thumbnail: string | null;
    };

/**
 * Looks up an NFT the user is requesting, so they can see they typed the
 * right ID before making the offer. The lookup goes to the network, the same
 * way the offer itself fetches a requested NFT.
 */
export function useRequestedNft(nftId: string): NftLookup {
  const trimmed = nftId.trim();
  const debounced = useDebounce(trimmed, 400);
  const [lookup, setLookup] = useState<NftLookup>({ status: 'idle' });

  useEffect(() => {
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

    commands
      .getNftMetadataUpdater({ nft_id: debounced })
      .then(({ metadata_updater_kind: kind, name }) => {
        if (cancelled) return;
        setLookup(
          kind === null
            ? { status: 'missing' }
            : { status: 'found', name, kind, thumbnail: null },
        );
      })
      .catch(() => {
        if (!cancelled) setLookup({ status: 'missing' });
      });

    return () => {
      cancelled = true;
    };
  }, [debounced]);

  // Looking the NFT up saves it, and the wallet then downloads its data and
  // makes a thumbnail in the background. This refreshes as that finishes.
  const thumbnails = useNftThumbnails(
    lookup.status === 'found' ? [debounced] : [],
  );

  // An empty or malformed ID needs no lookup, so it's shown straight away
  // rather than after the debounce. A valid ID that's still being typed
  // hasn't been checked yet.
  if (trimmed !== debounced) {
    if (!trimmed) return { status: 'idle' };
    if (!isValidAddress(trimmed, 'nft')) return { status: 'invalid' };
    return { status: 'loading' };
  }

  if (lookup.status === 'found') {
    return { ...lookup, thumbnail: thumbnails[debounced] ?? null };
  }

  return lookup;
}
