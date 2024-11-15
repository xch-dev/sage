import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
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
  DropdownMenuGroup,
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
import { Switch } from '@/components/ui/switch';
import { useDids } from '@/hooks/useDids';
import { amount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import BigNumber from 'bignumber.js';
import {
  EyeIcon,
  EyeOff,
  Flame,
  MoreVerticalIcon,
  PenIcon,
  SendIcon,
  UserIcon,
  UserRoundPlus,
} from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { commands, DidRecord, TransactionSummary } from '../bindings';

export function DidList() {
  const [showHidden, setShowHidden] = useState(false);

  const { dids, updateDids } = useDids();

  const visibleDids = showHidden ? dids : dids.filter((did) => did.visible);
  const hasHiddenDids = dids.findIndex((did) => !did.visible) > -1;

  return (
    <>
      <Header title='Profiles'>
        <ReceiveAddress />
      </Header>
      <Container>
        {hasHiddenDids && (
          <div className='inline-flex items-center gap-2 mb-2'>
            <label htmlFor='viewHidden'>View hidden</label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

        {visibleDids.length === 0 && (
          <Alert className='mt-2'>
            <UserRoundPlus className='h-4 w-4' />
            <AlertTitle>Create a profile?</AlertTitle>
            <AlertDescription>
              You do not currently have any {dids.length > 0 ? 'visible ' : ''}
              DID profiles. Would you like to create one?
            </AlertDescription>
          </Alert>
        )}

        <div className='mt-2 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          {visibleDids.map((did) => {
            return (
              <Profile
                key={did.launcher_id}
                did={did}
                updateDids={updateDids}
              />
            );
          })}
        </div>
      </Container>
    </>
  );
}

interface ProfileProps {
  did: DidRecord;
  updateDids: () => void;
}

function Profile({ did, updateDids }: ProfileProps) {
  const walletState = useWalletState();

  const [name, setName] = useState('');
  const [renameOpen, setRenameOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);

  const rename = () => {
    if (!name) return;

    commands.updateDid(did.launcher_id, name, did.visible).then((result) => {
      setRenameOpen(false);

      if (result.status === 'ok') {
        setName('');
        updateDids();
      } else {
        throw new Error(`Failed to rename DID: ${result.error.reason}`);
      }
    });
  };

  const toggleVisibility = () => {
    commands
      .updateDid(did.launcher_id, did.name, !did.visible)
      .then((result) => {
        if (result.status === 'ok') {
          updateDids();
        } else {
          throw new Error('Failed to toggle visibility for DID');
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
      .transferDids([did.launcher_id], values.address, values.fee)
      .then((result) => {
        setTransferOpen(false);
        if (result.status === 'error') {
          console.error('Failed to transfer DID', result.error);
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
      .transferDids(
        [did.launcher_id],
        walletState.sync.burn_address,
        values.fee,
      )
      .then((result) => {
        setBurnOpen(false);
        if (result.status === 'error') {
          console.error('Failed to burn DID', result.error);
        } else {
          setSummary(result.data);
        }
      });
  };

  return (
    <>
      <Card
        key={did.launcher_id}
        className={`${!did.visible ? 'opacity-50 grayscale' : did.create_transaction_id !== null ? 'pulsate-opacity' : ''}`}
      >
        <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
          <CardTitle className='text-md font-medium truncate flex items-center'>
            <UserIcon className='mr-2 h-4 w-4' />
            {did.name ?? 'Untitled Profile'}
          </CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' />
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
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>Transfer</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setBurnOpen(true);
                  }}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <span>Burn</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setRenameOpen(true);
                  }}
                >
                  <PenIcon className='mr-2 h-4 w-4' />
                  <span>Rename</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleVisibility();
                  }}
                >
                  {did.visible ? (
                    <EyeOff className='mr-2 h-4 w-4' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' />
                  )}
                  <span>{did.visible ? 'Hide' : 'Show'}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className='text-sm font-medium truncate'>{did.launcher_id}</div>
        </CardContent>
      </Card>

      <Dialog
        open={renameOpen}
        onOpenChange={(open) => !open && setRenameOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Profile</DialogTitle>
            <DialogDescription>
              Enter the new display name for this profile.
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>Name</Label>
              <Input
                id='name'
                placeholder='Profile name'
                value={name}
                onChange={(event) => setName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    rename();
                  }
                }}
              />
            </div>
          </div>
          <DialogFooter className='gap-2'>
            <Button
              variant='outline'
              onClick={() => {
                setRenameOpen(false);
                setName('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={rename} disabled={!name}>
              Rename
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={transferOpen} onOpenChange={setTransferOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Transfer Profile</DialogTitle>
            <DialogDescription>
              This will send the profile to the provided address.
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
            <DialogTitle>Burn Profile</DialogTitle>
            <DialogDescription>
              This will permanently delete the profile by sending it to the burn
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
        onConfirm={() => updateDids()}
      />
    </>
  );
}
