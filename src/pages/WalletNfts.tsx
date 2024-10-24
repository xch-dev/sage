import Container from '@/components/Container';
import Header from '@/components/Header';
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
import { Switch } from '@/components/ui/switch';
import { nftUri } from '@/lib/nftUri';
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  EyeIcon,
  EyeOff,
  Image,
  MoreVerticalIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { commands, events, NftRecord } from '../bindings';

export function WalletNfts() {
  const navigate = useNavigate();

  const [page, setPage] = useState(0);
  const [totalPages, setTotalPages] = useState(1);
  const [showHidden, setShowHidden] = useState(false);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const updateNfts = async (page: number) => {
    return await commands
      .getNfts({ offset: page * 12, limit: 12 })
      .then((result) => {
        if (result.status === 'ok') {
          setNfts(result.data.items);
          setTotalPages(Math.max(1, Math.ceil(result.data.total / 12)));
        } else {
          throw new Error('Failed to get NFTs');
        }
      });
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
  }, []);

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

  return (
    <>
      <Header title='NFTs'>
        <ReceiveAddress />
      </Header>

      <Container>
        {hasHiddenNfts && (
          <div className='inline-flex items-center gap-2 mb-2'>
            <label htmlFor='viewHidden'>View hidden</label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

        {visibleNfts.length === 0 ? (
          <Alert className='mt-2'>
            <Image className='h-4 w-4' />
            <AlertTitle>Mint an NFT?</AlertTitle>
            <AlertDescription>
              You do not currently have any {nfts.length > 0 ? 'visible ' : ''}
              NFTs. Would you like to mint one?
            </AlertDescription>
          </Alert>
        ) : (
          <div className='flex justify-center items-center gap-2'>
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
          </div>
        )}

        <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mt-6 mb-2'>
          {visibleNfts.map((nft, i) => (
            <Nft nft={nft} key={i} updateNfts={() => updateNfts(page)} />
          ))}
        </div>
      </Container>
    </>
  );
}

interface NftProps {
  nft: NftRecord;
  updateNfts: () => void;
}

function Nft({ nft, updateNfts }: NftProps) {
  let json: any = {};

  if (nft.metadata) {
    try {
      json = JSON.parse(nft.metadata);
    } catch (error) {
      console.error(error);
    }
  }

  const toggleVisibility = () => {
    commands.updateNft(nft.launcher_id, !nft.visible).then((result) => {
      if (result.status === 'ok') {
        updateNfts();
      } else {
        throw new Error('Failed to toggle visibility for NFT');
      }
    });
  };

  return (
    <Link
      to={`/nfts/${nft.launcher_id_hex}`}
      className={`group${`${!nft.visible ? ' opacity-50 grayscale' : nft.create_transaction_id !== null ? ' pulsate-opacity' : ''}`}`}
    >
      <div className='overflow-hidden rounded-t-md relative'>
        <img
          alt={json.name}
          loading='lazy'
          width='150'
          height='150'
          className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
          src={nftUri(nft.data_mime_type, nft.data)}
        />
      </div>
      <div className='text-md flex items-center justify-between border rounded-b p-1 pl-2'>
        <span className='truncate'>
          <span className='font-medium leading-none'>
            {json.name ?? 'Unknown NFT'}
          </span>
          <p className='text-xs text-muted-foreground'>
            {json.collection?.name ?? 'No collection'}
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
                className='cursor-pointer text-red-600 focus:text-red-500'
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
  );
}
