import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { ChevronLeftIcon, ChevronRightIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { commands, NftRecord } from '../bindings';

export function WalletNfts() {
  const [page, setPage] = useState(0);
  const [totalPages, setTotalPages] = useState(1);
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
    const interval = setInterval(() => {
      updateNfts(page);
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [page]);

  return (
    <>
      <Header title='NFTs'>
        {' '}
        <ReceiveAddress />
      </Header>
      <Container>
        <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mb-6'>
          {nfts.map((nft, i) => (
            <Nft nft={nft} key={i} />
          ))}
        </div>
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
    <Link to={`/nfts/${nft.launcher_id_hex}`} className='group space-y-3'>
      <span>
        <div className='overflow-hidden rounded-md'>
          <img
            alt={json.name}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={`data:${nft.data_mime_type};base64,${nft.data}`}
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
