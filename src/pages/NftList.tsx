import Container from '@/components/Container';
import Header from '@/components/Header';
import { MultiSelectActions } from '@/components/MultiSelectActions';
import { NftCard, NftCardList } from '@/components/NftCard';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useErrors } from '@/hooks/useErrors';
import { useNftParams, NftGroupMode } from '@/hooks/useNftParams';
import collectionImage from '@/images/collection.png';
import profileImage from '@/images/profile.png';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { EyeIcon, EyeOff, ImagePlusIcon, MoreVerticalIcon, UserIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  events,
  NftCollectionRecord,
  NftRecord,
  DidRecord,
} from '../bindings';

export function NftList() {
  const navigate = useNavigate();
  const { collection_id: collectionId, owner_did: ownerDid } = useParams();
  const { addError } = useErrors();

  const [params, setParams] = useNftParams();
  const { pageSize, page, sort, group, showHidden, query } = params;

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
        if (collectionId || ownerDid || group === NftGroupMode.None) {
          const params = {
            name: query || null,
            collection_id: collectionId ?? null,
            owner_did_id: ownerDid === 'No did' ? null : (ownerDid ?? null),
            offset: (page - 1) * pageSize,
            limit: pageSize,
            sort_mode: sort,
            include_hidden: showHidden,
          };

          console.log('Fetching NFTs with params:', params);
          const response = await commands.getNfts(params);
          console.log('NFTs response:', response);
          console.log('NFTs owner_dids:', response.nfts.map(nft => nft.owner_did));
          
          setNfts(response.nfts);

          if (collectionId) {
            const collectionResponse = await commands.getNftCollection({
              collection_id:
                collectionId === 'No collection' ? null : collectionId,
            });
            setCollection(collectionResponse.collection);
          }

          if (ownerDid) {
            const didResponse = await commands.getDids({});
            const foundDid = didResponse.dids.find(
              (did) => did.launcher_id === ownerDid,
            );
            setOwner(
              foundDid || {
                name: 'Unassigned NFTs',
                launcher_id: 'No did',
                visible: true,
                coin_id: 'No coin',
                address: 'no address',
                amount: 0,
                created_height: 0,
                create_transaction_id: 'No transaction',
                recovery_hash: '',
              },
            );
          }
        } else if (group === NftGroupMode.Collection) {
          await commands
            .getNftCollections({
              offset: (page - 1) * pageSize,
              limit: pageSize,
              include_hidden: showHidden,
            })
            .then((data) => setCollections(data.collections))
            .catch(addError);
        } else if (group === NftGroupMode.OwnerDid) {
          await commands
            .getDids({})
            .then((data) => setDids(data.dids))
            .catch(addError);
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
      addError,
    ],
  );

  useEffect(() => {
    // Clear NFTs when view parameters change
    setNfts([]);
    setCollection(null);
    setOwner(null);
    updateNfts(1);
  }, [updateNfts, collectionId, ownerDid]);

  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'nft_data'
      ) {
        updateNfts(page);
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateNfts, page]);

  // Add this effect to reset multi-select when route changes
  useEffect(() => {
    setMultiSelect(false);
    setSelected([]);
  }, [collectionId, ownerDid, group]);

  return (
    <>
      <Header
        title={
          collectionId ? (
            (collection?.name ?? t`Unknown Collection`)
          ) : ownerDid ? (
            (owner?.name ?? t`Unknown Profile`)
          ) : (
            <Trans>NFTs</Trans>
          )
        }
      >
        <ReceiveAddress />
      </Header>

      <Container>
        <Button onClick={() => navigate('/nfts/mint')}>
          <ImagePlusIcon className='h-4 w-4 mr-2' /> <Trans>Mint NFT</Trans>
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
          canLoadMore={nfts.length === pageSize}
        />

        <NftCardList>
          {!collectionId && !ownerDid && group === NftGroupMode.Collection ? (
            <>
              {collections.map((col, i) => (
                <Collection
                  col={col}
                  key={i}
                  updateNfts={() => updateNfts(page)}
                />
              ))}
              {nfts.length < pageSize && (
                <Collection
                  col={{
                    name: 'Uncategorized NFTs',
                    icon: '',
                    did_id: 'Miscellaneous',
                    metadata_collection_id: 'Uncategorized',
                    collection_id: 'No collection',
                    visible: true,
                  }}
                  updateNfts={() => updateNfts(page)}
                />
              )}
            </>
          ) : !collectionId && !ownerDid && group === NftGroupMode.OwnerDid ? (
            <>
              {dids.map((did, i) => (
                <DidGroup
                  did={did}
                  key={i}
                  updateNfts={() => updateNfts(page)}
                />
              ))}
              <DidGroup
                did={{
                  name: 'Unassigned NFTs',
                  launcher_id: 'No did',
                  visible: true,
                  coin_id: 'No coin',
                  address: 'no address',
                  amount: 0,
                  created_height: 0,
                  create_transaction_id: 'No transaction',
                  recovery_hash: '',
                }}
                updateNfts={() => updateNfts(page)}
              />
            </>
          ) : (
            nfts.map((nft, i) => (
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
                            setSelected([...selected, nft.launcher_id]);
                          } else if (
                            !value &&
                            selected.includes(nft.launcher_id)
                          ) {
                            setSelected(
                              selected.filter((id) => id !== nft.launcher_id),
                            );
                          }
                        },
                      ]
                    : null
                }
              />
            ))
          )}
        </NftCardList>
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

interface CollectionProps {
  col: NftCollectionRecord;
  updateNfts: () => void;
}

function Collection({ col }: CollectionProps) {
  return (
    <>
      <Link
        to={`/nfts/collections/${col.collection_id}`}
        className={`group${`${!col.visible ? ' opacity-50 grayscale' : ''}`} border border-neutral-200 rounded-lg dark:border-neutral-800`}
      >
        <div className='overflow-hidden rounded-t-lg relative'>
          <img
            alt={col.name ?? t`Unnamed`}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={collectionImage}
          />
        </div>
        <div className='border-t bg-white text-neutral-950 shadow  dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {col.name ?? t`Unnamed`}
            </span>
            {col.collection_id && (
              <p className='text-xs text-muted-foreground truncate'>
                {col.collection_id}
              </p>
            )}
          </span>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    // toggleVisibility();
                  }}
                >
                  {col.visible ? (
                    <EyeOff className='mr-2 h-4 w-4' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' />
                  )}
                  <span>{col.visible ? t`Hide` : t`Show`}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </Link>
    </>
  );
}

interface DidGroupProps {
  did: DidRecord;
  updateNfts: () => void;
}

function DidGroup({ did }: DidGroupProps) {
  return (
    <Link
      to={`/nfts/owners/${did.launcher_id}`}
      className={`group${!did.visible ? ' opacity-50 grayscale' : ''} border border-neutral-200 rounded-lg dark:border-neutral-800`}
    >
      <div className='overflow-hidden rounded-t-lg relative bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'>
        <UserIcon className='h-12 w-12 text-neutral-400 dark:text-neutral-600' />
      </div>
      <div className='border-t bg-white text-neutral-950 shadow dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
        <span className='truncate'>
          <span className='font-medium leading-none truncate'>
            {did.name ?? <Trans>Unnamed Profile</Trans>}
          </span>
          <p className='text-xs text-muted-foreground truncate'>
            {did.launcher_id}
          </p>
        </span>
      </div>
    </Link>
  );
}
