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
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import {
  EraserIcon,
  EyeIcon,
  FlameIcon,
  LogInIcon,
  MoreVerticalIcon,
  PenIcon,
  SnowflakeIcon,
  TrashIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, KeyInfo, SecretKeyInfo } from '../bindings';
import Container from '../components/Container';
import { loginAndUpdateState } from '../state';

export default function Login() {
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [keys, setKeys] = useState<KeyInfo[] | null>(null);
  const [network, setNetwork] = useState<string | null>(null);

  useEffect(() => {
    commands
      .getKeys({})
      .then((data) => setKeys(data.keys))
      .catch(addError);

    commands
      .networkConfig()
      .then((data) => setNetwork(data.network_id))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    commands
      .getKey({})
      .then((data) => data.key !== null && navigate('/wallet'))
      .catch(addError);
  }, [navigate, addError]);

  return (
    <div className='flex-1 space-y-4 p-8 pt-6'>
      <div className='flex items-center justify-between space-y-2'>
        {(keys?.length ?? 0) > 0 && (
          <>
            <h2 className='text-3xl font-bold tracking-tight'>Wallets</h2>
            <div className='flex items-center space-x-2'>
              <Button variant='outline' onClick={() => navigate('/import')}>
                Import
              </Button>
              <Button onClick={() => navigate('/create')}>Create</Button>
            </div>
          </>
        )}
      </div>
      {keys !== null ? (
        keys.length ? (
          <div className='grid md:grid-cols-2 lg:grid-cols-3 gap-4'>
            {keys.map((key, i) => (
              <WalletItem
                key={i}
                network={network}
                info={key}
                keys={keys}
                setKeys={setKeys}
              />
            ))}
          </div>
        ) : (
          <Welcome />
        )
      ) : (
        <SkeletonWalletList />
      )}
    </div>
  );
}

function SkeletonWalletList() {
  return (
    <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 m-4'>
      {Array.from({ length: 3 }).map((_, i) => (
        <div key={i} className='w-full'>
          <Skeleton className='h-[100px] w-full' />
        </div>
      ))}
    </div>
  );
}

interface WalletItemProps {
  network: string | null;
  info: KeyInfo;
  keys: KeyInfo[];
  setKeys: (keys: KeyInfo[]) => void;
}

function WalletItem({ network, info, keys, setKeys }: WalletItemProps) {
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [anchorEl, _setAnchorEl] = useState<HTMLElement | null>(null);
  const isMenuOpen = Boolean(anchorEl);

  const [isDeleteOpen, setDeleteOpen] = useState(false);

  const [isDetailsOpen, setDetailsOpen] = useState(false);
  const [secrets, setSecrets] = useState<SecretKeyInfo | null>(null);

  const [isRenameOpen, setRenameOpen] = useState(false);
  const [newName, setNewName] = useState('');

  const [isResyncOpen, setResyncOpen] = useState(false);
  const [deleteOffers, setDeleteOffers] = useState(false);

  const resyncSelf = () => {
    commands
      .resync({
        fingerprint: info.fingerprint,
        delete_offer_files: deleteOffers,
      })
      .catch(addError)
      .finally(() => setResyncOpen(false));
  };

  const deleteSelf = () => {
    commands
      .deleteKey({ fingerprint: info.fingerprint })
      .then(() =>
        setKeys(keys.filter((key) => key.fingerprint !== info.fingerprint)),
      )
      .catch(addError)
      .finally(() => setDeleteOpen(false));
  };

  const renameSelf = () => {
    if (!newName) return;

    commands
      .renameKey({ fingerprint: info.fingerprint, name: newName })
      .then(() =>
        setKeys(
          keys.map((key) =>
            key.fingerprint === info.fingerprint
              ? { ...key, name: newName }
              : key,
          ),
        ),
      )
      .catch(addError)
      .finally(() => setRenameOpen(false));

    setNewName('');
  };

  const loginSelf = (explicit: boolean) => {
    if (isMenuOpen && !explicit) return;

    loginAndUpdateState(info.fingerprint).then(() => {
      navigate('/wallet');
    });
  };

  useEffect(() => {
    if (!isDetailsOpen) {
      setSecrets(null);
      return;
    }

    commands
      .getSecretKey({ fingerprint: info.fingerprint })
      .then((data) => data.secrets !== null && setSecrets(data.secrets))
      .catch(addError);
  }, [isDetailsOpen, info.fingerprint, addError]);

  return (
    <>
      <Card onClick={() => loginSelf(false)} className='cursor-pointer'>
        <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
          <CardTitle className='text-2xl'>{info.name}</CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild className='-mr-2.5'>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    loginSelf(false);
                    e.stopPropagation();
                  }}
                >
                  <LogInIcon className='mr-2 h-4 w-4' />
                  <span>Login</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    setDetailsOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <EyeIcon className='mr-2 h-4 w-4' />
                  <span>Details</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    setRenameOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <PenIcon className='mr-2 h-4 w-4' />
                  <span>Rename</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer text-red-600 focus:text-red-500'
                  onClick={(e) => {
                    setResyncOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <EraserIcon className='mr-2 h-4 w-4' />
                  <span>Resync ({network})</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer text-red-600 focus:text-red-500'
                  onClick={(e) => {
                    setDeleteOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <TrashIcon className='mr-2 h-4 w-4' />
                  <span>Delete</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className='flex items-center mt-1 justify-between'>
            <span className='text-muted-foreground'>{info.fingerprint}</span>
            {info.has_secrets ? (
              <div className='inline-flex gap-0.5 items-center rounded-full border px-2 py-1.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80'>
                <FlameIcon className='h-4 w-4 pb-0.5' />
                <span>Hot Wallet</span>
              </div>
            ) : (
              <div className='inline-flex gap-0.5 items-center rounded-full border px-2 py-1.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80'>
                <SnowflakeIcon className='h-4 w-4 pb-0.5' />
                <span>Cold Wallet</span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Dialog
        open={isResyncOpen}
        onOpenChange={(open) => !open && setResyncOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Resync on {network}</DialogTitle>
            <DialogDescription>
              Are you sure you want to resync this wallet's data? This will
              redownload data from the network which can take a while depending
              on the size of the wallet.
              <div className='flex items-center gap-2 my-2'>
                <label htmlFor='deleteOffers'>Delete saved offer files</label>
                <Switch
                  id='deleteOffers'
                  checked={deleteOffers}
                  onCheckedChange={(value) => setDeleteOffers(value)}
                />
              </div>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setResyncOpen(false)}>
              Cancel
            </Button>
            <Button variant='destructive' onClick={resyncSelf} autoFocus>
              Resync
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isDeleteOpen}
        onOpenChange={(open) => !open && setDeleteOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Permanently Delete</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this wallet? This cannot be
              undone, and all funds will be lost unless you have saved your
              mnemonic phrase.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button variant='destructive' onClick={deleteSelf} autoFocus>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isRenameOpen}
        onOpenChange={(open) => !open && setRenameOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Wallet</DialogTitle>
            <DialogDescription>
              Enter the new display name for this wallet.
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>Wallet Name</Label>
              <Input
                id='name'
                placeholder='Name of your wallet'
                value={newName}
                onChange={(event) => setNewName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    renameSelf();
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
                setNewName('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={renameSelf} disabled={!newName}>
              Rename
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isDetailsOpen}
        onOpenChange={(open) => !open && setDetailsOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Wallet Details</DialogTitle>
          </DialogHeader>
          <div className='space-y-4'>
            <div>
              <h3 className='font-semibold'>Public Key</h3>
              <p className='break-all text-sm text-muted-foreground'>
                {info.public_key}
              </p>
            </div>
            {secrets && (
              <>
                <div>
                  <h3 className='font-semibold'>Secret Key</h3>
                  <p className='break-all text-sm text-muted-foreground'>
                    {secrets.secret_key}
                  </p>
                </div>
                {secrets.mnemonic && (
                  <div>
                    <h3 className='font-semibold'>Mnemonic</h3>
                    <p className='break-words text-sm text-muted-foreground'>
                      {secrets.mnemonic}
                    </p>
                  </div>
                )}
              </>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setDetailsOpen(false)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

function Welcome() {
  const navigate = useNavigate();

  return (
    <Container>
      <div className='text-center text-6xl'>Sage Wallet</div>

      <div className='text-center mt-4'>
        There aren't any wallets to log into yet. To get started, create a new
        wallet or import an existing one.
      </div>

      <div className='flex justify-center gap-4 mt-6'>
        <Button variant='outline' onClick={() => navigate('/import')}>
          Import Wallet
        </Button>
        <Button onClick={() => navigate('/create')}>Create Wallet</Button>
      </div>
    </Container>
  );
}
