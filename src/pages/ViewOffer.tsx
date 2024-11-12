import { Amount, commands, Error, OfferSummary } from '@/bindings';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import ErrorDialog from '@/components/ErrorDialog';
import Header from '@/components/Header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { HandCoins, Handshake } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

interface NftSummary {
  name: string;
}

interface CatSummary {
  name: string;
  ticker: string;
  amount: Amount;
}

export function ViewOffer() {
  const { offer } = useParams();

  const navigate = useNavigate();

  const [summary, setSummary] = useState<OfferSummary | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!offer) return;

    commands.viewOffer(offer).then((result) => {
      if (result.status === 'error') {
        setError(result.error);
      } else {
        setSummary(result.data);
      }
    });
  }, [offer]);

  let offeredAmount = BigNumber(0);
  const offeredCats: Record<string, CatSummary> = {};
  const offeredNfts: Record<string, NftSummary> = {};

  for (const offered of summary?.offered || []) {
    switch (offered.type) {
      case 'xch':
        offeredAmount = offeredAmount.plus(offered.offered_amount);
        break;

      case 'cat':
        if (!(offered.asset_id in offeredCats)) {
          offeredCats[offered.asset_id] = {
            amount: '0',
            name: offered.name || 'Unknown',
            ticker: offered.ticker || 'CAT',
          };
        }

        offeredCats[offered.asset_id].amount = BigNumber(
          offeredCats[offered.asset_id].amount,
        )
          .plus(offered.offered_amount)
          .toString();

        break;

      case 'nft':
        offeredNfts[offered.launcher_id] = {
          name: offered.name || 'Unknown',
        };
        break;
    }
  }

  let requestedAmount = BigNumber(0);
  const requestedCats: Record<string, CatSummary> = {};
  const requestedNfts: Record<string, NftSummary> = {};

  for (const requested of summary?.requested || []) {
    switch (requested.type) {
      case 'xch':
        requestedAmount = requestedAmount.plus(requested.amount);
        break;

      case 'cat':
        if (!(requested.asset_id in requestedCats)) {
          requestedCats[requested.asset_id] = {
            amount: '0',
            name: requested.name || 'Unknown',
            ticker: requested.ticker || 'CAT',
          };
        }

        requestedCats[requested.asset_id].amount = BigNumber(
          requestedCats[requested.asset_id].amount,
        )
          .plus(requested.amount)
          .toString();

        break;

      case 'nft':
        requestedNfts[requested.launcher_id] = {
          name: requested.name || 'Unknown',
        };
        break;
    }
  }

  return (
    <>
      <Header title='View Offer' />

      <Container>
        <CopyBox title='Offer' content={offer ?? ''} />

        <div className='mt-4 grid grid-cols-1 lg:grid-cols-2 gap-4'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                Requested
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                Add the assets you have to pay to fulfill the offer.
              </div>

              <Assets
                amount={offeredAmount}
                cats={offeredCats}
                nfts={offeredNfts}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                Offered
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                The assets being given to you as part of this offer.
              </div>

              <Assets
                amount={requestedAmount}
                cats={requestedCats}
                nfts={requestedNfts}
              />
            </CardContent>
          </Card>
        </div>

        <div className='mt-4 flex gap-2'>
          <Button variant='outline' onClick={() => {}}>
            Save Offer
          </Button>

          <Button onClick={() => {}}>Take Offer</Button>
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
  amount: BigNumber;
  cats: Record<string, CatSummary>;
  nfts: Record<string, NftSummary>;
}

function Assets({ amount, cats, nfts }: AssetsProps) {
  const walletState = useWalletState();

  if (
    amount.isLessThanOrEqualTo(0) &&
    Object.keys(cats).length === 0 &&
    Object.keys(nfts).length === 0
  ) {
    return <></>;
  }

  return (
    <div className='mt-4 flex flex-col gap-2'>
      {amount.isGreaterThan(0) && (
        <div className='mt-2 flex items-center gap-2 p-1.5 rounded-md border bg-neutral-900'>
          <Badge className='max-w-[100px]'>
            <span className='truncate'>Chia</span>
          </Badge>
          <div className='text-sm font-medium'>
            {amount.toString()} {walletState.sync.unit.ticker}
          </div>
        </div>
      )}

      {Object.entries(cats).map(([assetId, cat], i) => (
        <div
          key={i}
          className='mt-2 flex flex-col gap-1.5 p-1.5 rounded-md border bg-neutral-900'
        >
          <div className='flex items-center gap-2'>
            <Badge className='max-w-[100px]'>
              <span className='truncate'>CAT</span>
            </Badge>
            <div className='text-sm font-medium truncate'>
              {cat.amount} {cat.ticker} ({cat.name})
            </div>
          </div>

          <Separator />

          <div className='text-xs truncate'>{assetId}</div>
        </div>
      ))}

      {Object.entries(nfts).map(([launcherId, nft], i) => (
        <div
          key={i}
          className='mt-2 flex flex-col gap-1.5 p-1.5 rounded-md border bg-neutral-900'
        >
          <div className='flex items-center gap-2'>
            <Badge className='max-w-[100px]'>
              <span className='truncate'>NFT</span>
            </Badge>
            <div className='text-sm font-medium truncate'>{nft.name}</div>
          </div>

          <Separator />

          <div className='text-xs truncate'>{launcherId}</div>
        </div>
      ))}
    </div>
  );
}
