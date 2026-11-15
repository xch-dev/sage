import { AddKeyDialog } from '@/components/dialogs/AddKeyDialog';
import { KeyDetailsDialog } from '@/components/dialogs/KeyDetailsDialog';
import { Button } from '@/components/ui/button';
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
import { Switch } from '@/components/ui/switch';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { EyeIcon, MoreVerticalIcon, PlusIcon, TrashIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { commands, KeyInfo, WalletInfo } from '../../bindings';
import { SettingItem, SettingsSection } from './SettingsSection';

export function SecuritySettings() {
  const isMobile = platform() === 'ios' || platform() === 'android';

  return (
    <>
      {isMobile && <BiometricSettings />}
      <KeySettings />
    </>
  );
}

function BiometricSettings() {
  const { enabled, available, enableIfAvailable, disable } = useBiometric();

  const toggleBiometric = async (value: boolean) => {
    if (value) {
      await enableIfAvailable();
    } else {
      await disable();
    }
  };

  return (
    <SettingsSection title={t`Authentication`}>
      <SettingItem
        label={t`Biometric Authentication`}
        description={t`Require biometrics for sensitive actions`}
        control={
          <Switch
            checked={enabled}
            disabled={!available}
            onCheckedChange={toggleBiometric}
          />
        }
      />
    </SettingsSection>
  );
}

function keyKindName(kind: KeyInfo['kind']) {
  switch (kind) {
    case 'bls':
      return 'BLS';
  }
}

/** The keys in the keychain. Wallets add and remove their own keys, so this is
 *  only needed for lower level access, such as keys that aren't wallets. */
function KeySettings() {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

  const [keys, setKeys] = useState<KeyInfo[] | null>(null);
  // Until the wallets load, it isn't known which keys they use.
  const [wallets, setWallets] = useState<WalletInfo[] | null>(null);
  const [pendingDelete, setPendingDelete] = useState(false);
  const [addOpen, setAddOpen] = useState(false);
  const [details, setDetails] = useState<KeyInfo | null>(null);
  const [deleting, setDeleting] = useState<KeyInfo | null>(null);

  const refresh = useCallback(() => {
    commands
      .getKeys({})
      .then((data) => setKeys(data.keys))
      .catch(addError);

    commands
      .getWallets({})
      .then((data) => setWallets(data.wallets))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // A BLS key belongs to the BLS wallet with the same fingerprint.
  const walletFor = (key: KeyInfo) =>
    wallets?.find(
      (wallet) =>
        wallet.fingerprint === key.fingerprint && wallet.kind === 'bls',
    ) ?? null;

  const deleteKey = async () => {
    if (!deleting) return;

    // Set before the prompt, so the button can't be pressed again while it's open.
    setPendingDelete(true);

    if (!(await promptIfEnabled())) {
      setPendingDelete(false);
      return;
    }

    commands
      .deleteKey({ fingerprint: deleting.fingerprint })
      .then(refresh)
      .catch(addError)
      .finally(() => {
        setPendingDelete(false);
        setDeleting(null);
      });
  };

  return (
    <>
      <SettingsSection title={t`Keys`}>
        <SettingItem
          label={t`Keychain`}
          description={t`Keys are added and removed along with their wallets. Keys that aren't wallets can be managed here.`}
          control={
            <Button
              variant='secondary'
              size='sm'
              onClick={() => setAddOpen(true)}
            >
              <PlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
              <Trans>Add</Trans>
            </Button>
          }
        />

        {keys?.length === 0 && (
          <div className='p-3 text-sm text-muted-foreground'>
            <Trans>The keychain is empty.</Trans>
          </div>
        )}

        {keys?.map((key) => {
          const wallet = walletFor(key);
          const fingerprint = key.fingerprint;
          const walletName = wallet?.name;

          return (
            <SettingItem
              key={fingerprint}
              label={fingerprint.toString()}
              description={
                walletName ? t`Used by ${walletName}` : t`Not used by a wallet`
              }
              control={
                <div className='flex items-center gap-2'>
                  <div className='inline-flex items-center rounded-full px-3 py-1.5 text-xs bg-muted'>
                    {keyKindName(key.kind)}
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        aria-label={t`Key actions`}
                      >
                        <MoreVerticalIcon className='h-5 w-5' />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align='end'>
                      <DropdownMenuGroup>
                        <DropdownMenuItem
                          className='cursor-pointer'
                          onClick={() => setDetails(key)}
                        >
                          <EyeIcon className='mr-2 h-4 w-4' />
                          <span>
                            <Trans>Details</Trans>
                          </span>
                        </DropdownMenuItem>
                        {wallets !== null && !wallet && (
                          <DropdownMenuItem
                            className='cursor-pointer text-red-600 focus:text-red-500'
                            onClick={() => setDeleting(key)}
                          >
                            <TrashIcon className='mr-2 h-4 w-4' />
                            <span>
                              <Trans>Delete</Trans>
                            </span>
                          </DropdownMenuItem>
                        )}
                      </DropdownMenuGroup>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              }
            />
          );
        })}
      </SettingsSection>

      <AddKeyDialog open={addOpen} setOpen={setAddOpen} onAdded={refresh} />

      <KeyDetailsDialog
        keyInfo={details}
        wallet={details ? walletFor(details) : null}
        onClose={() => setDetails(null)}
      />

      <Dialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Delete Key</Trans>
            </DialogTitle>
            <DialogDescription>
              {deleting?.has_secrets ? (
                <Trans>
                  Are you sure you want to delete this key? Its secret key will
                  be removed from this device, so make sure you have a backup.
                  This cannot be undone.
                </Trans>
              ) : (
                <Trans>
                  Are you sure you want to delete this key? This cannot be
                  undone.
                </Trans>
              )}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDeleting(null)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button
              variant='destructive'
              onClick={deleteKey}
              disabled={pendingDelete}
            >
              <Trans>Delete</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
