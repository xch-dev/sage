import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import collectionImage from '@/images/collection.png';
import { amount } from '@/lib/formTypes';
import { nftUri } from '@/lib/nftUri';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import BigNumber from 'bignumber.js';
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
  SendIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link } from 'react-router-dom';
import { z } from 'zod';
import {
  commands,
  events,
  NftCollectionRecord,
  NftRecord,
  TransactionSummary,
} from '../bindings';

const pageSize = 12;

export function NftList() {
  const walletState = useWalletState();

  const [page, setPage] = useState(0);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const [showHidden, setShowHidden] = useState(false);
  const [view, setView] = useState<'name' | 'recent' | 'collection'>('name');

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
      .then(() => setPage(page + 1))
      .finally(() => {
        setLoading(false);
      });
  };

  const previousPage = () => {
    if (loading) return;
    setLoading(true);

    updateNfts(page - 1)
      .then(() => setPage(page - 1))
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

  const visibleNfts = showHidden ? nfts : nfts.filter((nft) => nft.visible);
  const hasHiddenNfts = nfts.findIndex((nft) => !nft.visible) > -1;

  const visibleCollections = showHidden
    ? collections
    : collections.filter((collection) => collection.visible);
  const hasHiddenCollections =
    collections.findIndex((collection) => !collection.visible) > -1;

  const hasHidden =
    (view === 'collection' && hasHiddenCollections) || hasHiddenNfts;

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
        {hasHidden && (
          <div className='inline-flex items-center gap-2 mb-2'>
            <label htmlFor='viewHidden'>View hidden</label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

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

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <div className='absolute right-0'>
                  <Button variant='outline' size='icon' onClick={() => {}}>
                    {view === 'name' ? (
                      <ArrowUpAz className='h-4 w-4' />
                    ) : view === 'recent' ? (
                      <Clock2 className='h-4 w-4' />
                    ) : (
                      <Images className='h-4 w-4' />
                    )}
                  </Button>
                </div>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='end'>
                <DropdownMenuGroup>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setView('name');
                      setPage(0);
                    }}
                  >
                    <ArrowUpAz className='mr-2 h-4 w-4' />
                    <span>Sort Alphabetically</span>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setView('recent');
                      setPage(0);
                    }}
                  >
                    <Clock2 className='mr-2 h-4 w-4' />
                    <span>Sort Recent</span>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setView('collection');
                      setPage(0);
                    }}
                  >
                    <Images className='mr-2 h-4 w-4' />
                    <span>Group Collections</span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}

        <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mt-6 mb-2'>
          {view === 'collection' ? (
            <>
              {visibleCollections.map((col, i) => (
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
            visibleNfts.map((nft, i) => (
              <Nft nft={nft} key={i} updateNfts={() => updateNfts(page)} />
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

interface NftProps {
  nft: NftRecord;
  updateNfts: () => void;
}

function Nft({ nft, updateNfts }: NftProps) {
  const walletState = useWalletState();

  const [isTransferOpen, setTransferOpen] = useState(false);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);

  const toggleVisibility = () => {
    commands.updateNft(nft.launcher_id, !nft.visible).then((result) => {
      if (result.status === 'ok') {
        updateNfts();
      } else {
        throw new Error('Failed to toggle visibility for NFT');
      }
    });
  };

  const transferFormSchema = z.object({
    address: z.string().min(1, 'Address is required'),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const transferForm = useForm<z.infer<typeof transferFormSchema>>({
    resolver: zodResolver(transferFormSchema),
    defaultValues: {
      address: '',
      fee: '0',
    },
  });

  const onTransferSubmit = (values: z.infer<typeof transferFormSchema>) => {
    commands
      .transferNft(nft.launcher_id, values.address, values.fee)
      .then((result) => {
        setTransferOpen(false);
        if (result.status === 'error') {
          console.error('Failed to transfer NFT', result.error);
        } else {
          setSummary(result.data);
        }
      });
  };

  return (
    <>
      <Link
        to={`/nfts/${nft.launcher_id}`}
        className={`group${`${!nft.visible ? ' opacity-50 grayscale' : !nft.created_height ? ' pulsate-opacity' : ''}`}`}
      >
        <div className='overflow-hidden rounded-t-md relative'>
          <img
            alt={nft.name ?? 'Unnamed'}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={nftUri(nft.data_mime_type, nft.data)}
          />
        </div>
        <div className='text-md flex items-center justify-between rounded-b p-1 pl-2 bg-neutral-200 dark:bg-neutral-800'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {nft.name ?? 'Unnamed'}
            </span>
            <p className='text-xs text-muted-foreground truncate'>
              {nft.collection_name ?? 'No collection'}
            </p>
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
                    transferForm.reset();
                    setTransferOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>Transfer</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleVisibility();
                  }}
                >
                  {nft.visible ? (
                    <EyeOff className='mr-2 h-4 w-4' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' />
                  )}
                  <span>{nft.visible ? 'Hide' : 'Show'}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </Link>

      <Dialog open={isTransferOpen} onOpenChange={setTransferOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Transfer NFT</DialogTitle>
            <DialogDescription>
              This will send the NFT to the provided address.
            </DialogDescription>
          </DialogHeader>
          <Form {...transferForm}>
            <form
              onSubmit={transferForm.handleSubmit(onTransferSubmit)}
              className='space-y-4'
            >
              <FormField
                control={transferForm.control}
                name='address'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Address</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={transferForm.control}
                name='fee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Network Fee</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setTransferOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Transfer</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <ConfirmationDialog
        summary={summary}
        close={() => setSummary(null)}
        onConfirm={() => updateNfts()}
      />
    </>
  );
}
