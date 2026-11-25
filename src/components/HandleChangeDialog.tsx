import { useHandleChangeStore } from '@/lib/handles';
import { Trans } from '@lingui/react/macro';
import { AlertTriangleIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
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
import { Label } from './ui/label';

/**
 * Asks the user to confirm sending to handles which now resolve to a different
 * address than before, showing both addresses.
 */
export function HandleChangeDialog() {
  const { changes, finish } = useHandleChangeStore();
  const [acknowledged, setAcknowledged] = useState(false);

  useEffect(() => {
    setAcknowledged(false);
  }, [changes]);

  return (
    <Dialog
      open={changes !== null}
      onOpenChange={(open) => {
        if (!open) finish(false);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle className='flex items-center gap-2'>
            <AlertTriangleIcon className='h-5 w-5 text-destructive' />
            <Trans>Handle changed</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              These handles resolve to a different address than when you last
              used them. They may have changed hands. Only continue if you trust
              the new addresses.
            </Trans>
          </DialogDescription>
        </DialogHeader>

        <div className='space-y-3 max-h-[50vh] overflow-auto'>
          {changes?.map((change) => (
            <div
              key={change.handle}
              className='rounded-md border border-destructive/40 p-3 text-sm space-y-2'
            >
              <div className='font-medium'>@{change.handle}</div>
              <div>
                <div className='text-xs text-muted-foreground'>
                  <Trans>Previous address</Trans>
                </div>
                <div className='font-mono text-xs break-all'>
                  {change.previousAddress}
                </div>
              </div>
              <div>
                <div className='text-xs text-muted-foreground'>
                  <Trans>New address</Trans>
                </div>
                <div className='font-mono text-xs break-all'>
                  {change.address}
                </div>
              </div>
            </div>
          ))}
        </div>

        <div className='flex items-center gap-2'>
          <Checkbox
            id='acknowledge-handle-change'
            checked={acknowledged}
            onCheckedChange={(checked) => setAcknowledged(checked === true)}
          />
          <Label htmlFor='acknowledge-handle-change'>
            <Trans>I want to send to the new addresses</Trans>
          </Label>
        </div>

        <DialogFooter className='gap-2'>
          <Button variant='outline' onClick={() => finish(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <Button
            variant='destructive'
            disabled={!acknowledged}
            onClick={() => finish(true)}
          >
            <Trans>Continue</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
