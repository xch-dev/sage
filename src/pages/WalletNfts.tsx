import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import missing from '@/missing.jpg';
import { ChevronLeftIcon, ChevronRightIcon, Image } from 'lucide-react';
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

  const visibleNfts = showHidden ? nfts : nfts;
  const hasHiddenNfts = false;

  return (
    <>
      <Header title='NFTs'>
        <ReceiveAddress />
      </Header>

      <Container>
        <Button onClick={() => navigate('mint-nft')} className='mb-4'>
          Mint NFT
        </Button>

        {hasHiddenNfts && (
          <div className='inline-flex items-center gap-2 ml-4'>
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

        <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mb-6'>
          {nfts.map((nft, i) => (
            <Nft nft={nft} key={i} />
          ))}
        </div>
      </Container>
    </>
  );
}

function Nft({ nft }: { nft: NftRecord }) {
  let json: any = {};

  if (nft.metadata) {
    try {
      json = JSON.parse(nft.metadata);
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <Link
      to={`/nfts/${nft.launcher_id_hex}`}
      className={`group space-y-3${nft.create_transaction_id !== null ? ' pulsate-opacity' : ''}`}
    >
      <span>
        <div className='overflow-hidden rounded-md'>
          <img
            alt={json.name}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={
              nft.data
                ? `data:${nft.data_mime_type};base64,${nft.data}`
                : missing
            }
          />
        </div>
      </span>
      <div className='space-y-1 text-sm'>
        <span className='font-medium leading-none'>
          {json.name ?? 'Unknown NFT'}
        </span>
        <p className='text-xs text-muted-foreground'>
          {json.collection && json.collection.name}
        </p>
      </div>
    </Link>
  );
}
