import { Resync } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useState } from 'react';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { LoadingButton } from '../ui/loading-button';
import { Switch } from '../ui/switch';

export interface ResyncDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  networkId?: string;
  submit: (options: Omit<Resync, 'fingerprint'>) => Promise<void>;
}

export function ResyncDialog({
  open,
  setOpen,
  networkId,
  submit,
}: ResyncDialogProps) {
  const { addError } = useErrors();

  const [pending, setPending] = useState(false);
  const [deleteCoins, setDeleteCoins] = useState(false);
  const [deleteOffers, setDeleteOffers] = useState(false);
  const [deleteCache, setDeleteCache] = useState(false);

  return (
    <Dialog open={open} onOpenChange={(open) => !open && setOpen(false)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {networkId ? (
              <Trans>Resync on {networkId}</Trans>
            ) : (
              <Trans>Resync</Trans>
            )}
          </DialogTitle>
          <DialogDescription>
            <Trans>
              Are you sure you want to resync this wallet&apos;s data? This will
              re-download data from the network which can take a while depending
              on the size of the wallet.
            </Trans>

            <div className='flex items-center gap-2 my-2'>
              <label htmlFor='deleteCoins'>
                <Trans>Delete coin data</Trans>
              </label>
              <Switch
                id='deleteCoins'
                checked={deleteCoins}
                onCheckedChange={(value) => setDeleteCoins(value)}
              />
            </div>

            <div className='flex items-center gap-2 my-2'>
              <label htmlFor='deleteOffers'>
                <Trans>Delete saved offer files</Trans>
              </label>
              <Switch
                id='deleteOffers'
                checked={deleteOffers}
                onCheckedChange={(value) => setDeleteOffers(value)}
              />
            </div>

            <div className='flex items-center gap-2 my-2'>
              <label htmlFor='deleteCache'>
                <Trans>Delete all cached data</Trans>
              </label>
              <Switch
                id='deleteCache'
                checked={deleteCache}
                onCheckedChange={(value) => setDeleteCache(value)}
              />
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='outline' onClick={() => setOpen(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <LoadingButton
            variant='destructive'
            loading={pending}
            loadingText={t`Resyncing`}
            onClick={() => {
              setPending(true);
              submit({
                delete_coins: deleteCoins,
                delete_offers: deleteOffers,
                // This will reset the derivation index, so we don't want to do this
                delete_addresses: false,
                delete_assets: deleteCache,
                delete_files: deleteCache,
                delete_blocks: deleteCache,
              })
                .catch(addError)
                .finally(() => {
                  setPending(false);
                  setOpen(false);
                });
            }}
            autoFocus
          >
            <Trans>Resync</Trans>
          </LoadingButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
