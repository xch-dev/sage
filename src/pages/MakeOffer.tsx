import { Assets, CatAmount, commands } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useWalletState } from '@/state';
import { HandCoins, Handshake, ImageIcon, TrashIcon } from 'lucide-react';
import { useEffect, useState } from 'react';

export function MakeOffer() {
  const [offerAssets, setOfferAssets] = useState<Assets>({
    xch: '0',
    nfts: [],
    cats: [],
  });

  const [requestAssets, setRequestAssets] = useState<Assets>({
    xch: '0',
    nfts: [],
    cats: [],
  });

  const [fee, setFee] = useState('');

  const make = () => {
    commands
      .makeOffer({
        offered_assets: offerAssets,
        requested_assets: requestAssets,
        fee,
      })
      .then((result) => {
        console.log(result);
      });
  };

  return (
    <>
      <Header title='Make Offer' />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                Offered
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                Add the assets you are offering.
              </div>

              <AssetSelector setAssets={setOfferAssets} />

              <div className='mt-4 flex flex-col space-y-1.5'>
                <Label htmlFor='fee'>Network Fee</Label>
                <Input
                  id='fee'
                  placeholder='Enter fee'
                  value={fee}
                  onChange={(e) => setFee(e.target.value)}
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                Requested
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                Add the assets you are requesting.
              </div>

              <AssetSelector setAssets={setRequestAssets} />
            </CardContent>
          </Card>
        </div>

        <Button className='mt-4' onClick={make}>
          Make Offer
        </Button>
      </Container>
    </>
  );
}

interface AssetSelectorProps {
  setAssets: (value: Assets) => void;
}

function AssetSelector({ setAssets }: AssetSelectorProps) {
  const walletState = useWalletState();

  const [includeAmount, setIncludeAmount] = useState(false);
  const [amount, setAmount] = useState('');
  const [nfts, setNfts] = useState<string[]>([]);
  const [cats, setCats] = useState<CatAmount[]>([]);

  useEffect(() => {
    setAssets({
      xch: includeAmount ? amount : '0',
      nfts,
      cats,
    });
  }, [includeAmount, amount, nfts, cats, setAssets]);

  return (
    <>
      <div className='mt-4 flex gap-2 w-full items-center'>
        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setNfts([...nfts, '']);
          }}
        >
          NFT
        </Button>

        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setCats([...cats, { asset_id: '', amount: '' }]);
          }}
        >
          Token
        </Button>
      </div>

      <div className='mt-4 flex items-center gap-2'>
        <Label htmlFor='include-offer-amount'>
          Include {walletState.sync.unit.ticker}
        </Label>
        <Switch
          id='include-offer-amount'
          checked={includeAmount}
          onCheckedChange={(value) => setIncludeAmount(value)}
        />
      </div>

      {includeAmount && (
        <div className='mt-4 flex flex-col space-y-1.5'>
          <Label htmlFor='offer-amount'>Amount</Label>
          <Input
            id='offer-amount'
            placeholder='Enter amount'
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
          />
        </div>
      )}

      {nfts.length > 0 && (
        <div className='flex flex-col gap-4 mt-4'>
          {nfts.map((nft, i) => (
            <div key={i} className='flex flex-col space-y-1.5'>
              <Label
                htmlFor={`offer-nft-${i}`}
                className='flex items-center gap-1'
              >
                <ImageIcon className='h-4 w-4' />
                <span>NFT {i + 1}</span>
              </Label>
              <div className='flex'>
                <Input
                  id={`offer-nft-${i}`}
                  className='rounded-r-none'
                  placeholder='Enter launcher id'
                  value={nft}
                  onChange={(e) => {
                    nfts[i] = e.target.value;
                    setNfts([...nfts]);
                  }}
                />
                <Button
                  variant='outline'
                  size='icon'
                  className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0'
                  onClick={() => {
                    nfts.splice(i, 1);
                    setNfts([...nfts]);
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {cats.length > 0 && (
        <div className='flex flex-col gap-4 mt-4'>
          {cats.map((cat, i) => (
            <div key={i} className='flex flex-col space-y-1.5'>
              <Label
                htmlFor={`offer-cat-${i}`}
                className='flex items-center gap-1'
              >
                <HandCoins className='h-4 w-4' />
                <span>Token {i + 1}</span>
              </Label>
              <div className='flex'>
                <Input
                  id={`offer-cat-${i}`}
                  className='rounded-r-none'
                  placeholder='Enter asset id'
                  value={cat.asset_id}
                  onChange={(e) => {
                    cats[i].asset_id = e.target.value;
                    setCats([...cats]);
                  }}
                />
                <Input
                  id={`offer-cat-${i}-amount`}
                  className='border-l-0 rounded-l-none rounded-r-none w-[100px]'
                  placeholder='Amount'
                  value={cat.amount}
                  onChange={(e) => {
                    cats[i].amount = e.target.value;
                    setCats([...cats]);
                  }}
                />
                <Button
                  variant='outline'
                  size='icon'
                  className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0'
                  onClick={() => {
                    cats.splice(i, 1);
                    setCats([...cats]);
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
