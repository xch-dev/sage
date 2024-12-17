import {
  commands,
  NftRecord,
  NftUriKind,
  TransactionResponse,
} from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { nftUri } from '@/lib/nftUri';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import BigNumber from 'bignumber.js';
import {
  Copy,
  EyeIcon,
  EyeOff,
  Flame,
  LinkIcon,
  MoreVertical,
  SendIcon,
  UserRoundMinus,
  UserRoundPlus,
} from 'lucide-react';
import { PropsWithChildren, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { z } from 'zod';
import { AssignNftDialog } from './AssignNftDialog';
import ConfirmationDialog from './ConfirmationDialog';
import { FeeOnlyDialog } from './FeeOnlyDialog';
import { TransferDialog } from './TransferDialog';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
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
  DropdownMenuSeparator,
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { TokenAmountInput } from './ui/masked-input';

export interface NftProps {
  nft: NftRecord;
  updateNfts: () => void;
  selectionState: [boolean, (value: boolean) => void] | null;
}

export function NftCardList({ children }: PropsWithChildren) {
  return (
    <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 mt-6 mb-2'>
      {children}
    </div>
  );
}

export function NftCard({ nft, updateNfts, selectionState }: NftProps) {
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [transferOpen, setTransferOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  const [unassignOpen, setUnassignOpen] = useState(false);
  const [addUrlOpen, setAddUrlOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const toggleVisibility = () => {
    commands
      .updateNft({ nft_id: nft.launcher_id, visible: !nft.visible })
      .then(updateNfts)
      .catch(addError);
  };

  const onTransferSubmit = (address: string, fee: string) => {
    commands
      .transferNfts({
        nft_ids: [nft.launcher_id],
        address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setTransferOpen(false));
  };

  const onAssignSubmit = (profile: string, fee: string) => {
    commands
      .assignNftsToDid({
        nft_ids: [nft.launcher_id],
        did_id: profile,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setAssignOpen(false));
  };

  const onUnassignSubmit = (fee: string) => {
    commands
      .assignNftsToDid({
        nft_ids: [nft.launcher_id],
        did_id: null,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setUnassignOpen(false));
  };

  const addUrlFormSchema = z.object({
    url: z.string().min(1, 'URL is required'),
    kind: z.string().min(1, 'Kind is required'),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const addUrlForm = useForm<z.infer<typeof addUrlFormSchema>>({
    resolver: zodResolver(addUrlFormSchema),
    defaultValues: {
      url: '',
      kind: 'data',
      fee: '0',
    },
  });

  const onAddUrlSubmit = (values: z.infer<typeof addUrlFormSchema>) => {
    commands
      .addNftUri({
        nft_id: nft.launcher_id,
        uri: values.url,
        kind: values.kind as NftUriKind,
        fee: toMojos(values.fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setAddUrlOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    commands
      .transferNfts({
        nft_ids: [nft.launcher_id],
        address: walletState.sync.burn_address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setBurnOpen(false));
  };

  return (
    <>
      <div
        className={`cursor-pointer group${`${!nft.visible ? ' opacity-50 grayscale' : !nft.created_height ? ' pulsate-opacity' : ''}`}`}
        onClick={() => {
          if (selectionState === null) {
            navigate(`/nfts/${nft.launcher_id}`);
          } else {
            selectionState[1](!selectionState[0]);
          }
        }}
      >
        <div className='overflow-hidden rounded-t-lg relative'>
          <img
            alt={nft.name ?? 'Unnamed'}
            loading='lazy'
            width='150'
            height='150'
            className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
            src={nftUri(nft.data_mime_type, nft.data)}
          />

          {selectionState !== null && (
            <Checkbox
              checked={selectionState[0]}
              className='absolute top-2 right-2 w-5 h-5'
            />
          )}
        </div>
        <div className=' border border-neutral-200 bg-white text-neutral-950 shadow dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
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
                    setAssignOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <UserRoundPlus className='mr-2 h-4 w-4' />
                  <span>
                    {nft.owner_did === null
                      ? 'Assign Profile'
                      : 'Reassign Profile'}
                  </span>
                </DropdownMenuItem>

                {nft.owner_did !== null && (
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setUnassignOpen(true);
                    }}
                    disabled={!nft.created_height}
                  >
                    <UserRoundMinus className='mr-2 h-4 w-4' />
                    <span>Unassign Profile</span>
                  </DropdownMenuItem>
                )}

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    addUrlForm.reset();
                    setAddUrlOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <LinkIcon className='mr-2 h-4 w-4' />
                  <span>Add URL</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setBurnOpen(true);
                  }}
                  disabled={!nft.created_height}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <span>Burn</span>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

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

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(nft.launcher_id);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' />
                  <span>Copy ID</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      <TransferDialog
        title='Transfer NFT'
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        This will send the NFT to the provided address.
      </TransferDialog>

      <AssignNftDialog
        title='Assign Profile'
        open={assignOpen}
        setOpen={setAssignOpen}
        onSubmit={onAssignSubmit}
      >
        This will assign the NFT to the selected profile.
      </AssignNftDialog>

      <FeeOnlyDialog
        title='Unassign Profile'
        open={unassignOpen}
        setOpen={setUnassignOpen}
        onSubmit={onUnassignSubmit}
      >
        This will unassign the NFT from its profile.
      </FeeOnlyDialog>

      <Dialog open={addUrlOpen} onOpenChange={setAddUrlOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add NFT URL</DialogTitle>
            <DialogDescription>
              This will add an additional URL to the NFT. It is not possible to
              remove URLs later, so be careful with this and try to use
              permanent URLs if possible.
            </DialogDescription>
          </DialogHeader>
          <Form {...addUrlForm}>
            <form
              onSubmit={addUrlForm.handleSubmit(onAddUrlSubmit)}
              className='space-y-4'
            >
              <FormField
                control={addUrlForm.control}
                name='url'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>URL</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={addUrlForm.control}
                name='kind'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Kind</FormLabel>
                    <FormControl>
                      <Select
                        name={field.name}
                        value={field.value}
                        onValueChange={field.onChange}
                      >
                        <SelectTrigger id='kind' aria-label='Select kind'>
                          <SelectValue placeholder='Select kind' />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value='data'>Data</SelectItem>
                          <SelectItem value='metadata'>Metadata</SelectItem>
                          <SelectItem value='license'>License</SelectItem>
                        </SelectContent>
                      </Select>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={addUrlForm.control}
                name='fee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Network Fee</FormLabel>
                    <FormControl>
                      <TokenAmountInput {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setAddUrlOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Add URL</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <FeeOnlyDialog
        title='Burn NFT'
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        This will permanently delete the NFT by sending it to the burn address.
      </FeeOnlyDialog>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => updateNfts()}
      />
    </>
  );
}
