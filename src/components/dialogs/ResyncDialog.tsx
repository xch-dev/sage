import { Resync } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
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
  const [deleteAddresses, setDeleteAddresses] = useState(false);
  const [deleteOffers, setDeleteOffers] = useState(false);

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
              Are you sure you want to resync this wallet's data? This will
              re-download data from the network which can take a while depending
              on the size of the wallet.
            </Trans>
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
              <label htmlFor='deleteAddresses'>
                <Trans>Delete addresses</Trans>
              </label>
              <Switch
                id='deleteAddresses'
                checked={deleteAddresses}
                onCheckedChange={(value) => setDeleteAddresses(value)}
              />
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='outline' onClick={() => setOpen(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <Button
            variant='destructive'
            onClick={() => {
              setPending(true);
              submit({
                delete_addresses: deleteAddresses,
                delete_offer_files: deleteOffers,
                delete_blockinfo: false,
              })
                .catch(addError)
                .finally(() => {
                  setPending(false);
                  setOpen(false);
                });
            }}
            autoFocus
            disabled={pending}
          >
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {pending ? <Trans>Resyncing</Trans> : <Trans>Resync</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
