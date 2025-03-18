import { NftCard } from '@/components/NftCard';
import { NftGroupCard } from '@/components/NftGroupCard';
import { CardSize, NftGroupMode } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { ReactNode, useCallback } from 'react';
import {
  commands,
  DidRecord,
  Error,
  NftCollectionRecord,
  NftRecord,
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
  cardSize?: CardSize;
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
  cardSize = CardSize.Large,
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
              isPlaceHolder={col.collection_id === 'No collection'}
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
              isPlaceHolder={false}
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
              isPlaceHolder={false}
            />
          ))}
        </>
      );
    }

    return nfts.map((nft) => (
      <NftCard
        nft={nft}
        key={nft.coin_id}
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
      className={`grid gap-2 md:gap-4 mt-6 mb-2 ${
        cardSize === CardSize.Large
          ? 'grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8'
          : 'grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 2xl:grid-cols-12'
      }`}
      role='grid'
      aria-label={t`NFT Gallery`}
    >
      {renderContent()}
      {children}
    </div>
  );
}
