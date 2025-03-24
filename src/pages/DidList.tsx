import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TransferDialog } from '@/components/TransferDialog';
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
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useDids } from '@/hooks/useDids';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  ActivityIcon,
  Copy,
  EyeIcon,
  EyeOff,
  Flame,
  MoreVerticalIcon,
  PenIcon,
  SendIcon,
  UserIcon,
  UserPlusIcon,
  UserRoundPlus,
} from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands, DidRecord, TransactionResponse } from '../bindings';
import { DidConfirmation } from '@/components/confirmations/DidConfirmation';

export function DidList() {
  const navigate = useNavigate();
  const { dids, updateDids } = useDids();
  const didsCount = dids.length;
  const [showHidden, setShowHidden] = useState(false);

  const visibleDids = showHidden ? dids : dids.filter((did) => did.visible);
  const hasHiddenDids = dids.findIndex((did) => !did.visible) > -1;

  return (
    <>
      <Header title={t`Profiles`}>
        <ReceiveAddress />
      </Header>
      <Container>
        <Button onClick={() => navigate('/dids/create')}>
          <UserPlusIcon className='h-4 w-4 mr-2' />
          <Trans>Create Profile</Trans>
        </Button>

        {hasHiddenDids && (
          <div className='flex items-center gap-2 my-4'>
            <label htmlFor='viewHidden'>
              <Trans>View hidden</Trans>
            </label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

        {didsCount === 0 && (
          <Alert className='mt-4'>
            <UserRoundPlus className='h-4 w-4' />
            <AlertTitle>
              <Trans>Create a profile?</Trans>
            </AlertTitle>
            <AlertDescription>
              <Plural
                value={didsCount}
                one='You do not currently have any DID profile. Would you like to create one?'
                other='You do not currently have any DID profiles. Would you like to create one?'
              />
            </AlertDescription>
          </Alert>
        )}

        <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          {visibleDids.map((did) => (
            <Profile key={did.launcher_id} did={did} updateDids={updateDids} />
          ))}
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
  const { addError } = useErrors();

  const walletState = useWalletState();

  const [name, setName] = useState('');
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const [renameOpen, setRenameOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [normalizeOpen, setNormalizeOpen] = useState(false);

  // Track which action is being performed
  const [isTransferring, setIsTransferring] = useState(false);
  const [isBurning, setIsBurning] = useState(false);
  const [isNormalizing, setIsNormalizing] = useState(false);
  const [transferAddress, setTransferAddress] = useState('');

  const rename = () => {
    if (!name) return;

    commands
      .updateDid({ did_id: did.launcher_id, name, visible: did.visible })
      .then(updateDids)
      .catch(addError)
      .finally(() => {
        setRenameOpen(false);
        setName('');
      });
  };

  const toggleVisibility = () => {
    commands
      .updateDid({
        did_id: did.launcher_id,
        name: did.name,
        visible: !did.visible,
      })
      .then(updateDids)
      .catch(addError);
  };

  const onTransferSubmit = (address: string, fee: string) => {
    setIsTransferring(true);
    setTransferAddress(address);
    commands
      .transferDids({
        did_ids: [did.launcher_id],
        address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsTransferring(false);
        addError(err);
      })
      .finally(() => setTransferOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    setIsBurning(true);
    commands
      .transferDids({
        did_ids: [did.launcher_id],
        address: walletState.sync.burn_address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsBurning(false);
        addError(err);
      })
      .finally(() => setBurnOpen(false));
  };

  const onNormalizeSubmit = (fee: string) => {
    setIsNormalizing(true);
    commands
      .normalizeDids({
        did_ids: [did.launcher_id],
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsNormalizing(false);
        addError(err);
      })
      .finally(() => setNormalizeOpen(false));
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
            {did.name ?? t`Untitled Profile`}
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
                  <span>
                    <Trans>Transfer</Trans>
                  </span>
                </DropdownMenuItem>

                {did.recovery_hash === null && (
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setNormalizeOpen(true);
                    }}
                  >
                    <ActivityIcon className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Normalize</Trans>
                    </span>
                  </DropdownMenuItem>
                )}

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setBurnOpen(true);
                  }}
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
                    writeText(did.launcher_id);
                    toast.success(t`DID ID copied to clipboard`);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Copy ID</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setRenameOpen(true);
                  }}
                >
                  <PenIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Rename</Trans>
                  </span>
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
                  <span>{did.visible ? t`Hide` : t`Show`}</span>
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
            <DialogTitle>
              <Trans>Rename Profile</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Enter the new display name for this profile.</Trans>
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>
                <Trans>Name</Trans>
              </Label>
              <Input
                id='name'
                placeholder={t`Profile name`}
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
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={rename} disabled={!name}>
              <Trans>Rename</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <TransferDialog
        title={t`Transfer Profile`}
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        <Trans>This will send the profile to the provided address.</Trans>
      </TransferDialog>

      <FeeOnlyDialog
        title={t`Burn Profile`}
        submitButtonLabel={t`Burn`}
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        <Trans>
          This will permanently delete the profile by sending it to the burn
          address.
        </Trans>
      </FeeOnlyDialog>

      <FeeOnlyDialog
        title={t`Normalize Profile`}
        submitButtonLabel={t`Normalize`}
        open={normalizeOpen}
        setOpen={setNormalizeOpen}
        onSubmit={onNormalizeSubmit}
      >
        <Trans>
          This will modify the profile's recovery info to be compatible with the
          Chia reference wallet.
        </Trans>
      </FeeOnlyDialog>

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => {
          setResponse(null);
          setIsTransferring(false);
          setIsBurning(false);
          setIsNormalizing(false);
        }}
        onConfirm={() => updateDids()}
        additionalData={
          isTransferring && response
            ? {
                title: t`Transfer DID`,
                content: (
                  <DidConfirmation
                    dids={[did]}
                    address={transferAddress}
                    type='transfer'
                  />
                ),
              }
            : isBurning && response
              ? {
                  title: t`Burn DID`,
                  content: <DidConfirmation dids={[did]} type='burn' />,
                }
              : isNormalizing && response
                ? {
                    title: t`Normalize DID`,
                    content: <DidConfirmation dids={[did]} type='normalize' />,
                  }
                : undefined
        }
      />
    </>
  );
}
