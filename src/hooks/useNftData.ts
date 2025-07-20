import { CustomError } from '@/contexts/ErrorContext';
import { queryNfts } from '@/lib/exportNfts';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
  commands,
  DidRecord,
  events,
  NftCollectionRecord,
  NftRecord,
} from '../bindings';
import { useDids } from './useDids';
import { useErrors } from './useErrors';
import { NftGroupMode, NftSortMode } from './useNftParams';

export const NO_COLLECTION_ID =
  'col1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq6rdgel';

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
    recovery_hash: '',
  };
}

export function useNftData(params: NftDataParams) {
  const { addError } = useErrors();
  const { dids } = useDids();

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

  // Use ref to store the latest updateNfts function to avoid circular dependencies
  const updateNftsRef = useRef<((page: number) => Promise<void>) | null>(null);

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
          // Use the shared queryNfts function for paginated results
          const queryParams = {
            sort: params.sort,
            group: params.group,
            showHidden: params.showHidden,
            query: params.query,
            collectionId: params.collectionId,
            ownerDid: params.ownerDid,
            minterDid: params.minterDid,
          };

          const { nfts, total } = await queryNfts(
            queryParams,
            params.pageSize,
            (page - 1) * params.pageSize,
          );

          setNfts(nfts);
          setNftTotal(total);

          if (params.collectionId) {
            const collectionResponse = await commands.getNftCollection({
              collection_id:
                params.collectionId === NO_COLLECTION_ID
                  ? null
                  : params.collectionId,
            });
            setCollection(collectionResponse.collection);
          } else if (params.ownerDid) {
            // Move dids dependency out of this callback - handled in separate useEffect
            setOwner(createDefaultDidRecord('Loading...', params.ownerDid));
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

            setCollections(collections);
            setCollectionTotal(response.total);
          } catch (error: unknown) {
            setCollections([]);
            setCollectionTotal(0);
            addError(error as CustomError);
          }
        } else if (params.group === NftGroupMode.OwnerDid) {
          // This will be handled by the separate useEffect for dids
          setOwnerDids([]);
          setOwnerDidsTotal(0);
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
          } catch (error: unknown) {
            setMinterDids([]);
            setMinterDidsTotal(0);
            addError(error as CustomError);
          }
        }
      } catch (error: unknown) {
        console.error('Error fetching NFTs:', error);
        addError(error as CustomError);
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

  // Store the latest updateNfts function in ref
  updateNftsRef.current = updateNfts;

  // Clear state and fetch new data when params change
  useEffect(() => {
    setNfts([]);
    setCollections([]);
    setOwnerDids([]);
    setMinterDids([]);
    setCollection(null);
    setOwner(null);
    // Use ref to avoid adding updateNfts to deps (would cause infinite loop)
    if (updateNftsRef.current) {
      updateNftsRef.current(params.page);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    params.collectionId,
    params.ownerDid,
    params.minterDid,
    params.page,
    params.pageSize,
    params.showHidden,
    params.sort,
    params.group,
    params.query,
  ]); // updateNfts not included to prevent infinite loop (stored in ref)

  // Listen for sync events - use ref to avoid circular dependency
  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'nft_data'
      ) {
        // Use the ref to get the latest function without causing re-renders
        if (updateNftsRef.current) {
          updateNftsRef.current(params.page);
        }
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params.page]); // Only depend on params.page, not updateNfts (stored in ref to avoid infinite loop)

  // Update when DIDs change if we're showing owner DIDs
  useEffect(() => {
    if (params.group === NftGroupMode.OwnerDid) {
      const ownerDids = [...dids];
      // Add Unassigned NFTs to the end if there's room on the last page
      if (
        ownerDids.length < params.pageSize &&
        params.page === Math.ceil((ownerDids.length + 1) / params.pageSize)
      ) {
        ownerDids.push(createDefaultDidRecord('Unassigned NFTs', 'No did'));
      }
      setOwnerDids(ownerDids);
      setOwnerDidsTotal(dids.length + 1);
    } else if (params.ownerDid) {
      const foundDid = dids.find((did) => did.launcher_id === params.ownerDid);
      setOwner(foundDid || createDefaultDidRecord('Unassigned NFTs', 'No did'));
    }
  }, [dids, params.group, params.ownerDid, params.page, params.pageSize]);

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
