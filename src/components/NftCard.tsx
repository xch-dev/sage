import { commands, NftRecord, TransactionSummary } from '@/bindings';
import { amount } from '@/lib/formTypes';
import { nftUri } from '@/lib/nftUri';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import BigNumber from 'bignumber.js';
import { EyeIcon, EyeOff, Flame, MoreVertical, SendIcon } from 'lucide-react';
import { PropsWithChildren, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link } from 'react-router-dom';
import { z } from 'zod';
import ConfirmationDialog from './ConfirmationDialog';
import { Button } from './ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from './ui/form';
import { Input } from './ui/input';

export interface NftProps {
  nft: NftRecord;
  updateNfts: () => void;
}

export function NftCardList({ children }: PropsWithChildren) {
  return (
    <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mt-6 mb-2'>
      {children}
    </div>
  );
}

export function NftCard({ nft, updateNfts }: NftProps) {
  const walletState = useWalletState();

  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);

  const toggleVisibility = () => {
    commands.updateNft(nft.launcher_id, !nft.visible).then((result) => {
      if (result.status === 'ok') {
        updateNfts();
      } else {
        throw new Error('Failed to toggle visibility for NFT');
      }
    });
  };

  const transferFormSchema = z.object({
    address: z.string().min(1, 'Address is required'),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const transferForm = useForm<z.infer<typeof transferFormSchema>>({
    resolver: zodResolver(transferFormSchema),
    defaultValues: {
      address: '',
      fee: '0',
    },
  });

  const onTransferSubmit = (values: z.infer<typeof transferFormSchema>) => {
    commands
      .transferNft(nft.launcher_id, values.address, values.fee)
      .then((result) => {
        setTransferOpen(false);
        if (result.status === 'error') {
          console.error('Failed to transfer NFT', result.error);
        } else {
          setSummary(result.data);
        }
      });
  };

  const burnFormSchema = z.object({
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const burnForm = useForm<z.infer<typeof burnFormSchema>>({
    resolver: zodResolver(burnFormSchema),
    defaultValues: {
      fee: '0',
    },
  });

  const onBurnSubmit = (values: z.infer<typeof burnFormSchema>) => {
    commands
      .transferNft(nft.launcher_id, walletState.sync.burn_address, values.fee)
      .then((result) => {
        setBurnOpen(false);
        if (result.status === 'error') {
          console.error('Failed to burn NFT', result.error);
        } else {
          setSummary(result.data);
        }
      });
  };

  return (
    <>
      <Link
        to={`/nfts/${nft.launcher_id}`}
        className={`group${`${!nft.visible ? ' opacity-50 grayscale' : !nft.created_height ? ' pulsate-opacity' : ''}`}`}
      >
        <div className='overflow-hidden rounded-t-md relative'>
          <img
            alt={nft.name ?? 'Unnamed'}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={nftUri(nft.data_mime_type, nft.data)}
          />
        </div>
        <div className='text-md flex items-center justify-between rounded-b p-1 pl-2 bg-neutral-200 dark:bg-neutral-800'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {nft.name ?? 'Unnamed'}
            </span>
            <p className='text-xs text-muted-foreground truncate'>
              {nft.collection_name ?? 'No collection'}
            </p>
          </span>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon'>
                <MoreVertical className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    transferForm.reset();
                    setTransferOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>Transfer</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    burnForm.reset();
                    setBurnOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <span>Burn</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
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

      <Dialog open={transferOpen} onOpenChange={setTransferOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Transfer NFT</DialogTitle>
            <DialogDescription>
              This will send the NFT to the provided address.
            </DialogDescription>
          </DialogHeader>
          <Form {...transferForm}>
            <form
              onSubmit={transferForm.handleSubmit(onTransferSubmit)}
              className='space-y-4'
            >
              <FormField
                control={transferForm.control}
                name='address'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Address</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={transferForm.control}
                name='fee'
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
                  onClick={() => setTransferOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Transfer</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog open={burnOpen} onOpenChange={setBurnOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Burn NFT</DialogTitle>
            <DialogDescription>
              This will permanently delete the NFT by sending it to the burn
              address.
            </DialogDescription>
          </DialogHeader>
          <Form {...burnForm}>
            <form
              onSubmit={burnForm.handleSubmit(onBurnSubmit)}
              className='space-y-4'
            >
              <FormField
                control={burnForm.control}
                name='fee'
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
                  onClick={() => setBurnOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Burn</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <ConfirmationDialog
        summary={summary}
        close={() => setSummary(null)}
        onConfirm={() => updateNfts()}
      />
    </>
  );
}
