import Container from '@/components/Container';
import Header from '@/components/Header';
import { MultiSelectActions } from '@/components/MultiSelectActions';
import { NftCardList } from '@/components/NftCardList';
import { NftOptions } from '@/components/NftOptions';
import { NftPageTitle } from '@/components/NftPageTitle';
import { Pagination } from '@/components/Pagination';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { useIntersectionObserver } from '@/hooks/useIntersectionObserver';
import { useNftData } from '@/hooks/useNftData';
import { NftGroupMode, useNftParams } from '@/hooks/useNftParams';
import { exportNfts } from '@/lib/exportNfts';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ImagePlusIcon } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

export function NftList() {
  const navigate = useNavigate();
  const {
    collection_id: collectionId,
    owner_did: ownerDid,
    minter_did: minterDid,
  } = useParams();
  const [params, setParams] = useNftParams();
  const { pageSize, sort, group, showHidden, query } = params;
  const [multiSelect, setMultiSelect] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const { addError } = useErrors();
  const {
    nfts,
    collections,
    ownerDids,
    minterDids,
    owner,
    collection,
    isLoading,
    updateNfts,
    total,
  } = useNftData({
    pageSize,
    sort,
    group,
    showHidden,
    query,
    collectionId,
    ownerDid,
    minterDid,
    page: params.page,
  });

  const optionsRef = useRef<HTMLDivElement>(null);
  const [isOptionsVisible, setIsOptionsVisible] = useState(true);

  useIntersectionObserver(optionsRef, ([entry]) => {
    setIsOptionsVisible(entry.isIntersecting);
  });

  // Reset multi-select when route changes
  useEffect(() => {
    setMultiSelect(false);
    setSelected([]);
  }, [collectionId, ownerDid, minterDid, group]);

  const canLoadMore = useCallback(() => {
    // If we're grouping by collection, or filtering by collection,owner,
    // or minter, use the total number of nfts in the current page
    // otherwise use the appropriate grouping list length
    if (collectionId || ownerDid || minterDid || group === NftGroupMode.None) {
      return nfts.length === pageSize;
    } else if (group === NftGroupMode.Collection) {
      return collections.length === pageSize;
    } else if (group === NftGroupMode.OwnerDid) {
      return ownerDids.length === pageSize;
    } else if (group === NftGroupMode.MinterDid) {
      return minterDids.length === pageSize;
    }
    return false;
  }, [
    collectionId,
    ownerDid,
    minterDid,
    group,
    nfts.length,
    collections.length,
    ownerDids.length,
    minterDids.length,
    pageSize,
  ]);

  const renderPagination = useCallback(
    (compact = false) => (
      <Pagination
        page={params.page}
        total={total}
        pageSize={params.pageSize}
        onPageChange={(newPage) => setParams({ page: newPage })}
        onPageSizeChange={(newSize) =>
          setParams({ pageSize: newSize, page: 1 })
        }
        pageSizeOptions={[24, 48, 72, 96]}
        compact={compact}
        canLoadMore={canLoadMore()}
        isLoading={isLoading}
      />
    ),
    [params.page, params.pageSize, total, setParams, canLoadMore, isLoading],
  );

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
        paginationControls={
          !isOptionsVisible ? renderPagination(true) : undefined
        }
      >
        <ReceiveAddress />
      </Header>

      <Container>
        <Button
          onClick={() => navigate('/nfts/mint')}
          aria-label={t`Create new NFT`}
        >
          <ImagePlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
          <Trans>Mint NFT</Trans>
        </Button>

        <div ref={optionsRef}>
          <NftOptions
            params={params}
            setParams={setParams}
            multiSelect={multiSelect}
            setMultiSelect={(value) => {
              setMultiSelect(value);
              setSelected([]);
            }}
            className='mt-4'
            renderPagination={() => renderPagination(false)}
            aria-live='polite'
            onExport={() =>
              exportNfts({
                sort,
                group,
                showHidden,
                query,
                collectionId,
                ownerDid,
                minterDid,
              })
            }
          />
        </div>

        <main aria-label={t`NFT Collection`}>
          <NftCardList
            collectionId={collectionId}
            ownerDid={ownerDid}
            minterDid={minterDid}
            group={group}
            nfts={nfts}
            collections={collections}
            ownerDids={ownerDids}
            minterDids={minterDids}
            updateNfts={updateNfts}
            page={params.page}
            multiSelect={multiSelect}
            selected={selected}
            setSelected={setSelected}
            addError={addError}
            cardSize={params.cardSize}
          />
        </main>
      </Container>

      {selected.length > 0 && (
        <MultiSelectActions
          selected={selected}
          onConfirm={() => {
            setSelected([]);
            setMultiSelect(false);
          }}
          aria-label={t`Actions for selected NFTs`}
        />
      )}
    </>
  );
}
