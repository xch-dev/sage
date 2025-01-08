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
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { amount } from '@/lib/formTypes';
import { toDecimal, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { RowSelectionState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import {
  HandHelping,
  MergeIcon,
  MoreHorizontalIcon,
  Send,
  SplitIcon,
} from 'lucide-react';
import { MouseEventHandler, useEffect, useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import {
  CatRecord,
  CoinRecord,
  commands,
  events,
  TransactionResponse,
} from '../bindings';
// import { Trans, t } from '@lingui/react/macro';
// import { t } from '@lingui/core/macro';
import StyledQRCode from '@/components/StyledQrCode';
import { fetch } from '@tauri-apps/plugin-http';
import { CopyButton } from '@/components/CopyButton';
interface QRCodeDialogProps {
  isOpen: boolean;
  onClose: (open: boolean) => void;
  asset: CatRecord | null;
  receive_address: string;
}

const getImageDataUrl = async (url: string) => {
  try {
    const response = await fetch(url);
    const blob = await response.blob();
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
  } catch (error) {
    console.error('Failed to load image:', error);
    return null;
  }
};

const QRCodeCopyButton = ({ receive_address }: { receive_address: string }) => {
  const [copySuccess, setCopySuccess] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(receive_address);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000); // Reset after 2 seconds
    } catch (err) {
      console.error('Failed to copy text:', err);
    }
  };

  return (
    <Button
      size='lg'
      variant='secondary'
      onClick={handleCopy}
      className='w-full'
    >
      {copySuccess ? <Trans>Copied!</Trans> : <Trans>Copy</Trans>}
    </Button>
  );
};

const QRCodeDialog = ({
  isOpen,
  onClose,
  asset,
  receive_address,
}: QRCodeDialogProps) => {
  const [imageDataUrl, setImageDataUrl] = useState<string | undefined>(
    undefined,
  );
  useEffect(() => {
    if (asset?.icon_url) {
      getImageDataUrl(asset.icon_url)
        .then((dataUrl) => setImageDataUrl(dataUrl as string))
        .catch((error) => {
          console.error('Failed to load image:', error);
        });
    }
  }, [asset?.icon_url]);

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className='sm:max-w-md'>
        <DialogHeader>
          <DialogTitle>
            <Trans>Receive {asset?.ticker}</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Use this address to receive {asset?.name}</Trans>
          </DialogDescription>
        </DialogHeader>
        <div className='flex'>
          <div className='flex flex-col items-center justify-center'>
            <div className='py-4'>
              <StyledQRCode
                data={receive_address}
                cornersSquareOptions={{
                  type: 'extra-rounded',
                }}
                dotsOptions={{
                  type: 'rounded',
                  color: '#000000',
                }}
                backgroundOptions={{}}
                image={imageDataUrl}
                imageOptions={{
                  hideBackgroundDots: true,
                  imageSize: 0.4,
                  margin: 5,
                  saveAsBlob: true,
                }}
              />
            </div>
            <span className='text-center break-words break-all'>
              {receive_address}
            </span>
            <div className='pt-8 w-full'>
              <QRCodeCopyButton receive_address={receive_address} />
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default function Token() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { getBalanceInUsd } = usePrices();

  const { asset_id: assetId } = useParams();
  const { addError } = useErrors();

  const [asset, setAsset] = useState<CatRecord | null>(null);
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [selectedCoins, setSelectedCoins] = useState<RowSelectionState>({});
  const { receive_address } = walletState.sync;

  const precision = useMemo(
    () => (assetId === 'xch' ? walletState.sync.unit.decimals : 3),
    [assetId, walletState.sync.unit.decimals],
  );

  const balanceInUsd = useMemo(() => {
    if (!asset) return '0';
    return getBalanceInUsd(asset.asset_id, toDecimal(asset.balance, precision));
  }, [asset, precision, getBalanceInUsd]);

  const updateCoins = useMemo(
    () => () => {
      const getCoins =
        assetId === 'xch'
          ? commands.getXchCoins({})
          : commands.getCatCoins({ asset_id: assetId! });

      getCoins.then((res) => setCoins(res.coins)).catch(addError);
    },
    [assetId, addError],
  );

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
  }, [updateCoins]);

  const updateCat = useMemo(
    () => () => {
      commands
        .getCat({ asset_id: assetId! })
        .then((res) => setAsset(res.cat))
        .catch(addError);
    },
    [assetId, addError],
  );

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
  }, [assetId, updateCat, walletState.sync]);

  const redownload = () => {
    if (!assetId || assetId === 'xch') return;

    commands
      .removeCat({ asset_id: assetId })
      .then(() => updateCat())
      .catch(addError);
  };

  const setVisibility = (visible: boolean) => {
    if (!asset || assetId === 'xch') return;
    asset.visible = visible;

    commands
      .updateCat({ record: asset })
      .then(() => navigate('/wallet'))
      .catch(addError);
  };

  const [isEditOpen, setEditOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newTicker, setNewTicker] = useState('');
  const [isReceiveOpen, setReceiveOpen] = useState(false);

  const edit = () => {
    if (!newName || !newTicker || !asset) return;

    asset.name = newName;
    asset.ticker = newTicker;

    commands
      .updateCat({ record: asset })
      .then(() => updateCat())
      .catch(addError)
      .finally(() => setEditOpen(false));
  };

  return (
    <>
      <Header
        title={
          <span>
            {asset ? (asset.name ?? t`Unknown asset`) : ''}{' '}
            {asset?.asset_id !== 'xch' && (
              <CopyButton value={asset?.asset_id ?? ''} />
            )}
          </span>
        }
      />
      <Container>
        <div className='flex flex-col gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-col pb-2'>
              <div className='flex flex-row justify-between items-center space-y-0 space-x-2'>
                <div className='flex text-xl sm:text-4xl font-medium font-mono truncate'>
                  <span className='truncate'>
                    {toDecimal(asset?.balance ?? '0', precision)}
                    &nbsp;
                  </span>
                  {asset?.ticker}
                </div>
                <div className='flex-shrink-0'>
                  <img
                    alt='asset icon'
                    src={asset?.icon_url ?? ''}
                    className='h-8 w-8'
                  />
                </div>
              </div>
              <div className='text-sm text-muted-foreground'>
                ~${balanceInUsd}
              </div>
            </CardHeader>
            <CardContent className='flex flex-col gap-2'>
              <ReceiveAddress className='mt-2' />

              <div className='flex gap-2 mt-2 flex-wrap'>
                <Link to={`/wallet/send/${assetId}`}>
                  <Button>
                    <Send className='mr-2 h-4 w-4' /> <Trans>Send</Trans>
                  </Button>
                </Link>
                <Button
                  variant={'outline'}
                  onClick={() => setReceiveOpen(true)}
                >
                  <HandHelping className='mr-2 h-4 w-4' />
                  <Trans>Receive</Trans>
                </Button>
                {asset && assetId !== 'xch' && (
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant='outline' size='icon'>
                        <MoreHorizontalIcon className='h-4 w-4' />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem onClick={() => setEditOpen(true)}>
                        <Trans>Edit</Trans>
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={redownload}>
                        <Trans>Refresh Info</Trans>
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setVisibility(!asset.visible)}
                      >
                        {asset.visible ? t`Hide` : t`Show`} {t`Asset`}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                )}
              </div>
            </CardContent>
          </Card>
          <CoinCard
            precision={precision}
            coins={coins}
            asset={asset}
            splitHandler={
              asset?.asset_id === 'xch' ? commands.splitXch : commands.splitCat
            }
            combineHandler={
              asset?.asset_id === 'xch'
                ? commands.combineXch
                : commands.combineCat
            }
            setResponse={setResponse}
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
            <DialogTitle>
              <Trans>Edit Token Details</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Enter the new display details for this token</Trans>
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>
                <Trans>Name</Trans>
              </Label>
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
              <Label htmlFor='ticker'>
                <Trans>Ticker</Trans>
              </Label>
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
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={edit} disabled={!newName || !newTicker}>
              <Trans>Rename</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <QRCodeDialog
        isOpen={isReceiveOpen}
        onClose={setReceiveOpen}
        asset={asset}
        receive_address={receive_address}
      />

      {/* <Dialog open={isReceiveOpen} onOpenChange={setReceiveOpen}>
        <DialogContent className='sm:max-w-md'>
          <DialogHeader>
            <DialogTitle>
              <Trans>Receive {asset?.ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Use this address to receive {asset?.name}</Trans>
            </DialogDescription>
          </DialogHeader>
          <div className='flex flex-col gap-4 items-center'>
            <StyledQRCode
              data={receive_address}
              cornersSquareOptions={{
                type: 'extra-rounded',
              }}
              dotsOptions={{
                type: 'rounded',
                color: '#000000',
                // gradient: {
                //   type: 'linear',
                //   rotation: 45,
                //   colorStops: [
                //     { offset: 0, color: '#4267b2' },
                //     { offset: 1, color: '#00ff00' },
                //   ],
                // },
              }}
              backgroundOptions={
                {
                  // round: 10,
                  // color: '#e9ebee',
                }
              }
              image='https://icons.dexie.space/xch.webp'
              imageOptions={{
                hideBackgroundDots: true,
                imageSize: 0.4,
                margin: 20,
                saveAsBlob: true,
              }}
            />
          </div>
          <DialogFooter className='sm:justify-start'>
            <Button variant='secondary' onClick={() => setReceiveOpen(false)}>
              <Trans>Close</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog> */}

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => setSelectedCoins({})}
      />
    </>
  );
}

interface CoinCardProps {
  precision: number;
  coins: CoinRecord[];
  asset: CatRecord | null;
  splitHandler: typeof commands.splitXch | null;
  combineHandler: typeof commands.combineXch | null;
  setResponse: (response: TransactionResponse) => void;
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
}

function CoinCard({
  precision,
  coins,
  asset,
  splitHandler,
  combineHandler,
  setResponse,
  selectedCoins,
  setSelectedCoins,
}: CoinCardProps) {
  const walletState = useWalletState();
  const ticker = asset?.ticker;

  const { addError } = useErrors();

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
    combineHandler?.({
      coin_ids: selectedCoinIds,
      fee: toMojos(values.combineFee, walletState.sync.unit.decimals),
    })
      .then(setResponse)
      .catch(addError)
      .finally(() => setCombineOpen(false));
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
    splitHandler?.({
      coin_ids: selectedCoinIds,
      output_count: values.outputCount,
      fee: toMojos(values.splitFee, walletState.sync.unit.decimals),
    })
      .then(setResponse)
      .catch(addError)
      .finally(() => setSplitOpen(false));
  };

  return (
    <Card className='max-w-full overflow-auto'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Coins</Trans>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <CoinList
          precision={precision}
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
                  <SplitIcon className='mr-2 h-4 w-4' /> <Trans>Split</Trans>
                </Button>
              )}
              {combineHandler && (
                <Button
                  variant='outline'
                  disabled={!canCombine}
                  onClick={() => setCombineOpen(true)}
                >
                  <MergeIcon className='mr-2 h-4 w-4' />
                  <Trans>Combine</Trans>
                </Button>
              )}
            </>
          }
        />
      </CardContent>

      <Dialog open={isCombineOpen} onOpenChange={setCombineOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Combine {ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will combine all of the selected coins into one.
              </Trans>
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
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
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
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Combine</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={isSplitOpen} onOpenChange={setSplitOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Split {ticker}</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>This will split all of the selected coins.</Trans>
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
                    <FormLabel>
                      <Trans>Output Count</Trans>
                    </FormLabel>
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
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
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
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Split</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
