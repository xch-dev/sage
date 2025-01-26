import { NftCard } from '@/components/NftCard';
import { NftGroupCard } from '@/components/NftGroupCard';
import { NftGroupMode } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { ReactNode, useCallback } from 'react';
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
  ownerDids: DidRecord[];
  minterDids: DidRecord[];
  updateNfts: (page: number) => void;
  page: number;
  multiSelect?: boolean;
  selected?: string[];
  setSelected?: React.Dispatch<React.SetStateAction<string[]>>;
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
  ownerDids,
  minterDids,
  updateNfts,
  page,
  multiSelect = false,
  selected = [],
  setSelected,
  addError,
  children,
}: NftCardListProps) {
  const handleSelection = useCallback(
    (id: string) => {
      if (!multiSelect || !setSelected) return;

      setSelected((prev: string[]) => {
        const newSelected = [...prev];
        const index = newSelected.indexOf(id);

        if (index === -1) {
          newSelected.push(id);
        } else {
          newSelected.splice(index, 1);
        }

        return newSelected;
      });
    },
    [multiSelect, setSelected],
  );

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
              canToggleVisibility={col.collection_id !== 'No collection'}
              onToggleVisibility={() => {
                commands
                  .updateNftCollection({
                    collection_id: col.collection_id,
                    visible: !col.visible,
                  })
                  .then(() => updateNfts(page))
                  .catch(addError);
              }}
            />
          ))}
        </>
      );
    }

    if (
      !collectionId &&
      !ownerDid &&
      !minterDid &&
      group === NftGroupMode.OwnerDid
    ) {
      return (
        <>
          {ownerDids.map((did, i) => (
            <NftGroupCard
              key={i}
              type='did'
              groupMode={group}
              item={did}
              updateNfts={updateNfts}
              page={page}
              canToggleVisibility={false}
            />
          ))}
        </>
      );
    }

    if (
      !collectionId &&
      !ownerDid &&
      !minterDid &&
      group === NftGroupMode.MinterDid
    ) {
      return (
        <>
          {minterDids.map((did, i) => (
            <NftGroupCard
              key={i}
              type='did'
              groupMode={group}
              item={did}
              updateNfts={updateNfts}
              page={page}
              canToggleVisibility={false}
            />
          ))}
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
                () => handleSelection(nft.launcher_id),
              ]
            : null
        }
      />
    ));
  };

  return (
    <div
      className='grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8 gap-2 md:gap-4 mt-6 mb-2'
      role='grid'
      aria-label={t`NFT Gallery`}
    >
      {renderContent()}
      {children}
    </div>
  );
}
