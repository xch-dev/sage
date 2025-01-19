import { NftCard } from '@/components/NftCard';
import { NftGroupCard } from '@/components/NftGroupCard';
import { NftGroupMode } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { ReactNode } from 'react';
import {
  NftCollectionRecord,
  NftRecord,
  DidRecord,
  commands,
  Error,
} from '../bindings';

interface NftCardListProps {
  collectionId?: string;
  ownerDid?: string;
  minterDid?: string;
  group?: NftGroupMode;
  nfts: NftRecord[];
  collections: NftCollectionRecord[];
  dids: DidRecord[];
  pageSize: number;
  updateNfts: (page: number) => void;
  page: number;
  multiSelect?: boolean;
  selected?: string[];
  setSelected?: (selected: string[]) => void;
  addError?: (error: Error) => void;
  children?: ReactNode;
}

export function NftCardList({
  collectionId,
  ownerDid,
  minterDid,
  group,
  nfts,
  collections,
  dids,
  pageSize,
  updateNfts,
  page,
  multiSelect = false,
  selected = [],
  setSelected,
  addError,
  children,
}: NftCardListProps) {
  const renderContent = () => {
    if (
      !collectionId &&
      !ownerDid &&
      !minterDid &&
      group === NftGroupMode.Collection
    ) {
      return (
        <>
          {collections.map((col, i) => (
            <NftGroupCard
              key={i}
              type='collection'
              item={col}
              updateNfts={updateNfts}
              page={page}
              onToggleVisibility={() => {
                commands
                  .updateNft({
                    nft_id: col.collection_id,
                    visible: !col.visible,
                  })
                  .then(() => updateNfts(page))
                  .catch(addError);
              }}
            />
          ))}
          {nfts.length < pageSize && (
            <NftGroupCard
              type='collection'
              item={{
                name: t`No Collection`,
                icon: '',
                did_id: 'Miscellaneous',
                metadata_collection_id: 'Uncategorized NFTs',
                collection_id: 'No collection',
                visible: true,
              }}
              updateNfts={updateNfts}
              page={page}
            />
          )}
        </>
      );
    }

    if (
      !collectionId &&
      !ownerDid &&
      !minterDid &&
      (group === NftGroupMode.OwnerDid || group === NftGroupMode.MinterDid)
    ) {
      return (
        <>
          {dids.map((did, i) => (
            <NftGroupCard
              key={i}
              type='did'
              groupMode={group}
              item={did}
              updateNfts={updateNfts}
              page={page}
            />
          ))}
          <NftGroupCard
            type='did'
            groupMode={group}
            item={{
              name:
                group === NftGroupMode.OwnerDid
                  ? t`Unassigned NFTs`
                  : t`Unknown Minter`,
              launcher_id: 'No did',
              visible: true,
              coin_id: 'No coin',
              address: 'No address',
              amount: 0,
              created_height: 0,
              create_transaction_id: 'No transaction',
              recovery_hash: '',
            }}
            updateNfts={updateNfts}
            page={page}
          />
        </>
      );
    }

    return nfts.map((nft, i) => (
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
                    setSelected?.([...selected, nft.launcher_id]);
                  } else if (!value && selected.includes(nft.launcher_id)) {
                    setSelected?.(
                      selected.filter((id) => id !== nft.launcher_id),
                    );
                  }
                },
              ]
            : null
        }
      />
    ));
  };

  return (
    <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-6 mb-2'>
      {renderContent()}
      {children}
    </div>
  );
}
