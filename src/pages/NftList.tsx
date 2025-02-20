import Container from '@/components/Container';
import Header from '@/components/Header';
import { MultiSelectActions } from '@/components/MultiSelectActions';
import { NftPageTitle } from '@/components/NftPageTitle';
import { NftCardList } from '@/components/NftCardList';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { useNftParams, NftGroupMode } from '@/hooks/useNftParams';
import { Trans } from '@lingui/react/macro';
import { ImagePlusIcon, EyeIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useNftData } from '@/hooks/useNftData';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';

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
        <Button
          onClick={() => navigate('/nfts/mint')}
          aria-label={t`Create new NFT`}
        >
          <ImagePlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
          <Trans>Mint NFT</Trans>
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
          total={total}
          canLoadMore={canLoadMore()}
          aria-live='polite'
        />

        <main aria-label={t`NFT Collection`} aria-busy={isLoading}>
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
