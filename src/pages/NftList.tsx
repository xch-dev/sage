import Container from '@/components/Container';
import Header from '@/components/Header';
import { NftCard } from '@/components/NftCard';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import collectionImage from '@/images/collection.png';
import { useWalletState } from '@/state';
import {
  ArrowUpAz,
  ChevronLeftIcon,
  ChevronRightIcon,
  Clock2,
  EyeIcon,
  EyeOff,
  Image,
  Images,
  MoreVerticalIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { commands, events, NftCollectionRecord, NftRecord } from '../bindings';

const pageSize = 12;

enum View {
  Name = 'name',
  Recent = 'recent',
  Collection = 'collection',
}

function parseView(view: string): View {
  switch (view) {
    case 'name':
      return View.Name;
    case 'recent':
      return View.Recent;
    case 'collection':
      return View.Collection;
    default:
      return View.Name;
  }
}

export function NftList() {
  const walletState = useWalletState();

  const [params, setParams] = useSearchParams();

  const page = parseInt(params.get('page') ?? '0');
  const view = parseView(params.get('view') ?? 'name');

  const updateParams = ({ page, view }: { page?: number; view?: View }) => {
    setParams((prev) => {
      const next = new URLSearchParams(prev);
      if (page !== undefined) {
        next.set('page', page.toString());
      }
      if (view !== undefined) {
        next.set('view', view);
      }
      return next;
    });
  };

  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const [showHidden, setShowHidden] = useState(false);

  const updateNfts = async (page: number) => {
    if (view === 'name' || view === 'recent') {
      return await commands
        .getNfts({
          offset: page * pageSize,
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
          offset: page * pageSize,
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
  };

  const nextPage = () => {
    if (loading) return;
    setLoading(true);

    updateNfts(page + 1)
      .then(() => updateParams({ page: page + 1 }))
      .finally(() => {
        setLoading(false);
      });
  };

  const previousPage = () => {
    if (loading) return;
    setLoading(true);

    updateNfts(page - 1)
      .then(() => updateParams({ page: page - 1 }))
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    updateNfts(0);
  }, [view]);

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
  }, [page]);

  useEffect(() => {
    updateNfts(page);
  }, [showHidden]);

  const totalPages = Math.max(
    1,
    Math.ceil(
      (view === 'collection'
        ? walletState.nfts.visible_collections
        : walletState.nfts.visible_nfts) / pageSize,
    ),
  );

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
          <div className='relative flex justify-center items-center gap-2'>
            <Button
              variant='outline'
              size='icon'
              onClick={() => previousPage()}
              disabled={page === 0}
            >
              <ChevronLeftIcon className='h-4 w-4' />
            </Button>
            <p className='text-sm text-muted-foreground font-medium'>
              Page {page + 1} of {totalPages}
            </p>
            <Button
              variant='outline'
              size='icon'
              onClick={() => nextPage()}
              disabled={page >= totalPages - 1}
            >
              <ChevronRightIcon className='h-4 w-4' />
            </Button>

            <div className='absolute right-0 flex gap-2 items-center'>
              <Button
                variant='outline'
                size='icon'
                onClick={() => setShowHidden(!showHidden)}
              >
                {showHidden ? (
                  <EyeIcon className='h-4 w-4' />
                ) : (
                  <EyeOff className='h-4 w-4' />
                )}
              </Button>

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant='outline' size='icon'>
                    {view === 'name' ? (
                      <ArrowUpAz className='h-4 w-4' />
                    ) : view === 'recent' ? (
                      <Clock2 className='h-4 w-4' />
                    ) : (
                      <Images className='h-4 w-4' />
                    )}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align='end'>
                  <DropdownMenuGroup>
                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        updateParams({
                          page: 0,
                          view: View.Name,
                        });
                      }}
                    >
                      <ArrowUpAz className='mr-2 h-4 w-4' />
                      <span>Sort Alphabetically</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        updateParams({
                          page: 0,
                          view: View.Recent,
                        });
                      }}
                    >
                      <Clock2 className='mr-2 h-4 w-4' />
                      <span>Sort Recent</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        updateParams({
                          page: 0,
                          view: View.Collection,
                        });
                      }}
                    >
                      <Images className='mr-2 h-4 w-4' />
                      <span>Group Collections</span>
                    </DropdownMenuItem>
                  </DropdownMenuGroup>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        )}

        <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mt-6 mb-2'>
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
                  collection_id: null,
                  visible: true,
                }}
                updateNfts={() => updateNfts(page)}
              />
            </>
          ) : (
            nfts.map((nft, i) => (
              <NftCard nft={nft} key={i} updateNfts={() => updateNfts(page)} />
            ))
          )}
        </div>
      </Container>
    </>
  );
}

interface CollectionProps {
  col: Omit<NftCollectionRecord, 'collection_id'> & {
    collection_id: string | null;
  };
  updateNfts: () => void;
}

function Collection({ col, updateNfts }: CollectionProps) {
  const toggleVisibility = () => {
    // commands.updateNft(nft.launcher_id, !nft.visible).then((result) => {
    //   if (result.status === 'ok') {
    //     updateNfts();
    //   } else {
    //     throw new Error('Failed to toggle visibility for NFT');
    //   }
    // });
  };

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
                    toggleVisibility();
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
