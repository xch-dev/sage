import Container from '@/components/Container';
import Header from '@/components/Header';
import { MultiSelectActions } from '@/components/MultiSelectActions';
import { NftPageTitle } from '@/components/NftPageTitle';
import { NftCardList } from '@/components/NftCardList';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { useNftParams, NftGroupMode } from '@/hooks/useNftParams';
import { Trans } from '@lingui/react/macro';
import { ImagePlusIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  events,
  NftCollectionRecord,
  NftRecord,
  DidRecord,
} from '../bindings';

// Add this helper function before the NftList component
function createDefaultDidRecord(name: string, launcherId: string): DidRecord {
  return {
    name,
    launcher_id: launcherId,
    visible: true,
    coin_id: 'No coin',
    address: 'No address',
    amount: 0,
    created_height: 0,
    create_transaction_id: 'No transaction',
    recovery_hash: '',
  };
}

export function NftList() {
  const navigate = useNavigate();
  const {
    collection_id: collectionId,
    owner_did: ownerDid,
    minter_did: minterDid,
  } = useParams();
  const { addError } = useErrors();
  const [params, setParams] = useNftParams();
  const { pageSize, sort, group, showHidden, query } = params;
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const [owner, setOwner] = useState<DidRecord | null>(null);
  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [multiSelect, setMultiSelect] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const updateNfts = useCallback(
    async (page: number) => {
      setIsLoading(true);
      try {
        if (
          collectionId ||
          ownerDid ||
          minterDid ||
          group === NftGroupMode.None
        ) {
          // the queries in rust differentiate 'none' from null
          // - 'none' means unassigned nfts
          // - null means all nfts for the given group
          const params = {
            name: query || null,
            collection_id:
              collectionId === 'No collection'
                ? 'none'
                : (collectionId ?? null),
            owner_did_id: ownerDid === 'No did' ? 'none' : (ownerDid ?? null),
            minter_did_id:
              minterDid === 'No did' ? 'none' : (minterDid ?? null),
            offset: (page - 1) * pageSize,
            limit: pageSize,
            sort_mode: sort,
            include_hidden: showHidden,
          };

          const response = await commands.getNfts(params);
          setNfts(response.nfts);

          if (collectionId) {
            const collectionResponse = await commands.getNftCollection({
              collection_id:
                collectionId === 'No collection' ? null : collectionId,
            });
            setCollection(collectionResponse.collection);
          } else if (ownerDid) {
            const didResponse = await commands.getDids({});
            const foundDid = didResponse.dids.find(
              (did) => did.launcher_id === ownerDid,
            );
            setOwner(
              foundDid || createDefaultDidRecord('Unassigned NFTs', 'No did')
            );
          } else if (minterDid) {
            setOwner(createDefaultDidRecord(minterDid, minterDid));
          }
        } else if (group === NftGroupMode.Collection) {
          try {
            const response = await commands.getNftCollections({
              offset: (page - 1) * pageSize,
              limit: pageSize,
              include_hidden: showHidden,
            });
            setCollections(response.collections);
          } catch (error: any) {
            setCollections([]);
            addError(error);
          }
        } else if (group === NftGroupMode.OwnerDid) {
          try {
            const response = await commands.getDids({});
            setDids(response.dids);
          } catch (error: any) {
            setDids([]);
            addError(error);
          }
        } else if (group === NftGroupMode.MinterDid) {
          try {
            const uniqueMinterDids = await commands.getMinterDidIds({});
            const minterDids: DidRecord[] = uniqueMinterDids.did_ids.map(
              (did) => createDefaultDidRecord(
                `${did.replace('did:chia:', '').slice(0, 16)}...`,
                did
              )
            );
            setDids(minterDids);
          } catch (error: any) {
            setDids([]);
            addError(error);
          }
        }
      } catch (error: any) {
        console.error('Error fetching NFTs:', error);
        addError(error);
      } finally {
        setIsLoading(false);
      }
    },
    [
      pageSize,
      showHidden,
      sort,
      group,
      query,
      collectionId,
      ownerDid,
      minterDid,
      addError,
    ],
  );

  useEffect(() => {
    // Clear all state when view parameters change
    setNfts([]);
    setCollections([]);
    setDids([]);
    setCollection(null);
    setOwner(null);
    updateNfts(params.page);
  }, [updateNfts, collectionId, ownerDid, minterDid, params.page]);

  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'nft_data'
      ) {
        updateNfts(params.page);
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateNfts, params.page]);

  // Add this effect to reset multi-select when route changes
  useEffect(() => {
    setMultiSelect(false);
    setSelected([]);
  }, [collectionId, ownerDid, group]);

  const canLoadMore = useCallback(() => {
    if (collectionId || ownerDid || minterDid || group === NftGroupMode.None) {
      return nfts.length === pageSize;
    } else if (group === NftGroupMode.Collection) {
      return collections.length === pageSize;
    } else if (
      group === NftGroupMode.OwnerDid ||
      group === NftGroupMode.MinterDid
    ) {
      return dids.length === pageSize;
    }
    return false;
  }, [
    collectionId,
    ownerDid,
    minterDid,
    group,
    nfts.length,
    collections.length,
    dids.length,
    pageSize,
  ]);

  return (
    <>
      <Header
        title={
          <NftPageTitle
            collectionId={collectionId}
            collection={collection}
            ownerDid={ownerDid}
            owner={owner}
            minterDid={minterDid}
            group={group}
          />
        }
      >
        <ReceiveAddress />
      </Header>

      <Container>
        <Button onClick={() => navigate('/nfts/mint')}>
          <ImagePlusIcon className='h-4 w-4 mr-2' aria-hidden={true} /> <Trans>Mint NFT</Trans>
        </Button>

        <NftOptions
          params={params}
          setParams={setParams}
          multiSelect={multiSelect}
          setMultiSelect={(value) => {
            setMultiSelect(value);
            setSelected([]);
          }}
          className='mt-4'
          isLoading={isLoading}
          canLoadMore={canLoadMore()}
        />

        <NftCardList
          collectionId={collectionId}
          ownerDid={ownerDid}
          minterDid={minterDid}
          group={group}
          nfts={nfts}
          collections={collections}
          dids={dids}
          pageSize={pageSize}
          updateNfts={updateNfts}
          page={params.page}
          multiSelect={multiSelect}
          selected={selected}
          setSelected={setSelected}
          addError={addError}
        />
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
