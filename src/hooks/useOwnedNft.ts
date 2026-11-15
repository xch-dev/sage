import { NftRecord, commands, events } from '@/bindings';
import { nftUri } from '@/lib/nftUri';
import { useCallback, useEffect, useMemo, useState } from 'react';

/** Thumbnails for the given NFTs, keyed by ID, refreshed as NFT data syncs. */
export function useNftThumbnails(nftIds: string[]) {
  const [thumbnails, setThumbnails] = useState<Record<string, string>>({});
  // A stable key, so a new array with the same IDs doesn't refetch.
  const key = nftIds.join(',');
  const ids = useMemo(() => (key ? key.split(',') : []), [key]);

  const update = useCallback(() => {
    let cancelled = false;

    Promise.all(
      ids.map((id) =>
        commands
          .getNftThumbnail({ nft_id: id })
          .then(({ thumbnail }) => [id, thumbnail] as const)
          .catch(() => [id, null] as const),
      ),
    ).then((results) => {
      if (cancelled) return;
      const map: Record<string, string> = {};
      for (const [id, thumbnail] of results) {
        if (thumbnail !== null) map[id] = nftUri('image/png', thumbnail);
      }
      setThumbnails(map);
    });

    return () => {
      cancelled = true;
    };
  }, [ids]);

  useEffect(() => {
    let cancelUpdate = update();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'nft_data') {
        cancelUpdate();
        cancelUpdate = update();
      }
    });

    return () => {
      cancelUpdate();
      unlisten.then((u) => u());
    };
  }, [update]);

  return thumbnails;
}

/** An NFT from this wallet, with its thumbnail. */
export function useOwnedNft(nftId: string) {
  const [nft, setNft] = useState<NftRecord | null>(null);
  const thumbnails = useNftThumbnails(nftId ? [nftId] : []);

  useEffect(() => {
    if (!nftId) return;
    let cancelled = false;

    commands
      .getNft({ nft_id: nftId })
      .then((response) => {
        if (!cancelled) setNft(response.nft);
      })
      .catch(() => {
        if (!cancelled) setNft(null);
      });

    return () => {
      cancelled = true;
    };
  }, [nftId]);

  return {
    nft: nft?.launcher_id === nftId ? nft : null,
    thumbnail: thumbnails[nftId] ?? null,
  };
}
