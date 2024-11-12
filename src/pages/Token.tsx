import CoinList from '@/components/CoinList';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
import { Label } from '@/components/ui/label';
import { amount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import {
  HandHelping,
  MergeIcon,
  MoreHorizontalIcon,
  Send,
  SplitIcon,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import {
  CatRecord,
  CoinRecord,
  commands,
  events,
  TransactionSummary,
} from '../bindings';
import { CopyButton } from '@/components/CopyButton';

export default function Token() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const { asset_id: assetId } = useParams();

  const [asset, setAsset] = useState<CatRecord | null>(null);
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);
  const [selectedCoins, setSelectedCoins] = useState<RowSelectionState>({});

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
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
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
        const type = event.payload.type;

        if (
          type === 'coin_state' ||
          type === 'puzzle_batch_synced' ||
          type === 'cat_info'
        ) {
          updateCat();
        }
      });

      return () => {
        unlisten.then((u) => u());
      };
    }
  }, [assetId, walletState.sync.balance]);

  const redownload = () => {
    if (!assetId || assetId === 'xch') return;

    commands.removeCatInfo(assetId).then((res) => {
      if (res.status === 'ok') {
        updateCat();
      }
    });
  };

  const setVisibility = (visible: boolean) => {
    if (!asset || assetId === 'xch') return;
    asset.visible = visible;

    commands.updateCatInfo(asset).then((res) => {
      if (res.status === 'ok') {
        navigate('/wallet');
      }
    });
  };

  const [isEditOpen, setEditOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newTicker, setNewTicker] = useState('');

  const edit = () => {
    if (!newName || !newTicker || !asset) return;

    asset.name = newName;
    asset.ticker = newTicker;

    commands.updateCatInfo(asset).then((res) => {
      if (res.status === 'ok') {
        updateCat();
      }
    });

    setEditOpen(false);
  };

  return (
    <>
      <Header
        title={
          <span>
            {asset ? (asset.name ?? 'Unknown asset') : ''}{' '}
            {asset?.asset_id !== 'xch' && (
              <CopyButton value={asset?.asset_id ?? ''} />
            )}
          </span>
        }
      />
      <Container>
        <div className='flex flex-col gap-8 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row justify-between items-center space-y-0 space-x-2 pb-2'>
              <div className='flex text-xl sm:text-4xl font-medium font-mono truncate'>
                <span className='truncate'>{asset?.balance ?? ' '}&nbsp;</span>
                {asset?.ticker}
              </div>
              <div className='flex-shrink-0'>
                <img
                  alt='asset icon'
                  src={asset?.icon_url ?? ''}
                  className='h-8 w-8'
                />
              </div>
            </CardHeader>
            <CardContent className='flex flex-col gap-2'>
              <ReceiveAddress className='mt-2' />

              <div className='flex gap-2 mt-2 flex-wrap'>
                <Link to={`/wallet/send/${assetId}`}>
                  <Button>
                    <Send className='mr-2 h-4 w-4' /> Send
                  </Button>
                </Link>
                <Link to='/wallet/receive'>
                  <Button variant={'outline'}>
                    <HandHelping className='mr-2 h-4 w-4' /> Receive
                  </Button>
                </Link>
                {asset && assetId !== 'xch' && (
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant='outline' size='icon'>
                        <MoreHorizontalIcon className='h-4 w-4' />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem onClick={() => setEditOpen(true)}>
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={redownload}>
                        Refresh Info
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setVisibility(!asset.visible)}
                      >
                        {asset.visible ? 'Hide' : 'Show'} Asset
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                )}
              </div>
            </CardContent>
          </Card>
          <CoinCard
            coins={coins}
            asset={asset}
            splitHandler={
              asset?.asset_id === 'xch' ? commands.split : commands.splitCat
            }
            combineHandler={
              asset?.asset_id === 'xch' ? commands.combine : commands.combineCat
            }
            setSummary={setSummary}
            selectedCoins={selectedCoins}
            setSelectedCoins={setSelectedCoins}
          />
        </div>
      </Container>

      <Dialog
        open={isEditOpen}
        onOpenChange={(open) => !open && setEditOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit Token Details</DialogTitle>
            <DialogDescription>
              Enter the new display details for this token
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>Name</Label>
              <Input
                id='name'
                placeholder='Name of this token'
                value={newName}
                onChange={(event) => setNewName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    edit();
                  }
                }}
              />
            </div>
          </div>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='ticker'>Ticker</Label>
              <Input
                id='ticker'
                placeholder='Ticker for this token'
                value={newTicker}
                onChange={(event) => setNewTicker(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    edit();
                  }
                }}
              />
            </div>
          </div>

          <DialogFooter className='gap-2'>
            <Button
              variant='outline'
              onClick={() => {
                setEditOpen(false);
                setNewName('');
                setNewTicker('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={edit} disabled={!newName || !newTicker}>
              Rename
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmationDialog
        summary={summary}
        close={() => setSummary(null)}
        onConfirm={() => setSelectedCoins({})}
      />
    </>
  );
}

interface CoinCardProps {
  coins: CoinRecord[];
  asset: CatRecord | null;
  splitHandler: typeof commands.split | null;
  combineHandler: typeof commands.combine | null;
  setSummary: (summary: TransactionSummary) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
}

function CoinCard({
  coins,
  asset,
  splitHandler,
  combineHandler,
  setSummary,
  selectedCoins,
  setSelectedCoins,
}: CoinCardProps) {
  const walletState = useWalletState();

  const selectedCoinIds = useMemo(() => {
    return Object.keys(selectedCoins).filter((key) => selectedCoins[key]);
  }, [selectedCoins]);

  const canCombine = useMemo(
    () =>
      selectedCoinIds.length >= 2 &&
      selectedCoinIds.every((id) => {
        const coin = coins.find((coin) => coin.coin_id === id);
        return (
          !coin?.spend_transaction_id &&
          !coin?.create_transaction_id &&
          coin?.created_height &&
          !coin?.spent_height
        );
      }),
    [selectedCoinIds, coins],
  );
  const canSplit = useMemo(
    () =>
      selectedCoinIds.length >= 1 &&
      selectedCoinIds.every((id) => {
        const coin = coins.find((coin) => coin.coin_id === id);
        return (
          !coin?.spend_transaction_id &&
          !coin?.create_transaction_id &&
          coin?.created_height &&
          !coin?.spent_height
        );
      }),
    [selectedCoinIds, coins],
  );

  const [isCombineOpen, setCombineOpen] = useState(false);
  const [isSplitOpen, setSplitOpen] = useState(false);

  const combineFormSchema = z.object({
    combineFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const combineForm = useForm<z.infer<typeof combineFormSchema>>({
    resolver: zodResolver(combineFormSchema),
    defaultValues: {
      combineFee: '0',
    },
  });

  const onCombineSubmit = (values: z.infer<typeof combineFormSchema>) => {
    combineHandler?.(selectedCoinIds, values.combineFee)
      .then((result) => {
        setCombineOpen(false);

        if (result.status === 'ok') {
          setSummary(result.data);
        }
      })
      .catch((error) => console.log('Failed to combine coins', error));
  };

  const splitFormSchema = z.object({
    outputCount: z.number().int().min(2).max(4294967295),
    splitFee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const splitForm = useForm<z.infer<typeof splitFormSchema>>({
    resolver: zodResolver(splitFormSchema),
    defaultValues: {
      outputCount: 2,
      splitFee: '0',
    },
  });

  const onSplitSubmit = (values: z.infer<typeof splitFormSchema>) => {
    splitHandler?.(selectedCoinIds, values.outputCount, values.splitFee)
      .then((result) => {
        setSplitOpen(false);

        if (result.status === 'ok') {
          setSummary(result.data);
        }
      })
      .catch((error) => console.log('Failed to split coins', error));
  };

  return (
    <Card className='max-w-full overflow-auto'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>Coins</CardTitle>
      </CardHeader>
      <CardContent>
        <CoinList
          coins={coins}
          selectedCoins={selectedCoins}
          setSelectedCoins={setSelectedCoins}
          actions={
            <>
              {splitHandler && (
                <Button
                  variant='outline'
                  disabled={!canSplit}
                  onClick={() => setSplitOpen(true)}
                >
                  <SplitIcon className='mr-2 h-4 w-4' /> Split
                </Button>
              )}
              {combineHandler && (
                <Button
                  variant='outline'
                  disabled={!canCombine}
                  onClick={() => setCombineOpen(true)}
                >
                  <MergeIcon className='mr-2 h-4 w-4' />
                  Combine
                </Button>
              )}
            </>
          }
        />
      </CardContent>

      <Dialog open={isCombineOpen} onOpenChange={setCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Combine {asset?.ticker}</DialogTitle>
            <DialogDescription>
              This will combine all of the selected coins into one.
            </DialogDescription>
          </DialogHeader>
          <Form {...combineForm}>
            <form
              onSubmit={combineForm.handleSubmit(onCombineSubmit)}
              className='space-y-4'
            >
              <FormField
                control={combineForm.control}
                name='combineFee'
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
                  onClick={() => setCombineOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Combine</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={isSplitOpen} onOpenChange={setSplitOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Split {asset?.ticker}</DialogTitle>
            <DialogDescription>
              This will split all of the selected coins.
            </DialogDescription>
          </DialogHeader>
          <Form {...splitForm}>
            <form
              onSubmit={splitForm.handleSubmit(onSplitSubmit)}
              className='space-y-4'
            >
              <FormField
                control={splitForm.control}
                name='outputCount'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Output Count</FormLabel>
                    <FormControl>
                      <Input
                        type='number'
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseInt(e.target.value))
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={splitForm.control}
                name='splitFee'
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
                  onClick={() => setSplitOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Split</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
