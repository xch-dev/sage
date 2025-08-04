import { getMintGardenProfile } from '@/lib/marketplaces';
import { getAssetDisplayName } from '@/lib/utils';
import { useEffect, useMemo, useState } from 'react';
import { AssetKind, commands, DidRecord, events } from '../bindings';

export interface UseDidDataParams {
  did: DidRecord | string;
}

export interface MintGardenProfile {
  encoded_id: string;
  name: string;
  avatar_uri: string | null;
  is_unknown: boolean;
}

export interface DidAsset {
  icon_url: string | null;
  kind: AssetKind;
  revocation_address: null;
  name: string;
  ticker: string;
  precision: number;
  asset_id: string;
  balance: string;
  balanceInUsd: string;
  priceInUsd: string;
}

export function useDidData({ did: inputDid }: UseDidDataParams) {
  const [fetchedDid, setFetchedDid] = useState<DidRecord | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [mintGardenProfile, setMintGardenProfile] =
    useState<MintGardenProfile | null>(null);
  const [isMintGardenLoading, setIsMintGardenLoading] = useState(false);

  // Determine if this is an owned DID and create the final DidRecord
  const isOwned = typeof inputDid !== 'string';
  const launcherId =
    typeof inputDid === 'string' ? inputDid : inputDid.launcher_id;

  const didRecord: DidRecord = useMemo(() => {
    if (isOwned) {
      return inputDid as DidRecord;
    }

    // Use fetched data if available, otherwise create fallback
    if (fetchedDid) {
      return fetchedDid;
    }

    // Fallback for string launcher_id when no fetched data
    return {
      launcher_id: inputDid as string,
      name: `${(inputDid as string).slice(9, 19)}...${(inputDid as string).slice(-4)}`,
      visible: true,
      created_height: null,
      recovery_hash: null,
      coin_id: '0',
      address: '',
      amount: 0,
    };
  }, [inputDid, fetchedDid, isOwned]);

  const updateDid = useMemo(
    () => () => {
      if (!launcherId || isOwned) return; // Don't fetch if we already have owned DID data

      setIsLoading(true);
      commands
        .getProfile({ launcher_id: launcherId })
        .then((data) => setFetchedDid(data.did))
        .catch(() => {
          // On error, we'll use the fallback DidRecord created in the memo above
          // Don't call addError here to avoid showing errors for non-owned DIDs
        })
        .finally(() => setIsLoading(false));
    },
    [launcherId, isOwned],
  );

  // Fetch MintGarden profile data
  useEffect(() => {
    if (!didRecord?.launcher_id) return;

    setIsMintGardenLoading(true);
    getMintGardenProfile(didRecord.launcher_id)
      .then((profileData) => {
        setMintGardenProfile(profileData);
      })
      .catch(() => {
        // Create fallback profile for failed lookups
        setMintGardenProfile({
          encoded_id: didRecord.launcher_id,
          name: didRecord.name ?? '',
          avatar_uri: null,
          is_unknown: true,
        });
      })
      .finally(() => {
        setIsMintGardenLoading(false);
      });
  }, [didRecord?.launcher_id, didRecord?.name]);

  // Create DID asset object
  const didAsset: DidAsset | null = useMemo(() => {
    if (!didRecord || !mintGardenProfile) return null;

    return {
      icon_url: mintGardenProfile.avatar_uri,
      kind: 'did' as AssetKind,
      revocation_address: null,
      name: !mintGardenProfile.is_unknown
        ? mintGardenProfile.name
        : getAssetDisplayName(
            didRecord.name || mintGardenProfile.name,
            null,
            'did',
          ),
      ticker: '',
      precision: 0,
      asset_id: didRecord.launcher_id,
      balance: '0',
      balanceInUsd: '0',
      priceInUsd: '0',
    };
  }, [didRecord, mintGardenProfile]);

  useEffect(() => {
    updateDid();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'did_info'
      ) {
        updateDid();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateDid]);

  return {
    did: didRecord,
    isLoading,
    updateDid,
    mintGardenProfile,
    isMintGardenLoading,
    didAsset,
    isOwned,
  };
}
