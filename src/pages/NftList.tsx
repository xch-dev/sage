import { BurnDialog } from '@/components/BurnDialog';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { NftCard, NftCardList } from '@/components/NftCard';
import { NftOptions } from '@/components/NftOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TransferDialog } from '@/components/TransferDialog';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useNftParams } from '@/hooks/useNftParams';
import collectionImage from '@/images/collection.png';
import { useWalletState } from '@/state';
import {
  ChevronDown,
  EyeIcon,
  EyeOff,
  Flame,
  Image,
  MoreVerticalIcon,
  SendIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  commands,
  events,
  NftCollectionRecord,
  NftRecord,
  TransactionSummary,
} from '../bindings';

export function NftList() {
  const walletState = useWalletState();

  const [params, setParams] = useNftParams();
  const { pageSize, page, view, showHidden } = params;

  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [multiSelect, setMultiSelect] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);

  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);

  const updateNfts = useCallback(
    async (page: number) => {
      if (view === 'name' || view === 'recent') {
        return await commands
          .getNfts({
            offset: (page - 1) * pageSize,
            limit: pageSize,
            sort_mode: view,
            include_hidden: showHidden,
          })
          .then((result) => {
            if (result.status === 'ok') {
              setNfts(result.data);
            } else {
              throw new Error('Failed to get NFTs');
            }
          });
      } else if (view === 'collection') {
        await commands
          .getNftCollections({
            offset: (page - 1) * pageSize,
            limit: pageSize,
            include_hidden: showHidden,
          })
          .then((result) => {
            if (result.status === 'ok') {
              setCollections(result.data);
            } else {
              throw new Error('Failed to get NFT collections');
            }
          });
      }
    },
    [pageSize, showHidden, view],
  );

  useEffect(() => {
    updateNfts(0);
  }, [updateNfts]);

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

  useEffect(() => {
    updateNfts(page);
  }, [updateNfts, page]);

  const totalPages = Math.max(
    1,
    Math.ceil(
      (view === 'collection'
        ? walletState.nfts.visible_collections
        : showHidden
          ? walletState.nfts.nfts
          : walletState.nfts.visible_nfts) / pageSize,
    ),
  );

  const onTransferSubmit = (address: string, fee: string) => {
    commands.transferNfts(selected, address, fee).then((result) => {
      setTransferOpen(false);
      if (result.status === 'error') {
        console.error('Failed to transfer NFTs', result.error);
      } else {
        setSummary(result.data);
      }
    });
  };

  const onBurnSubmit = (fee: string) => {
    commands
      .transferNfts(selected, walletState.sync.burn_address, fee)
      .then((result) => {
        setBurnOpen(false);
        if (result.status === 'error') {
          console.error('Failed to burn NFTs', result.error);
        } else {
          setSummary(result.data);
        }
      });
  };

  return (
    <>
      <Header title='NFTs'>
        <ReceiveAddress />
      </Header>

      <Container>
        {walletState.nfts.nfts === 0 ? (
          <Alert className='mt-2'>
            <Image className='h-4 w-4' />
            <AlertTitle>Mint an NFT?</AlertTitle>
            <AlertDescription>
              You do not currently have any NFTs. Would you like to mint one?
            </AlertDescription>
          </Alert>
        ) : (
          <NftOptions
            totalPages={totalPages}
            allowCollections
            params={params}
            setParams={setParams}
            multiSelect={multiSelect}
            setMultiSelect={(value) => {
              setMultiSelect(value);
              setSelected([]);
            }}
          />
        )}

        <NftCardList>
          {view === 'collection' ? (
            <>
              {collections.map((col, i) => (
                <Collection
                  col={col}
                  key={i}
                  updateNfts={() => updateNfts(page)}
                />
              ))}
              <Collection
                col={{
                  name: 'Uncategorized',
                  icon: '',
                  did_id: 'Miscellaneous',
                  metadata_collection_id: 'Uncategorized',
                  collection_id: 'none',
                  visible: true,
                  // TODO: Fix
                  nfts: 0,
                  visible_nfts: 0,
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
        <div className='absolute flex justify-between items-center gap-3 bottom-6 w-60 px-5 p-3 rounded-lg shadow-md shadow-black left-1/2 -translate-x-1/2 bg-neutral-800'>
          <span className='flex-shrink-0'>{selected.length} selected</span>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button className='flex items-center gap-1'>
                Actions
                <ChevronDown className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='center'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setTransferOpen(true);
                  }}
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>Transfer</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setBurnOpen(true);
                  }}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <span>Burn</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      )}

      <TransferDialog
        title='Bulk Transfer NFTs'
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        This will bulk transfer {selected.length} NFTs to another wallet. Are
        you sure you want to proceed?
      </TransferDialog>

      <BurnDialog
        title='Bulk Burn NFTs'
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        This will bulk burn {selected.length} NFTs. This cannot be undone. Are
        you sure you want to proceed?
      </BurnDialog>

      <ConfirmationDialog
        summary={summary}
        close={() => {
          setSummary(null);
          setSelected([]);
          setMultiSelect(false);
        }}
      />
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
        to={`/collections/${col.collection_id}`}
        className={`group${`${!col.visible ? ' opacity-50 grayscale' : ''}`}`}
      >
        <div className='overflow-hidden rounded-t-md relative'>
          <img
            alt={col.name ?? 'Unnamed'}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={collectionImage}
          />
        </div>
        <div className='text-md flex items-center justify-between rounded-b p-1 pl-2 bg-neutral-200 dark:bg-neutral-800'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {col.name ?? 'Unnamed'}
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
                  <span>{col.visible ? 'Hide' : 'Show'}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </Link>
    </>
  );
}
