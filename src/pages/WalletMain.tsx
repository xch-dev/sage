import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { CatRecord, commands, events } from '../bindings';
import { useWalletState } from '../state';

export function MainWallet() {
  const walletState = useWalletState();

  const [cats, setCats] = useState<CatRecord[]>([]);
  const [showHidden, setShowHidden] = useState(false);

  const visibleCats = cats.filter((cat) => showHidden || cat.visible);
  const hasHiddenAssets = !!cats.find((cat) => !cat.visible);

  const updateCats = () => {
    commands.getCats().then(async (result) => {
      if (result.status === 'ok') {
        setCats(result.data);
      }
    });
  };

  useEffect(() => {
    updateCats();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'cat_info'
      ) {
        updateCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  return (
    <>
      <Header title='Assets'>
        <ReceiveAddress />
      </Header>
      <Container>
        <div className='grid gap-2 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          <Link to={`/wallet/token/xch`}>
            <Card className='transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900'>
              <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
                <CardTitle className='text-md font-medium'>Chia</CardTitle>

                <img
                  alt={`XCH logo`}
                  className='h-6 w-6'
                  src='https://icons.dexie.space/xch.webp'
                />
              </CardHeader>
              <CardContent>
                <div className='text-2xl font-medium truncate'>
                  {walletState.sync.balance}
                </div>
              </CardContent>
            </Card>
          </Link>
          {visibleCats.map((cat) => (
            <Link key={cat.asset_id} to={`/wallet/token/${cat.asset_id}`}>
              <Card
                className={`transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900 ${!cat.visible ? 'opacity-50 grayscale' : ''}`}
              >
                <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
                  <CardTitle className='text-md font-medium truncate'>
                    {cat.name || 'Unknown CAT'}
                  </CardTitle>

                  {cat.icon_url && (
                    <img
                      alt={`${cat.asset_id} logo`}
                      className='h-6 w-6'
                      src={cat.icon_url}
                    />
                  )}
                </CardHeader>
                <CardContent>
                  <div className='text-2xl font-medium truncate'>
                    {cat.balance} {cat.ticker ?? ''}
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
        {hasHiddenAssets && (
          <div className='mt-4'>
            <Button
              variant='link'
              className='text-muted-foreground text-xs'
              onClick={() => setShowHidden(!showHidden)}
            >
              {showHidden ? 'Hide' : 'Show'} hidden assets
            </Button>
          </div>
        )}
      </Container>
    </>
  );
}
