import { commands, NftUpdaterKind } from '@/bindings';
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
      /** Null until the NFT's metadata has been downloaded. */
      name: string | null;
      kind: NftUpdaterKind;
      /** Null while it loads, or if there isn't one. */
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
        if (kind === null) {
          setLookup({ status: 'missing' });
          return;
        }

        setLookup({ status: 'found', name, kind, thumbnail: null });

        // The thumbnail can take a while to come from MintGarden, so it's
        // loaded once the NFT is found rather than holding that up. It needs
        // the NFT's info, which the lookup above saves to the wallet.
        commands
          .getRequestedNftThumbnail({ nft_id: debounced })
          .then(({ thumbnail }) => {
            if (cancelled || thumbnail === null) return;
            setLookup((current) =>
              current.status === 'found'
                ? { ...current, thumbnail: nftUri('image/png', thumbnail) }
                : current,
            );
          })
          // Without a thumbnail, the preview keeps its placeholder.
          .catch(() => undefined);
      })
      .catch(() => {
        if (!cancelled) setLookup({ status: 'missing' });
      });

    return () => {
      cancelled = true;
    };
  }, [debounced]);

  // An empty or malformed ID needs no lookup, so it's shown straight away
  // rather than after the debounce. A valid ID that's still being typed
  // hasn't been checked yet.
  if (trimmed !== debounced) {
    if (!trimmed) return { status: 'idle' };
    if (!isValidAddress(trimmed, 'nft')) return { status: 'invalid' };
    return { status: 'loading' };
  }

  return lookup;
}
