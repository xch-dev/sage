import {
  commands,
  NftData,
  NftRecord,
  NftUriKind,
  TransactionResponse,
} from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { nftUri } from '@/lib/nftUri';
import { toMojos } from '@/lib/utils';
import { useOfferState, useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import BigNumber from 'bignumber.js';
import {
  Copy,
  EyeIcon,
  EyeOff,
  Flame,
  HandCoins,
  LinkIcon,
  MoreVertical,
  SendIcon,
  UserRoundPlus,
} from 'lucide-react';
import { PropsWithChildren, useEffect, useState } from 'react';
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
import { TokenAmountInput } from './ui/masked-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';

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
  const offerState = useOfferState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [data, setData] = useState<NftData | null>(null);
  const [transferOpen, setTransferOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  const [addUrlOpen, setAddUrlOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  useEffect(() => {
    commands
      .getNftData({ nft_id: nft.launcher_id })
      .then((response) => setData(response.data))
      .catch(addError);
  }, [nft.launcher_id, addError]);

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

  const onAssignSubmit = (profile: string | null, fee: string) => {
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

  const addUrlFormSchema = z.object({
    url: z.string().min(1, t`URL is required`),
    kind: z.string().min(1, t`Kind is required`),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
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
        className={`cursor-pointer group${!nft.visible ? ' opacity-50 grayscale' : !nft.created_height ? ' pulsate-opacity' : ''}`}
        onClick={() => {
          if (selectionState === null) {
            navigate(`/nfts/${nft.launcher_id}`);
          } else {
            selectionState[1](!selectionState[0]);
          }
        }}
      >
        <div className='overflow-hidden rounded-t-lg relative'>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <img
                  alt={nft.name ?? t`Unnamed`}
                  loading='lazy'
                  width='150'
                  height='150'
                  className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
                  src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
                />
              </TooltipTrigger>
              <TooltipContent>
                <p>{nft.name ?? t`Unnamed`}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>

          {selectionState !== null && (
            <Checkbox
              checked={selectionState[0]}
              className='absolute top-2 right-2 w-5 h-5'
            />
          )}
        </div>
        <div className='border border-neutral-200 bg-white text-neutral-950 shadow dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {nft.name ?? t`Unnamed`}
            </span>
            <p className='text-xs text-muted-foreground truncate'>
              {nft.collection_name ?? t`No collection`}
            </p>
            <p className='text-xs text-muted-foreground truncate'>
              DEBUG {nft.owner_did ?? t`No owner`}
            </p>
          </span>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon' aria-label={t`NFT options`}>
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
                  <span>
                    <Trans>Transfer</Trans>
                  </span>
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
                    {nft.owner_did === null ? (
                      <Trans>Assign Profile</Trans>
                    ) : (
                      <Trans>Edit Profile</Trans>
                    )}
                  </span>
                </DropdownMenuItem>

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
                  <span>
                    <Trans>Add URL</Trans>
                  </span>
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
                  <span>
                    <Trans>Burn</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();

                    const newNfts = [...offerState.offered.nfts];

                    newNfts.push(nft.launcher_id);

                    useOfferState.setState({
                      offered: {
                        ...offerState.offered,
                        nfts: newNfts,
                      },
                    });
                  }}
                  disabled={
                    !nft.created_height ||
                    offerState.offered.nfts.findIndex(
                      (nftId) => nftId === nft.launcher_id,
                    ) !== -1
                  }
                >
                  <HandCoins className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Add to Offer</Trans>
                  </span>
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
                  <span>
                    {nft.visible ? <Trans>Hide</Trans> : <Trans>Show</Trans>}
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(nft.launcher_id);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Copy ID</Trans>
                  </span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      <TransferDialog
        title={t`Transfer NFT`}
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        <Trans>This will send the NFT to the provided address.</Trans>
      </TransferDialog>

      <AssignNftDialog
        title={t`Assign Profile`}
        open={assignOpen}
        setOpen={setAssignOpen}
        onSubmit={onAssignSubmit}
      >
        <Trans>This will assign the NFT to the selected profile.</Trans>
      </AssignNftDialog>

      <Dialog open={addUrlOpen} onOpenChange={setAddUrlOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Add NFT URL</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will add an additional URL to the NFT. It is not possible
                to remove URLs later, so be careful with this and try to use
                permanent URLs if possible.
              </Trans>
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
                    <FormLabel>
                      <Trans>URL</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input {...field} placeholder={t`Enter URL`} />
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
                    <FormLabel>
                      <Trans>Kind</Trans>
                    </FormLabel>
                    <FormControl>
                      <Select
                        name={field.name}
                        value={field.value}
                        onValueChange={field.onChange}
                      >
                        <SelectTrigger id='kind' aria-label={t`Select kind`}>
                          <SelectValue placeholder={t`Select kind`} />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value='data'>
                            <Trans>Data</Trans>
                          </SelectItem>
                          <SelectItem value='metadata'>
                            <Trans>Metadata</Trans>
                          </SelectItem>
                          <SelectItem value='license'>
                            <Trans>License</Trans>
                          </SelectItem>
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
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
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
                  <Trans>Cancel</Trans>
                </Button>
                <Button type='submit'>
                  <Trans>Add URL</Trans>
                </Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <FeeOnlyDialog
        title={t`Burn NFT`}
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        <Trans>
          This will permanently delete the NFT by sending it to the burn
          address.
        </Trans>
      </FeeOnlyDialog>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => updateNfts()}
      />
    </>
  );
}
