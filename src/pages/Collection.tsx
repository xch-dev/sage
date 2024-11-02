import { commands, events, NftCollectionRecord, NftRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { NftCard, NftCardList } from '@/components/NftCard';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { NftView, useNftParams } from '@/hooks/useNftParams';
import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export default function Collection() {
  const { collection_id: collectionId } = useParams();

  const [params, setParams] = useNftParams();
  const { pageSize, page, view, showHidden } = params;

  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [nfts, setNfts] = useState<NftRecord[]>([]);

  const updateNfts = useCallback(
    async (page: number) => {
      if (view === NftView.Collection) return;

      await commands
        .getCollectionNfts({
          collection_id:
            collectionId === 'none' ? null : (collectionId ?? null),
          offset: (page - 1) * pageSize,
          limit: pageSize,
          sort_mode: view,
          include_hidden: showHidden,
        })
        .then((result) => {
          if (result.status === 'ok') {
            setNfts(result.data);
          } else {
            throw new Error('Failed to get NFTs');
          }
        });

      await commands
        .getNftCollection(
          collectionId === 'none' ? null : (collectionId ?? null),
        )
        .then((result) => {
          if (result.status === 'ok') {
            setCollection(result.data);
          } else {
            throw new Error('Failed to get collection');
          }
        });
    },
    [collectionId, pageSize, showHidden, view],
  );

  useEffect(() => {
    updateNfts(0);
  }, [updateNfts]);

  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'nft_data'
      ) {
        updateNfts(page);
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateNfts, page]);

  useEffect(() => {
    updateNfts(page);
  }, [updateNfts, page]);

  const totalPages = Math.max(
    1,
    Math.ceil(
      ((showHidden ? collection?.nfts : collection?.visible_nfts) ?? 0) /
        pageSize,
    ),
  );

  return (
    <>
      <Header title={`${collection?.name ?? 'Unknown'} NFTs`}>
        <ReceiveAddress />
      </Header>

      <Container>
        <NftOptions
          totalPages={totalPages}
          params={params}
          setParams={setParams}
        />

        <NftCardList>
          {nfts.map((nft, i) => (
            <NftCard nft={nft} key={i} updateNfts={() => updateNfts(page)} />
          ))}
        </NftCardList>
      </Container>
    </>
  );
}
