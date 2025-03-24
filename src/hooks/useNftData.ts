import { isValidAddress } from '@/lib/utils';
import { useCallback, useEffect, useState } from 'react';
import {
  commands,
  DidRecord,
  events,
  NftCollectionRecord,
  NftRecord,
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
  ownerDids: DidRecord[];
  minterDids: DidRecord[];
  owner: DidRecord | null;
  collection: NftCollectionRecord | null;
  isLoading: boolean;
  nftTotal: number;
  collectionTotal: number;
  ownerDidsTotal: number;
  minterDidsTotal: number;
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

  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [ownerDids, setOwnerDids] = useState<DidRecord[]>([]);
  const [minterDids, setMinterDids] = useState<DidRecord[]>([]);
  const [owner, setOwner] = useState<DidRecord | null>(null);
  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(false);
  const [nftTotal, setNftTotal] = useState(0);
  const [collectionTotal, setCollectionTotal] = useState(0);
  const [ownerDidsTotal, setOwnerDidsTotal] = useState(0);
  const [minterDidsTotal, setMinterDidsTotal] = useState(0);

  const updateNfts = useCallback(
    async (page: number) => {
      setIsLoading(true);
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

          let nfts: NftRecord[] = [];
          let total: number = 0;
          if (params.query && isValidAddress(params.query, 'nft')) {
            const response = await commands.getNft({ nft_id: params.query });
            nfts = response.nft ? [response.nft] : [];
            total = nfts.length;
          } else {
            const response = await commands.getNfts(queryParams);
            nfts = response.nfts;
            total = response.total;
          }

          setNfts(nfts);
          setNftTotal(total);

          if (params.collectionId) {
            const collectionResponse = await commands.getNftCollection({
              collection_id:
                params.collectionId === 'No collection'
                  ? null
                  : params.collectionId,
            });
            setCollection(collectionResponse.collection);
          } else if (params.ownerDid) {
            const didResponse = await commands.getDids({});
            const foundDid = didResponse.dids.find(
              (did) => did.launcher_id === params.ownerDid,
            );
            setOwner(
              foundDid || createDefaultDidRecord('Unassigned NFTs', 'No did'),
            );
          } else if (params.minterDid) {
            setOwner(
              createDefaultDidRecord(params.minterDid, params.minterDid),
            );
          }
        } else if (params.group === NftGroupMode.Collection) {
          try {
            const response = await commands.getNftCollections({
              offset: (page - 1) * params.pageSize,
              limit: params.pageSize,
              include_hidden: params.showHidden,
            });

            const collections = response.collections;

            // Add No Collection to the end if we're on the last page and there's room
            if (
              collections.length < params.pageSize &&
              page === Math.ceil((response.total + 1) / params.pageSize)
            ) {
              collections.push({
                name: 'No Collection',
                icon: '',
                did_id: 'Miscellaneous',
                metadata_collection_id: 'Uncategorized NFTs',
                collection_id: 'No collection',
                visible: true,
              });
            }

            setCollections(collections);
            setCollectionTotal(response.total + 1); // Add 1 for No Collection
          } catch (error: any) {
            setCollections([]);
            setCollectionTotal(0);
            addError(error);
          }
        } else if (params.group === NftGroupMode.OwnerDid) {
          try {
            const response = await commands.getDids({});
            const ownerDids = response.dids;

            // Add Unassigned NFTs to the end if there's room on the last page
            if (
              ownerDids.length < params.pageSize &&
              page === Math.ceil((ownerDids.length + 1) / params.pageSize)
            ) {
              ownerDids.push(
                createDefaultDidRecord('Unassigned NFTs', 'No did'),
              );
            }

            setOwnerDids(ownerDids);
            setOwnerDidsTotal(response.dids.length + 1); // Add 1 for Unassigned NFTs
          } catch (error: any) {
            setOwnerDids([]);
            setOwnerDidsTotal(0);
            addError(error);
          }
        } else if (params.group === NftGroupMode.MinterDid) {
          try {
            const uniqueMinterDids = await commands.getMinterDidIds({
              limit: params.pageSize,
              offset: (page - 1) * params.pageSize,
            });

            const minterDids: DidRecord[] = uniqueMinterDids.did_ids.map(
              (did) =>
                createDefaultDidRecord(
                  `${did.replace('did:chia:', '').slice(0, 16)}...`,
                  did,
                ),
            );

            // Add Unknown Minter to the end of the list if we're on the last page
            if (
              minterDids.length < params.pageSize &&
              page === Math.ceil((uniqueMinterDids.total + 1) / params.pageSize)
            ) {
              minterDids.push(
                createDefaultDidRecord('Unknown Minter', 'No did'),
              );
            }

            setMinterDids(minterDids);
            setMinterDidsTotal(uniqueMinterDids.total + 1); // Add 1 for Unknown Minter
          } catch (error: any) {
            setMinterDids([]);
            setMinterDidsTotal(0);
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
    setNfts([]);
    setCollections([]);
    setOwnerDids([]);
    setMinterDids([]);
    setCollection(null);
    setOwner(null);
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

  // Helper function to get the correct total based on current view
  const getTotal = useCallback(() => {
    if (
      params.collectionId ||
      params.ownerDid ||
      params.minterDid ||
      params.group === NftGroupMode.None
    ) {
      return nftTotal;
    } else if (params.group === NftGroupMode.Collection) {
      return collectionTotal;
    } else if (params.group === NftGroupMode.OwnerDid) {
      return ownerDidsTotal;
    } else if (params.group === NftGroupMode.MinterDid) {
      return minterDidsTotal;
    }
    return 0;
  }, [
    params.collectionId,
    params.ownerDid,
    params.minterDid,
    params.group,
    nftTotal,
    collectionTotal,
    ownerDidsTotal,
    minterDidsTotal,
  ]);

  return {
    nfts,
    collections,
    ownerDids,
    minterDids,
    owner,
    collection,
    isLoading,
    total: getTotal(),
    updateNfts,
  };
}
