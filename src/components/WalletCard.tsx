import { EmojiPicker } from '@/components/EmojiPicker';
import { ResyncDialog } from '@/components/dialogs/ResyncDialog';
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
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  CopyIcon,
  EraserIcon,
  EyeIcon,
  FlameIcon,
  LogInIcon,
  MoreVerticalIcon,
  PenIcon,
  RefreshCcw,
  SnowflakeIcon,
  TrashIcon,
  WalletIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { Spoiler } from 'spoiled';
import { useTheme } from 'theme-o-rama';
import { commands, KeyInfo, SecretKeyInfo } from '../bindings';
import { CustomError } from '../contexts/ErrorContext';
import { useWallet } from '../contexts/WalletContext';
import { loginAndUpdateState, logoutAndUpdateState } from '../state';

interface WalletCardProps {
  draggable?: boolean;
  info: KeyInfo;
  keys: KeyInfo[];
  setKeys: (keys: KeyInfo[]) => void;
}

export function WalletCard({
  draggable,
  info,
  keys,
  setKeys,
}: WalletCardProps) {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { setWallet } = useWallet();
  const { promptIfEnabled } = useBiometric();

  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [secrets, setSecrets] = useState<SecretKeyInfo | null>(null);
  const [isRenameOpen, setIsRenameOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [isResyncOpen, setIsResyncOpen] = useState(false);
  const [isMigrationDialogOpen, setIsMigrationDialogOpen] = useState(false);
  const [isEmojiPickerOpen, setIsEmojiPickerOpen] = useState(false);
  const { currentTheme } = useTheme();

  const deleteSelf = async () => {
    if (await promptIfEnabled()) {
      await commands
        .deleteKey({ fingerprint: info.fingerprint })
        .then(() =>
          setKeys(keys.filter((key) => key.fingerprint !== info.fingerprint)),
        )
        .catch(addError);
    }

    setIsDeleteOpen(false);
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
      .finally(() => setIsRenameOpen(false));

    setNewName('');
  };

  const updateEmoji = (emoji: string | null) => {
    commands
      .setWalletEmoji({ fingerprint: info.fingerprint, emoji })
      .then(() =>
        setKeys(
          keys.map((key) =>
            key.fingerprint === info.fingerprint ? { ...key, emoji } : key,
          ),
        ),
      )
      .catch(addError);
  };

  const copyAddress = async () => {
    try {
      await commands.login({ fingerprint: info.fingerprint });
      const sync = await commands.getSyncStatus({});

      if (sync?.receive_address) {
        await writeText(sync.receive_address);
        toast.success(t`Address copied to clipboard`);
      } else {
        toast.error(t`No address found`);
      }
    } catch {
      toast.error(t`Failed to copy address to clipboard`);
    } finally {
      try {
        await commands.logout({});
      } catch (error) {
        console.error(error);
      }
    }
  };

  const loginSelf = async () => {
    try {
      await loginAndUpdateState(info.fingerprint);

      const data = await commands.getKey({});
      setWallet(data.key);
      navigate('/wallet');
    } catch (error: unknown) {
      if (
        typeof error === 'object' &&
        error !== null &&
        'kind' in error &&
        error.kind === 'database_migration'
      ) {
        setIsMigrationDialogOpen(true);
      } else {
        addError(error as CustomError);
      }
    }
  };

  useEffect(() => {
    (async () => {
      if (!isDetailsOpen || !(await promptIfEnabled())) {
        setSecrets(null);
        return;
      }

      commands
        .getSecretKey({ fingerprint: info.fingerprint })
        .then((data) => data.secrets !== null && setSecrets(data.secrets))
        .catch(addError);
    })();
  }, [isDetailsOpen, info.fingerprint, addError, promptIfEnabled]);

  const values = useSortable({
    id: draggable ? info.fingerprint : 'not-draggable',
  });

  let style: React.CSSProperties = {
    transform: CSS.Transform.toString(values.transform),
    transition: values.transition,
    opacity: values.isDragging ? 0 : 1,
  };

  if (!draggable) {
    values.listeners = {};
    style = {};
  }

  const networkId = info.network_id;

  return (
    <>
      <Card
        ref={values.setNodeRef}
        {...values.listeners}
        {...values.attributes}
        style={style}
        onClick={loginSelf}
        className='cursor-pointer'
      >
        <CardHeader className='flex flex-row items-center justify-between p-5 pt-4 pb-2'>
          <div className='flex items-center gap-3'>
            <EmojiPicker
              value={info.emoji}
              onChange={updateEmoji}
              open={isEmojiPickerOpen}
              onOpenChange={setIsEmojiPickerOpen}
            >
              <div
                className='cursor-pointer hover:opacity-80 transition-opacity'
                onClick={(e) => {
                  e.stopPropagation();
                }}
                title='Click to change emoji'
              >
                {info.emoji ? (
                  <span
                    className='text-3xl'
                    role='img'
                    aria-label='Wallet emoji'
                  >
                    {info.emoji}
                  </span>
                ) : (
                  <div className='w-8 h-8 rounded-full bg-muted flex items-center justify-center'>
                    <WalletIcon className='h-4 w-4 text-muted-foreground' />
                  </div>
                )}
              </div>
            </EmojiPicker>
            <CardTitle className='text-2xl'>{info.name}</CardTitle>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild className='-mr-2.5'>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end' data-no-dnd>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    loginSelf();
                    e.stopPropagation();
                  }}
                >
                  <LogInIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Login</Trans>
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={async (e) => {
                    e.stopPropagation();
                    await copyAddress();
                  }}
                >
                  <CopyIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Copy Address</Trans>
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    setIsDetailsOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <EyeIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Details</Trans>
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    setIsRenameOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <PenIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Rename</Trans>
                  </span>
                </DropdownMenuItem>
                {info.emoji && (
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      updateEmoji(null);
                      e.stopPropagation();
                    }}
                  >
                    <EraserIcon className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Remove Emoji</Trans>
                    </span>
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem
                  className='cursor-pointer text-red-600 focus:text-red-500'
                  onClick={(e) => {
                    setIsResyncOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <RefreshCcw className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Resync ({networkId})</Trans>
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer text-red-600 focus:text-red-500'
                  onClick={(e) => {
                    setIsDeleteOpen(true);
                    e.stopPropagation();
                  }}
                >
                  <TrashIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Delete</Trans>
                  </span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent className='p-0 px-5 pb-5'>
          <div className='flex items-center justify-between'>
            <span className='text-muted-foreground'>{info.fingerprint}</span>
            {info.has_secrets ? (
              <div className='inline-flex gap-1 items-center rounded-full px-3 py-1.5 text-xs bg-muted'>
                <FlameIcon className='h-4 w-4' />
                <span>
                  <Trans>Hot</Trans>
                </span>
              </div>
            ) : (
              <div className='inline-flex gap-1 items-center rounded-full px-3 py-1.5 text-xs bg-muted'>
                <SnowflakeIcon className='h-4 w-4' />
                <span>
                  <Trans>Cold</Trans>
                </span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <ResyncDialog
        open={isResyncOpen}
        setOpen={setIsResyncOpen}
        networkId={networkId}
        submit={async (options) => {
          await commands.resync({
            fingerprint: info.fingerprint,
            ...options,
          });
        }}
      />

      <Dialog
        open={isDeleteOpen}
        onOpenChange={(open) => !open && setIsDeleteOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Permanently Delete</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Are you sure you want to delete this wallet? This cannot be
                undone, and all funds will be lost unless you have saved your
                mnemonic phrase.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setIsDeleteOpen(false)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button variant='destructive' onClick={deleteSelf} autoFocus>
              <Trans>Delete</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isRenameOpen}
        onOpenChange={(open) => !open && setIsRenameOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Rename Wallet</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Enter the new display name for this wallet.</Trans>
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>
                <Trans>Wallet Name</Trans>
              </Label>
              <Input
                id='name'
                placeholder={t`Name of your wallet`}
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
                setIsRenameOpen(false);
                setNewName('');
              }}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={renameSelf} disabled={!newName}>
              <Trans>Rename</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isDetailsOpen}
        onOpenChange={(open) => !open && setIsDetailsOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Wallet Details</Trans>
            </DialogTitle>
          </DialogHeader>
          <div className='space-y-4'>
            <div>
              <h3 className='font-semibold'>
                <Trans>Network</Trans>
              </h3>
              <p className='break-all text-sm text-muted-foreground'>
                {networkId}
              </p>
            </div>
            <div>
              <h3 className='font-semibold'>
                <Trans>Public Key</Trans>
              </h3>
              <p className='break-all text-sm text-muted-foreground'>
                {info.public_key}
              </p>
            </div>
            {secrets && (
              <>
                <div>
                  <h3 className='font-semibold'>
                    <Trans>Secret Key</Trans>
                  </h3>
                  <p className='break-all text-sm text-muted-foreground'>
                    <Spoiler theme={currentTheme?.mostLike ?? 'light'}>
                      {secrets.secret_key}
                    </Spoiler>
                  </p>
                </div>
                {secrets.mnemonic && (
                  <div>
                    <h3 className='font-semibold'>
                      <Trans>Mnemonic</Trans>
                    </h3>
                    <p className='break-words text-sm text-muted-foreground'>
                      <Spoiler theme={currentTheme?.mostLike ?? 'light'}>
                        {secrets.mnemonic}
                      </Spoiler>
                    </p>
                  </div>
                )}
              </>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setIsDetailsOpen(false)}>
              <Trans>Done</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isMigrationDialogOpen}
        onOpenChange={(open) => !open && setIsMigrationDialogOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Database Migration Required</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                In order to proceed with the update, the wallet will be fully
                resynced. This means any imported offer files or custom asset
                names will be removed, but you can manually add them again after
                if needed.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant='outline'
              onClick={async () => {
                setIsMigrationDialogOpen(false);
                try {
                  await logoutAndUpdateState();
                } catch (error) {
                  console.error('Error during logout:', error);
                }
              }}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button
              variant='default'
              onClick={async () => {
                setIsMigrationDialogOpen(false);
                await logoutAndUpdateState();
                await commands.deleteDatabase({
                  fingerprint: info.fingerprint,
                  network: info.network_id,
                });
                await loginSelf();
              }}
              autoFocus
            >
              <Trans>OK</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
