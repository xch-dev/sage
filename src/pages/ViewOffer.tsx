import { commands, Error, OfferAssets, OfferSummary } from '@/bindings';
import Container from '@/components/Container';
import ErrorDialog from '@/components/ErrorDialog';
import Header from '@/components/Header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { ArrowDownIcon, ArrowUpIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

export function ViewOffer() {
  const { offer } = useParams();

  const navigate = useNavigate();

  const [summary, setSummary] = useState<OfferSummary | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!offer) return;

    commands.viewOffer({ offer }).then((result) => {
      if (result.status === 'error') {
        setError(result.error);
      } else {
        setSummary(result.data.offer);
      }
    });
  }, [offer]);

  return (
    <>
      <Header title='View Offer' />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <ArrowUpIcon className='mr-2 h-4 w-4' />
                Offering
              </CardTitle>
            </CardHeader>
            <CardContent className='flex flex-col divide-y'>
              <div className='text-sm text-muted-foreground'>
                The assets you have to pay to fulfill the offer.
              </div>

              <div className='mt-4'>
                <Assets
                  assets={
                    summary?.taker ?? {
                      xch: { amount: '0', royalty: '0' },
                      cats: {},
                      nfts: {},
                    }
                  }
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <ArrowDownIcon className='mr-2 h-4 w-4' />
                Receiving
              </CardTitle>
            </CardHeader>
            <CardContent className='flex flex-col divide-y'>
              <div className='text-sm text-muted-foreground'>
                The assets being given to you as part of this offer.
              </div>

              <div className='mt-4'>
                <Assets
                  assets={
                    summary?.maker ?? {
                      xch: { amount: '0', royalty: '0' },
                      cats: {},
                      nfts: {},
                    }
                  }
                />
              </div>
            </CardContent>
          </Card>
        </div>

        <div className='mt-4 flex gap-2'>
          <Button variant='outline' onClick={() => {}}>
            Save Offer
          </Button>

          <Button
            onClick={() => {
              commands
                .takeOffer({ offer: offer!, fee: '0.00005' })
                .then((result) => {
                  if (result.status === 'error') {
                    setError(result.error);
                  } else {
                    navigate('/offers');
                  }
                });
            }}
          >
            Take Offer
          </Button>
        </div>

        <ErrorDialog
          error={error}
          setError={(error) => {
            setError(error);
            if (error === null) navigate('/offers');
          }}
        />
      </Container>
    </>
  );
}

interface AssetsProps {
  assets: OfferAssets;
}

function Assets({ assets }: AssetsProps) {
  const walletState = useWalletState();

  const amount = BigNumber(assets.xch.amount);

  if (
    amount.isLessThanOrEqualTo(0) &&
    Object.keys(assets.cats).length === 0 &&
    Object.keys(assets.nfts).length === 0
  ) {
    return <></>;
  }

  return (
    <div className='flex flex-col space-y-4 divide-y [&>*]:pt-4 divide-neutral-200 dark:divide-neutral-800'>
      {amount.isGreaterThan(0) && (
        <div className='flex justify-between items-center gap-2 rounded-md'>
          <Badge>
            <span className='truncate'>XCH</span>
          </Badge>
          <div className='text-sm font-medium'>
            {amount.toString()} {walletState.sync.unit.ticker}
          </div>
        </div>
      )}

      {Object.entries(assets.cats).map(([assetId, cat], i) => (
        <div key={i} className='flex flex-col gap-1.5 rounded-md'>
          <div className='overflow-hidden flex justify-between items-center gap-2'>
            <div className='truncate flex items-center gap-2'>
              <Badge className='max-w-[100px] bg-blue-600 text-white dark:bg-blue-600 dark:text-white'>
                <span className='truncate'>CAT</span>
              </Badge>
              <div className='text-xs text-muted-foreground truncate'>
                {cat.name
                  ? `${cat.name} (${assetId.substring(0, 6)}...${assetId.substring(
                      assetId.length - 6,
                    )})`
                  : assetId.substring(0, 6) +
                    '...' +
                    assetId.substring(assetId.length - 6)}
              </div>
            </div>
            <div className='text-sm font-medium whitespace-nowrap'>
              {cat.amount} {cat.ticker}
            </div>
          </div>
        </div>
      ))}

      {Object.entries(assets.nfts).map(([launcherId, nft], i) => (
        <div key={i} className='flex flex-col gap-1.5 rounded-md'>
          <div className='overflow-hidden flex justify-between items-center gap-2'>
            <div className='truncate flex items-center gap-2'>
              <Badge className='max-w-[100px] bg-green-600 text-white dark:bg-green-600 dark:text-white'>
                <span className='truncate'>NFT</span>
              </Badge>
              <div className='max-w-[10rem] text-xs truncate text-muted-foreground'>
                {launcherId}
              </div>
            </div>

            <div className='text-sm font-medium'>{nft.name}</div>
          </div>
        </div>
      ))}
    </div>
  );
}
