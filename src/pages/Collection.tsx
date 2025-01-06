import { commands, events, NftCollectionRecord, NftRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { MultiSelectActions } from '@/components/MultiSelectActions';
import { NftCard, NftCardList } from '@/components/NftCard';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { useErrors } from '@/hooks/useErrors';
import { NftView, useNftParams } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export default function Collection() {
  const { addError } = useErrors();
  const { collection_id: collectionId } = useParams();

  const [params, setParams] = useNftParams();
  const { pageSize, page, view, showHidden, query } = params;

  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [multiSelect, setMultiSelect] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const updateNfts = useCallback(
    async (page: number) => {
      if (view === NftView.Collection) return;

      setIsLoading(true);
      try {
        await commands
          .getNfts({
            collection_id:
              collectionId === 'No collection'
                ? 'none'
                : (collectionId ?? null),
            did_id: null,
            name: query || null,
            offset: (page - 1) * pageSize,
            limit: pageSize,
            sort_mode: view,
            include_hidden: showHidden,
          })
          .then((data) => setNfts(data.nfts))
          .catch(addError);

        await commands
          .getNftCollection({
            collection_id:
              collectionId === 'No collection' ? null : (collectionId ?? null),
          })
          .then((data) => setCollection(data.collection))
          .catch(addError);
      } finally {
        setIsLoading(false);
      }
    },
    [collectionId, pageSize, showHidden, view, query, addError],
  );

  useEffect(() => {
    updateNfts(1);
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

  return (
    <>
      <Header title={`${collection?.name ?? t`Unknown`} NFTs`}>
        <ReceiveAddress />
      </Header>

      <Container>
        <NftOptions
          isCollection
          params={params}
          setParams={setParams}
          multiSelect={multiSelect}
          setMultiSelect={(value) => {
            setMultiSelect(value);
            setSelected([]);
          }}
          isLoading={isLoading}
        />

        <NftCardList>
          {nfts.map((nft, i) => (
            <NftCard
              nft={nft}
              key={i}
              updateNfts={() => updateNfts(page)}
              selectionState={
                multiSelect
                  ? [
                      selected.includes(nft.launcher_id),
                      (value) => {
                        if (value && !selected.includes(nft.launcher_id)) {
                          setSelected([...selected, nft.launcher_id]);
                        } else if (
                          !value &&
                          selected.includes(nft.launcher_id)
                        ) {
                          setSelected(
                            selected.filter((id) => id !== nft.launcher_id),
                          );
                        }
                      },
                    ]
                  : null
              }
            />
          ))}
        </NftCardList>
      </Container>

      {selected.length > 0 && (
        <MultiSelectActions
          selected={selected}
          onConfirm={() => {
            setSelected([]);
            setMultiSelect(false);
          }}
        />
      )}
    </>
  );
}
