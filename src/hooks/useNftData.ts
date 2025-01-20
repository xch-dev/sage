import { useCallback, useEffect, useState } from 'react';
import {
  commands,
  events,
  NftCollectionRecord,
  NftRecord,
  DidRecord,
} from '../bindings';
import { useErrors } from './useErrors';
import { NftGroupMode, NftSortMode } from './useNftParams';

interface NftDataParams {
  pageSize: number;
  sort: NftSortMode;
  group: NftGroupMode;
  showHidden: boolean;
  query: string | null;
  collectionId?: string;
  ownerDid?: string;
  minterDid?: string;
  page: number;
}

interface NftDataState {
  nfts: NftRecord[];
  collections: NftCollectionRecord[];
  dids: DidRecord[];
  owner: DidRecord | null;
  collection: NftCollectionRecord | null;
  isLoading: boolean;
}

// Helper function moved from NftList
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

export function useNftData(params: NftDataParams) {
  const { addError } = useErrors();
  const [state, setState] = useState<NftDataState>({
    nfts: [],
    collections: [],
    dids: [],
    owner: null,
    collection: null,
    isLoading: false,
  });

  const updateNfts = useCallback(
    async (page: number) => {
      setState((prev) => ({ ...prev, isLoading: true }));
      try {
        if (
          params.collectionId ||
          params.ownerDid ||
          params.minterDid ||
          params.group === NftGroupMode.None
        ) {
          const queryParams = {
            name: params.query || null,
            collection_id:
              params.collectionId === 'No collection'
                ? 'none'
                : (params.collectionId ?? null),
            owner_did_id:
              params.ownerDid === 'No did' ? 'none' : (params.ownerDid ?? null),
            minter_did_id:
              params.minterDid === 'No did'
                ? 'none'
                : (params.minterDid ?? null),
            offset: (page - 1) * params.pageSize,
            limit: params.pageSize,
            sort_mode: params.sort,
            include_hidden: params.showHidden,
          };

          const response = await commands.getNfts(queryParams);

          const updates: Partial<NftDataState> = {
            nfts: response.nfts,
          };

          if (params.collectionId) {
            const collectionResponse = await commands.getNftCollection({
              collection_id:
                params.collectionId === 'No collection'
                  ? null
                  : params.collectionId,
            });
            updates.collection = collectionResponse.collection;
          } else if (params.ownerDid) {
            const didResponse = await commands.getDids({});
            const foundDid = didResponse.dids.find(
              (did) => did.launcher_id === params.ownerDid,
            );
            updates.owner =
              foundDid || createDefaultDidRecord('Unassigned NFTs', 'No did');
          } else if (params.minterDid) {
            updates.owner = createDefaultDidRecord(
              params.minterDid,
              params.minterDid,
            );
          }

          setState((prev) => ({ ...prev, ...updates }));
        } else if (params.group === NftGroupMode.Collection) {
          try {
            const response = await commands.getNftCollections({
              offset: (page - 1) * params.pageSize,
              limit: params.pageSize,
              include_hidden: params.showHidden,
            });
            setState((prev) => ({
              ...prev,
              collections: response.collections,
            }));
          } catch (error: any) {
            setState((prev) => ({ ...prev, collections: [] }));
            addError(error);
          }
        } else if (params.group === NftGroupMode.OwnerDid) {
          try {
            const response = await commands.getDids({});
            setState((prev) => ({ ...prev, dids: response.dids }));
          } catch (error: any) {
            setState((prev) => ({ ...prev, dids: [] }));
            addError(error);
          }
        } else if (params.group === NftGroupMode.MinterDid) {
          try {
            const uniqueMinterDids = await commands.getMinterDidIds({});
            const minterDids: DidRecord[] = uniqueMinterDids.did_ids.map(
              (did) =>
                createDefaultDidRecord(
                  `${did.replace('did:chia:', '').slice(0, 16)}...`,
                  did,
                ),
            );
            setState((prev) => ({ ...prev, dids: minterDids }));
          } catch (error: any) {
            setState((prev) => ({ ...prev, dids: [] }));
            addError(error);
          }
        }
      } catch (error: any) {
        console.error('Error fetching NFTs:', error);
        addError(error);
      } finally {
        setState((prev) => ({ ...prev, isLoading: false }));
      }
    },
    [
      params.pageSize,
      params.showHidden,
      params.sort,
      params.group,
      params.query,
      params.collectionId,
      params.ownerDid,
      params.minterDid,
      addError,
    ],
  );

  // Clear state and fetch new data when params change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      nfts: [],
      collections: [],
      dids: [],
      collection: null,
      owner: null,
    }));
    updateNfts(params.page);
  }, [
    updateNfts,
    params.collectionId,
    params.ownerDid,
    params.minterDid,
    params.page,
  ]);

  // Listen for sync events
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

  return {
    ...state,
    updateNfts,
  };
}
