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
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  EyeIcon,
  EyeOff,
  MoreVerticalIcon,
  PenIcon,
  UserIcon,
  UserRoundPlus,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, DidRecord, events } from '../bindings';

export function WalletDids() {
  const navigate = useNavigate();

  const [showHidden, setShowHidden] = useState(false);
  const [dids, setDids] = useState<DidRecord[]>([]);

  const updateDids = async () => {
    return await commands.getDids().then((result) => {
      if (result.status === 'ok') {
        setDids(result.data);
      } else {
        throw new Error('Failed to get DIDs');
      }
    });
  };

  useEffect(() => {
    updateDids();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'did_info'
      ) {
        updateDids();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  const visibleDids = showHidden ? dids : dids.filter((did) => did.visible);
  const hasHiddenDids = dids.findIndex((did) => !did.visible) > -1;

  return (
    <>
      <Header title='Profiles'>
        <ReceiveAddress />
      </Header>
      <Container>
        <Button onClick={() => navigate('create-profile')} className='mb-4'>
          Create Profile
        </Button>

        {hasHiddenDids && (
          <div className='inline-flex items-center gap-2 ml-4'>
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
          {visibleDids
            .sort((a, b) => {
              if (a.visible !== b.visible) {
                return a.visible ? -1 : 1;
              }

              if (a.name && b.name) {
                return a.name.localeCompare(b.name);
              } else if (a.name) {
                return -1;
              } else if (b.name) {
                return 1;
              } else {
                return a.coin_id.localeCompare(b.coin_id);
              }
            })
            .map((did) => {
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
  const [name, setName] = useState('');
  const [renameOpen, setRenameOpen] = useState(false);

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
                    setRenameOpen(true);
                  }}
                >
                  <PenIcon className='mr-2 h-4 w-4' />
                  <span>Rename</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer text-red-600 focus:text-red-500'
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
    </>
  );
}
