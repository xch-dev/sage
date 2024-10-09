import { GridRowSelectionModel } from '@mui/x-data-grid';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { CatRecord, CoinRecord, commands, events } from '../bindings';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { HandHelping, Send } from 'lucide-react';
import Container from '@/components/Container';
import CoinList from '@/components/CoinList';
import { useWalletState } from '@/state';

export default function Token() {
  const navigate = useNavigate();
  const { asset_id: assetId } = useParams();

  const walletState = useWalletState();

  const [asset, setAsset] = useState<CatRecord | null>(null);
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [selectedCoins, setSelectedCoins] = useState<GridRowSelectionModel>([]);

  const updateCoins = () => {
    const getCoins =
      assetId === 'xch' ? commands.getCoins() : commands.getCatCoins(assetId!);
    getCoins.then((res) => {
      if (res.status === 'ok') {
        setCoins(res.data);
      }
    });
  };

  useEffect(() => {
    updateCoins();

    const unlisten = events.syncEvent.listen((event) => {
      if (
        event.payload.type === 'coin_update' ||
        event.payload.type === 'puzzle_update'
      ) {
        updateCoins();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  const updateCat = () => {
    commands.getCat(assetId!).then((res) => {
      if (res.status === 'ok') {
        setAsset(res.data);
      }
    });
  };

  useEffect(() => {
    if (assetId === 'xch') {
      setAsset({
        asset_id: 'xch',
        name: 'Chia',
        description: 'The native token of the Chia blockchain.',
        ticker: walletState.sync.unit.ticker,
        balance: walletState.sync.balance,
        icon_url: 'https://icons.dexie.space/xch.webp',
        visible: true,
      });
    } else {
      updateCat();

      const unlisten = events.syncEvent.listen((event) => {
        if (event.payload.type === 'cat_update') {
          updateCat();
        }
      });

      return () => {
        unlisten.then((u) => u());
      };
    }
  }, [assetId]);

  return (
    <>
      <Header title={asset?.name ?? 'Unknown asset'} />
      <Container>
        <div className='grid lg:grid-cols-2 gap-8'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
              <CardTitle className='text-lg font-medium'>Balance</CardTitle>
              <div>
                <img src={asset?.icon_url ?? ''} className='h-8 w-8' />
              </div>
            </CardHeader>
            <CardContent className='flex flex-col gap-2'>
              <div className='text-4xl font-medium font-mono'>
                {asset?.balance ?? 'Loading'} {asset?.ticker}
              </div>
              <div className='flex gap-2 mt-2'>
                <Button
                  onClick={() =>
                    navigate(
                      assetId === 'xch'
                        ? '/wallet/send'
                        : '/wallet/send-cat/' + assetId,
                    )
                  }
                >
                  <Send className='mr-2 h-4 w-4' /> Send
                </Button>
                <Button
                  variant={'outline'}
                  onClick={() => navigate('/wallet/receive')}
                >
                  <HandHelping className='mr-2 h-4 w-4' /> Receive
                </Button>
              </div>
            </CardContent>
          </Card>
          <Card className='lg:col-span-2 overflow-auto'>
            <CardHeader>
              <CardTitle className='text-lg font-medium'>Coins</CardTitle>
            </CardHeader>
            <CardContent>
              <CoinList
                coins={coins}
                selectedCoins={selectedCoins}
                setSelectedCoins={setSelectedCoins}
              />
            </CardContent>
          </Card>
        </div>
      </Container>
    </>
  );
}
